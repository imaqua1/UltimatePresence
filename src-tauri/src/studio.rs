//! Roblox Studio source: a tiny HTTP server on localhost:4455 that speaks the
//! StudioPresence plugin's protocol, plus a watcher that notices when Studio exits.
//!
//! Protocol (unchanged from the original server):
//!   POST /  {"updateType": "SET_ACTIVITY", "activity": {...}}  -> set activity
//!   POST /  {"updateType": "CLOSE"}  (no activity)               -> clear
//!   While details == "Testing", other updates are ignored for 3 seconds.
//!   Response: 200 "SET Activity".

use crate::activity::{Activity, Button};
use crate::platform;
use crate::settings::{SourceSettings, Template};
use crate::template;
use crate::state::{lock_core, update};
use serde_json::{Map, Value};
use std::time::{Duration, Instant};
use tauri::AppHandle;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};

pub const PORT: u16 = 4455;
pub const TESTING_HOLD: Duration = Duration::from_secs(3);
/// The plugin only POSTs when `details` changes, so silence alone doesn't mean
/// Studio is gone. We clear after this much silence *and* no Studio process.
pub const SILENCE_TIMEOUT: Duration = Duration::from_secs(10);
const MAX_HEADER: usize = 16 * 1024;
const MAX_BODY: usize = 256 * 1024;

#[derive(Debug, Default)]
pub struct StudioState {
    pub activity: Option<Activity>,
    /// Last activity that wasn't a playtest (used when playtests are hidden).
    pub last_non_testing: Option<Activity>,
    /// What the user is doing (kind + script), and since when (unix ms).
    task_key: String,
    pub task_since_ms: Option<i64>,
    pub last_request: Option<Instant>,
    pub last_testing: Option<Instant>,
    pub running: bool,
    pub server_ok: bool,
    pub server_error: Option<String>,
    pub old_server_warned: bool,
}

#[derive(Debug, PartialEq)]
pub enum Outcome {
    Set { connected: bool, playtest: Option<bool> },
    Ignored,
    Cleared { had_activity: bool, close: bool },
}

/// Applies one plugin request body to the state. Pure, so it can be unit-tested.
pub fn apply_request(st: &mut StudioState, body: &str, now: Instant) -> Outcome {
    let connected = st.activity.is_none()
        && st.last_request.map_or(true, |t| now.duration_since(t) > SILENCE_TIMEOUT);
    st.last_request = Some(now);

    let root: Option<Value> = serde_json::from_str(body).ok();
    let close = root.as_ref().and_then(|r| r.get("updateType")).and_then(Value::as_str) == Some("CLOSE");
    let Some(obj) = root.as_ref().and_then(|r| r.get("activity")).and_then(Value::as_object) else {
        // Original: `if (!data) client.clearActivity()` — covers CLOSE, empty and invalid bodies.
        let had_activity = st.activity.take().is_some();
        return Outcome::Cleared { had_activity, close };
    };

    let details = obj.get("details").and_then(Value::as_str);
    let is_testing = details == Some("Testing");
    if is_testing {
        st.last_testing = Some(now);
    } else if st.last_testing.is_some_and(|t| now.duration_since(t) < TESTING_HOLD) {
        return Outcome::Ignored;
    }

    let was_testing = st.activity.as_ref().and_then(|a| a.details.as_deref()) == Some("Testing");
    let playtest = match (was_testing, is_testing) {
        (false, true) => Some(true),
        (true, false) => Some(false),
        _ => None,
    };

    let activity = activity_from_plugin(obj);

    // Original server also checked updateType on the activity object itself.
    if obj.get("updateType").and_then(Value::as_str) == Some("CLOSE") {
        st.activity = None;
        return Outcome::Cleared { had_activity: true, close: true };
    }

    let parsed = parse(&activity);
    let key = format!("{}:{}", parsed.kind.key(), parsed.script.as_deref().unwrap_or(""));
    if key != st.task_key {
        st.task_key = key;
        st.task_since_ms = Some(now_ms());
    }
    if parsed.kind != Kind::Testing {
        st.last_non_testing = Some(activity.clone());
    }
    st.activity = Some(activity);
    Outcome::Set { connected, playtest }
}

pub fn should_time_out(st: &StudioState, now: Instant) -> bool {
    st.activity.is_some()
        && !st.running
        && st.last_request.map_or(true, |t| now.duration_since(t) >= SILENCE_TIMEOUT)
}

fn as_text(v: Option<&Value>) -> Option<String> {
    match v? {
        Value::String(s) => Some(s.clone()),
        Value::Number(n) => Some(n.to_string()),
        _ => None,
    }
}

pub fn activity_from_plugin(o: &Map<String, Value>) -> Activity {
    let assets = o.get("assets").and_then(Value::as_object);
    let asset = |k: &str| as_text(assets.and_then(|m| m.get(k)));

    // The plugin sends os.time() (seconds); Discord wants milliseconds.
    let start_timestamp = o
        .get("timestamps")
        .and_then(|t| t.get("start"))
        .and_then(Value::as_f64)
        .filter(|v| v.is_finite() && *v > 0.0)
        .map(|v| if v < 1e11 { (v * 1000.0) as i64 } else { v as i64 });

    // "none" is the plugin's placeholder for "no small icon".
    let small_image = asset("small_image").filter(|k| k != "none");
    // The plugin writes the small tooltip into `small_image_key` (typo), so fall back to it.
    let small_text = small_image
        .as_ref()
        .and_then(|_| asset("small_text").or_else(|| asset("small_image_key")));

    let buttons = o
        .get("buttons")
        .and_then(Value::as_array)
        .map(|arr| {
            arr.iter()
                .filter_map(|b| {
                    Some(Button {
                        label: b.get("label")?.as_str()?.to_owned(),
                        url: b.get("url")?.as_str()?.to_owned(),
                    })
                })
                .collect()
        })
        .unwrap_or_default();

    Activity {
        details: as_text(o.get("details")),
        state: as_text(o.get("state")),
        start_timestamp,
        large_image: asset("large_image"),
        large_text: asset("large_text"),
        small_image,
        small_text,
        buttons,
    }
}

// ---------------------------------------------------------------------------
// Presentation: plugin data → structured parts → your templates and privacy
// ---------------------------------------------------------------------------

fn now_ms() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_millis() as i64)
        .unwrap_or(0)
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Kind {
    Editing,
    Developing,
    Testing,
    Animating,
    /// Particle / VFX plugins (Qwinkle's Part-Icles, …).
    Vfx,
    /// An animation tool and a VFX tool open at the same time.
    Combo,
    /// Something this version doesn't recognise; shown as the plugin sent it.
    Other,
}

impl Kind {
    pub fn key(self) -> &'static str {
        match self {
            Kind::Editing => "editing",
            Kind::Developing => "developing",
            Kind::Testing => "testing",
            Kind::Animating => "animating",
            Kind::Vfx => "vfx",
            Kind::Combo => "combo",
            Kind::Other => "other",
        }
    }
}

pub const KINDS: [Kind; 6] = [Kind::Editing, Kind::Developing, Kind::Testing, Kind::Animating, Kind::Vfx, Kind::Combo];

/// What the plugin's text means.
#[derive(Clone, Debug, PartialEq)]
pub struct Parsed {
    pub kind: Kind,
    pub script: Option<String>,
    pub line: Option<u32>,
    pub lines: Option<u32>,
    pub script_type: Option<&'static str>,
    pub place: Option<String>,
    /// The animation or VFX plugin in use (e.g. "Moon Animator 2").
    pub tool: Option<String>,
}

/// Small-image keys a (forked) plugin uses to say which tool is open.
const ANIMATOR_BADGES: &[&str] = &["moonanimator", "moonanimator2", "eclipseanimator"];
const VFX_BADGES: &[&str] = &["qwinkleparticles", "qwinkle", "vfx"];

pub fn parse(a: &Activity) -> Parsed {
    let d = a.details.as_deref().unwrap_or("").trim();
    let place = a
        .state
        .as_deref()
        .map(|s| s.strip_prefix("Workspace:").unwrap_or(s).trim().to_owned())
        .filter(|p| !p.is_empty());
    let script_type = match a.large_image.as_deref() {
        Some("serverscript") => Some("server script"),
        Some("clientscript") => Some("local script"),
        Some("modulescript") => Some("module script"),
        _ => None,
    };
    let mut p = Parsed { kind: Kind::Other, script: None, line: None, lines: None, script_type, place, tool: None };

    let badge = a.small_image.as_deref().unwrap_or("");
    let tool = || a.large_text.clone().filter(|t| !t.contains("StudioPresence") && !t.starts_with("UltimatePresence") && !t.starts_with("Editing"));
    if d == "Testing" {
        p.kind = Kind::Testing;
    } else if d == "Animating & making VFX" || badge == "animatingvfx" {
        p.kind = Kind::Combo;
        p.tool = tool();
    } else if d == "Making VFX" || VFX_BADGES.contains(&badge) {
        p.kind = Kind::Vfx;
        p.tool = tool();
    } else if d == "Animating" || ANIMATOR_BADGES.contains(&badge) {
        p.kind = Kind::Animating;
        p.tool = tool();
    } else if d == "Developing" {
        p.kind = Kind::Developing;
    } else if let (Some(idx), true) = (d.rfind(" ("), d.ends_with(')')) {
        // "<Script> (Line X / Y)" or "<Script> (N lines)"
        let inner = &d[idx + 2..d.len() - 1];
        let nums: Vec<u32> = inner.split(|c: char| !c.is_ascii_digit()).filter_map(|n| n.parse().ok()).collect();
        if inner.starts_with("Line ") && nums.len() == 2 {
            p.kind = Kind::Editing;
            p.line = Some(nums[0]);
            p.lines = Some(nums[1]);
        } else if inner.ends_with(" lines") && nums.len() == 1 {
            p.kind = Kind::Editing;
            p.lines = Some(nums[0]);
        }
        if p.kind == Kind::Editing {
            p.script = Some(d[..idx].to_owned()).filter(|s| !s.is_empty());
        }
    }
    p
}

/// Built-in wording styles. `custom` falls back to classic for kinds left empty.
pub fn preset(style: &str, kind: Kind) -> Template {
    let t = |top: &str, bottom: &str| Template { top: top.into(), bottom: bottom.into() };
    match (style, kind) {
        ("minimal", Kind::Editing) => t("Scripting", "{place}"),
        ("minimal", Kind::Developing) => t("Building", "{place}"),
        ("minimal", Kind::Testing) => t("Playtesting", "{place}"),
        ("minimal", Kind::Animating) => t("Animating", "{place}"),
        ("minimal", Kind::Vfx) => t("Making VFX", "{place}"),
        ("minimal", Kind::Combo) => t("Animating & VFX", "{place}"),
        ("detailed", Kind::Editing) => t("Editing {script}", "[Line {line} of {lines} · ]{place}"),
        ("detailed", Kind::Developing) => t("Building in Studio", "{place}"),
        ("detailed", Kind::Testing) => t("Playtesting", "{place}"),
        ("detailed", Kind::Animating) => t("Animating[ with {tool}]", "{place}"),
        ("detailed", Kind::Vfx) => t("Making VFX[ with {tool}]", "{place}"),
        ("detailed", Kind::Combo) => t("Animating & making VFX", "[{tool} · ]{place}"),
        (_, Kind::Editing) => t("{script}[ (Line {line} / {lines})]", "[Workspace: {place}]"),
        (_, Kind::Developing) => t("Developing", "[Workspace: {place}]"),
        (_, Kind::Testing) => t("Testing", "[Workspace: {place}]"),
        (_, Kind::Animating) => t("Animating", "[Workspace: {place}]"),
        (_, Kind::Vfx) => t("Making VFX", "[Workspace: {place}]"),
        (_, Kind::Combo) => t("Animating & making VFX", "[Workspace: {place}]"),
        (_, Kind::Other) => t("{details}", "{state}"),
    }
}

fn template_for(s: &SourceSettings, kind: Kind) -> Template {
    if s.style == "custom" {
        if let Some(t) = s.templates.get(kind.key()).filter(|t| !t.top.trim().is_empty() || !t.bottom.trim().is_empty()) {
            return t.clone();
        }
    }
    preset(&s.style, kind)
}

/// Result of applying settings to what Studio sent.
pub struct Presented {
    pub activity: Option<Activity>,
    /// Why nothing is shown although Studio sent something (for the window).
    pub reason: Option<String>,
}

/// Premade UltimatePresence images for a state: (large, badge). StudioPresence's own
/// Discord app doesn't have them, so they're only used with your own application.
/// UltimatePresence's own images only exist in your own Discord application;
/// StudioPresence's original one keeps its own images.
fn own_app(s: &SourceSettings) -> bool {
    // StudioPresence's original application (from v0.1) doesn't have these images.
    s.application_id != "1028311936854675458"
}

/// The Zzz image for idle.
pub fn idle_icon(s: &SourceSettings) -> Option<&'static str> {
    own_app(s).then_some("icon_idle")
}

/// Premade large image for a state (badges are always the app's own icon).
pub fn state_icon(kind: Kind, s: &SourceSettings) -> Option<&'static str> {
    if !own_app(s) {
        return None;
    }
    match kind {
        Kind::Developing => Some("icon_building"),
        Kind::Testing => Some("icon_playtest"),
        Kind::Animating => Some("icon_animating"),
        Kind::Vfx => Some("icon_vfx"),
        Kind::Combo => Some("icon_combo"),
        _ => None,
    }
}

/// The small badge is always Roblox Studio's own icon.
pub const STUDIO_BADGE: &str = "studio";

/// You count as idle after this long without keyboard or mouse input; your status then says "Idle".
pub const IDLE_AFTER: Duration = Duration::from_secs(5 * 60);

/// Hover text on every large image. (The original plugin sent "StudioPresence by iArxic".)
pub const STUDIO_HOVER: &str = "UltimatePresence by aqua";

pub fn is_hidden_place(s: &SourceSettings, place: Option<&str>) -> bool {
    place.is_some_and(|p| s.hidden_places.iter().any(|h| h.eq_ignore_ascii_case(p)))
}

/// Renders one raw activity with the settings (no idle / playtest rules).
#[cfg(test)]
pub fn render(raw: &Activity, s: &SourceSettings, task_since_ms: Option<i64>) -> Activity {
    render_as(raw, s, task_since_ms, false)
}

/// Wording for the idle state (away from the keyboard for a while).
pub fn idle_preset(style: &str) -> Template {
    let t = |top: &str, bottom: &str| Template { top: top.into(), bottom: bottom.into() };
    match style {
        "minimal" => t("Idle", "{place}"),
        "detailed" => t("Idle", "[Last in {place}]"),
        _ => t("Idle", "[Workspace: {place}]"),
    }
}

/// Renders an activity; with `idle` it uses the idle wording and Zzz icon instead.
pub fn render_as(raw: &Activity, s: &SourceSettings, task_since_ms: Option<i64>, idle: bool) -> Activity {
    let p = parse(raw);
    let action = match p.kind {
        Kind::Editing => "Editing",
        Kind::Developing => "Building",
        Kind::Testing => "Playtesting",
        Kind::Animating => "Animating",
        Kind::Vfx => "Making VFX",
        Kind::Combo => "Animating & making VFX",
        Kind::Other => "Working",
    };
    let alias = if s.place_alias.is_empty() { "a private project".to_owned() } else { s.place_alias.clone() };
    let var = |name: &str| -> Option<String> {
        match name {
            "action" => Some(action.to_owned()),
            "script" => p.script.as_ref().map(|n| if s.hide_script { "a script".to_owned() } else { n.clone() }),
            "line" => p.line.filter(|_| !s.hide_lines).map(|n| n.to_string()),
            "lines" => p.lines.filter(|_| !s.hide_lines).map(|n| n.to_string()),
            "type" => p.script_type.map(str::to_owned),
            "place" => p.place.as_ref().map(|n| if s.hide_place { alias.clone() } else { n.clone() }),
            "tool" | "animator" => p.tool.clone().filter(|_| !s.hide_tools),
            "details" => raw.details.clone().filter(|_| !s.hide_script),
            "state" => raw.state.clone().filter(|_| !s.hide_place),
            _ => None,
        }
    };
    let t = if idle {
        s.templates
            .get("idle")
            .filter(|t| s.style == "custom" && (!t.top.trim().is_empty() || !t.bottom.trim().is_empty()))
            .cloned()
            .unwrap_or_else(|| idle_preset(&s.style))
    } else {
        template_for(s, p.kind)
    };
    let top = template::render(&t.top, &var);
    let bottom = template::render(&t.bottom, &var);

    // Fixed icons: the state's own picture (script type while editing), and the app's badge.
    let large_image = if idle { idle_icon(s) } else { state_icon(p.kind, s) }
        .map(str::to_owned)
        .or_else(|| raw.large_image.clone());
    let small_image = Some(STUDIO_BADGE.to_owned());

    Activity {
        details: Some(top).filter(|t| !t.is_empty()),
        state: Some(bottom).filter(|t| !t.is_empty()),
        start_timestamp: match s.timer.as_str() {
            "off" => None,
            "activity" => task_since_ms.or(raw.start_timestamp),
            _ => raw.start_timestamp,
        },
        // Hover text: the large image credits this app, the badge names the app you're in.
        // (Never the plugin or tool, so nothing is spoiled.)
        large_text: large_image.as_ref().map(|_| STUDIO_HOVER.to_owned()),
        small_text: small_image.as_ref().map(|_| "Roblox Studio".to_owned()),
        large_image,
        small_image,
        buttons: s.buttons.iter().map(|b| Button { label: b.label.clone(), url: b.url.clone() }).collect(),
    }
}

/// `away`: no keyboard or mouse input for IDLE_AFTER. (The plugin only reports changes to
/// the script, line or playtest, so its silence can't tell building from being away.)
pub fn present(st: &StudioState, s: &SourceSettings, away: bool) -> Presented {
    let none = |reason: Option<String>| Presented { activity: None, reason };
    let Some(mut raw) = st.activity.as_ref() else { return none(None) };

    if parse(raw).kind == Kind::Testing && !s.show_testing {
        match st.last_non_testing.as_ref() {
            Some(prev) => raw = prev,
            None => return none(Some("Playtesting is hidden".into())),
        }
    }
    if is_hidden_place(s, parse(raw).place.as_deref()) {
        return none(Some("Hidden while you’re in this place".into()));
    }
    Presented { activity: Some(render_as(raw, s, st.task_since_ms, away)), reason: None }
}

/// Sample plugin data for previews when Studio isn't connected.
pub fn sample(kind: Kind) -> Activity {
    let base = Activity {
        state: Some("Workspace: Obby Tycoon".into()),
        start_timestamp: Some(now_ms() - (23 * 60 + 14) * 1000),
        large_image: Some("studio".into()),
        large_text: Some(STUDIO_HOVER.into()),
        ..Default::default()
    };
    match kind {
        Kind::Editing => Activity {
            details: Some("DataStoreHandler (Line 42 / 310)".into()),
            large_image: Some("serverscript".into()),
            large_text: Some("Editing a SERVER Script".into()),
            ..base
        },
        Kind::Testing => Activity {
            details: Some("Testing".into()),
            small_image: Some("playtesticon".into()),
            small_text: Some("Testing".into()),
            ..base
        },
        Kind::Animating => Activity {
            details: Some("Animating".into()),
            large_text: Some("Moon Animator 2".into()),
            small_image: Some("moonanimator".into()),
            small_text: Some("Animating".into()),
            ..base
        },
        Kind::Vfx => Activity {
            details: Some("Making VFX".into()),
            large_text: Some("Qwinkle's Part-Icles".into()),
            small_image: Some("qwinkleparticles".into()),
            small_text: Some("Making VFX".into()),
            ..base
        },
        Kind::Combo => Activity {
            details: Some("Animating & making VFX".into()),
            large_text: Some("Eclipse Animator + Qwinkle's Part-Icles".into()),
            small_image: Some("animatingvfx".into()),
            small_text: Some("Animating & making VFX".into()),
            ..base
        },
        _ => Activity { details: Some("Developing".into()), ..base },
    }
}

// ---------------------------------------------------------------------------
// Runtime: HTTP server + process watcher
// ---------------------------------------------------------------------------

pub fn start(app: &AppHandle) {
    let a = app.clone();
    tauri::async_runtime::spawn(async move { serve(a).await });
    let a = app.clone();
    tauri::async_runtime::spawn(async move { watch(a).await });
}

async fn serve(app: AppHandle) {
    let mut reported: Option<String> = None;
    loop {
        match TcpListener::bind(("127.0.0.1", PORT)).await {
            Ok(v4) => {
                // "localhost" may resolve to ::1 inside Studio, so listen there too (loopback only).
                if let Ok(v6) = TcpListener::bind(("::1", PORT)).await {
                    tauri::async_runtime::spawn(accept_loop(v6, app.clone()));
                }
                update(&app, |c| {
                    c.studio.server_ok = true;
                    c.studio.server_error = None;
                    c.info(format!("Listening for Roblox Studio on localhost:{PORT}"));
                });
                accept_loop(v4, app.clone()).await;
                return;
            }
            Err(e) => {
                let msg = if e.kind() == std::io::ErrorKind::AddrInUse {
                    format!("Port {PORT} is already in use. Close the old StudioPresence server (studiopresence-win.exe) or whatever else is using it. Retrying…")
                } else {
                    format!("Can't listen on port {PORT}: {e}. Retrying…")
                };
                if reported.as_deref() != Some(msg.as_str()) {
                    update(&app, |c| {
                        c.studio.server_ok = false;
                        c.studio.server_error = Some(msg.clone());
                        c.error(msg.clone());
                    });
                    reported = Some(msg);
                }
                tokio::time::sleep(Duration::from_secs(5)).await;
            }
        }
    }
}

async fn accept_loop(listener: TcpListener, app: AppHandle) {
    loop {
        match listener.accept().await {
            Ok((sock, _)) => {
                let app = app.clone();
                tauri::async_runtime::spawn(async move {
                    let _ = tokio::time::timeout(Duration::from_secs(10), handle_conn(sock, &app)).await;
                });
            }
            Err(_) => tokio::time::sleep(Duration::from_millis(100)).await,
        }
    }
}

fn find(hay: &[u8], needle: &[u8]) -> Option<usize> {
    hay.windows(needle.len()).position(|w| w == needle)
}

/// Returns the decoded body once the terminating chunk has arrived.
fn decode_chunked(mut data: &[u8]) -> Option<Vec<u8>> {
    let mut out = Vec::new();
    loop {
        let line_end = find(data, b"\r\n")?;
        let size_str = std::str::from_utf8(&data[..line_end]).ok()?;
        let size = usize::from_str_radix(size_str.split(';').next()?.trim(), 16).ok()?;
        data = &data[line_end + 2..];
        if size == 0 {
            return Some(out);
        }
        if data.len() < size + 2 {
            return None;
        }
        out.extend_from_slice(&data[..size]);
        data = &data[size + 2..];
    }
}

async fn respond(sock: &mut TcpStream, code: u16, ctype: &str, body: &str) -> std::io::Result<()> {
    let reason = match code {
        200 => "OK",
        403 => "Forbidden",
        405 => "Method Not Allowed",
        413 => "Payload Too Large",
        431 => "Request Header Fields Too Large",
        _ => "Error",
    };
    let msg = format!(
        "HTTP/1.1 {code} {reason}\r\nContent-Type: {ctype}\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",
        body.len()
    );
    sock.write_all(msg.as_bytes()).await?;
    sock.shutdown().await
}

async fn handle_conn(mut sock: TcpStream, app: &AppHandle) -> std::io::Result<()> {
    let mut buf = Vec::with_capacity(4096);
    let mut chunk = [0u8; 4096];
    let head_end = loop {
        if let Some(p) = find(&buf, b"\r\n\r\n") {
            break p;
        }
        if buf.len() > MAX_HEADER {
            return respond(&mut sock, 431, "text/plain", "Header too large").await;
        }
        let n = sock.read(&mut chunk).await?;
        if n == 0 {
            return Ok(());
        }
        buf.extend_from_slice(&chunk[..n]);
    };

    let head = String::from_utf8_lossy(&buf[..head_end]).into_owned();
    let mut lines = head.split("\r\n");
    let mut request_line = lines.next().unwrap_or("").split_whitespace();
    let method = request_line.next().unwrap_or("").to_ascii_uppercase();
    let path = request_line.next().unwrap_or("/").to_owned();

    let (mut content_length, mut chunked, mut origin, mut expect) = (0usize, false, None::<String>, false);
    for line in lines {
        if let Some((k, v)) = line.split_once(':') {
            let v = v.trim();
            match k.trim().to_ascii_lowercase().as_str() {
                "content-length" => content_length = v.parse().unwrap_or(0),
                "transfer-encoding" => chunked = v.to_ascii_lowercase().contains("chunked"),
                "origin" => origin = Some(v.to_owned()),
                "expect" => expect = v.eq_ignore_ascii_case("100-continue"),
                _ => {}
            }
        }
    }

    // Browsers always send Origin on cross-site requests; Studio doesn't. This stops
    // random web pages from changing your Discord status via localhost.
    if let Some(o) = origin {
        update(app, |c| c.warn(format!("Blocked a request from a web page ({o})")));
        return respond(&mut sock, 403, "text/plain", "Forbidden").await;
    }

    let mut body = buf.split_off(head_end + 4);
    if expect && body.is_empty() {
        sock.write_all(b"HTTP/1.1 100 Continue\r\n\r\n").await?;
    }
    if chunked {
        loop {
            if let Some(decoded) = decode_chunked(&body) {
                body = decoded;
                break;
            }
            if body.len() > MAX_BODY {
                return respond(&mut sock, 413, "text/plain", "Too large").await;
            }
            let n = sock.read(&mut chunk).await?;
            if n == 0 {
                return Ok(());
            }
            body.extend_from_slice(&chunk[..n]);
        }
    } else {
        if content_length > MAX_BODY {
            return respond(&mut sock, 413, "text/plain", "Too large").await;
        }
        while body.len() < content_length {
            let n = sock.read(&mut chunk).await?;
            if n == 0 {
                break;
            }
            body.extend_from_slice(&chunk[..n]);
        }
        body.truncate(content_length);
    }

    match method.as_str() {
        "POST" if path.starts_with("/blender") => {
            crate::blender::handle_post(app, &String::from_utf8_lossy(&body));
            respond(&mut sock, 200, "text/plain", "OK").await
        }
        "POST" => {
            handle_post(app, &String::from_utf8_lossy(&body));
            respond(&mut sock, 200, "text/plain", "SET Activity").await
        }
        "GET" if path == "/" || path.starts_with("/status") => {
            let json = status_json(app);
            respond(&mut sock, 200, "application/json", &json).await
        }
        _ => respond(&mut sock, 405, "text/plain", "Method Not Allowed").await,
    }
}

fn handle_post(app: &AppHandle, body: &str) {
    update(app, |c| {
        match apply_request(&mut c.studio, body, Instant::now()) {
            Outcome::Set { connected, playtest } => {
                if connected {
                    c.info("Roblox Studio connected");
                }
                match playtest {
                    Some(true) => c.info("Playtest started"),
                    Some(false) => c.info("Playtest ended"),
                    None => {}
                }
            }
            Outcome::Cleared { had_activity: true, close } => {
                c.info(if close { "Studio ended the session (CLOSE), presence cleared" } else { "Studio sent an empty update, presence cleared" });
            }
            Outcome::Cleared { .. } | Outcome::Ignored => {}
        }
    });
}

/// Small status endpoint for a future plugin fork (show the app's state inside Studio).
fn status_json(app: &AppHandle) -> String {
    let snap = lock_core(app).snapshot();
    let studio = snap.source(crate::sources::ROBLOX_STUDIO);
    serde_json::json!({
        "app": "UltimatePresence",
        "version": snap.version,
        "discord": snap.discord_status,
        "paused": snap.paused,
        "enabled": studio.map(|s| s.settings.enabled),
        "privacy": studio.map(|s| s.settings.hide_script || s.settings.hide_place),
        "studio": studio.map(|s| s.status),
        "activeSource": snap.active_source,
        "sharing": snap.active_source == Some(crate::sources::ROBLOX_STUDIO) && snap.discord_status == "connected",
        "shown": snap.shown,
    })
    .to_string()
}

/// Once a second: which apps are running, which one is in front, and time-based rules
/// (timeouts, idle).
async fn watch(app: AppHandle) {
    let mut tick = tokio::time::interval(Duration::from_secs(1));
    loop {
        tick.tick().await;
        let (names, foreground) = tauri::async_runtime::spawn_blocking(|| (platform::process_names(), platform::foreground_process_name()))
            .await
            .unwrap_or_default();
        let running = names.iter().any(|n| platform::is_studio_process(n));
        let blender_running = names.iter().any(|n| platform::is_blender_process(n));
        let focused = foreground.as_deref().and_then(platform::source_for_process);
        let old_server = names.iter().any(|n| n.to_ascii_lowercase().starts_with("studiopresence"));
        let now = Instant::now();

        let needs_update = {
            let c = lock_core(&app);
            c.studio.running != running
                || c.blender.running != blender_running
                || c.focused != focused
                || (!running && should_time_out(&c.studio, now))
                || crate::blender::should_time_out(&c.blender, now)
                || c.is_stale()
                || (old_server && !c.studio.old_server_warned)
        };
        if needs_update {
            update(&app, |c| {
                if c.focused != focused {
                    c.focused = focused;
                }
                if let Some(id) = focused {
                    c.focus.insert(id, now);
                }
                if c.blender.running != blender_running {
                    c.blender.running = blender_running;
                    c.info(if blender_running { "Blender is running" } else { "Blender closed" });
                }
                if crate::blender::should_time_out(&c.blender, now) {
                    c.blender.payload = None;
                    c.info("Blender stopped sending updates, presence cleared");
                }
                if c.studio.running != running {
                    c.studio.running = running;
                    c.info(if running { "Roblox Studio is running" } else { "Roblox Studio closed" });
                }
                if should_time_out(&c.studio, Instant::now()) {
                    c.studio.activity = None;
                    c.info("Studio stopped sending updates, presence cleared");
                }
                if old_server && !c.studio.old_server_warned {
                    c.studio.old_server_warned = true;
                    c.warn("The old StudioPresence server (studiopresence-win) is running. Close it and delete its startup.vbs from your Startup folder; UltimatePresence replaces it.");
                }
            });
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn body(details: &str) -> String {
        serde_json::json!({
            "updateType": "SET_ACTIVITY",
            "activity": {
                "details": details,
                "state": "Workspace: My Obby",
                "timestamps": { "start": 1_700_000_000 },
                "assets": {
                    "large_image": "serverscript", "large_text": "Editing a SERVER Script",
                    "small_image": "none", "small_image_key": "none"
                }
            }
        })
        .to_string()
    }

    #[test]
    fn sets_activity_like_the_plugin_sends() {
        let mut st = StudioState::default();
        let now = Instant::now();
        let out = apply_request(&mut st, &body("Main (Line 3 / 40)"), now);
        assert_eq!(out, Outcome::Set { connected: true, playtest: None });
        let a = st.activity.unwrap();
        assert_eq!(a.details.as_deref(), Some("Main (Line 3 / 40)"));
        assert_eq!(a.start_timestamp, Some(1_700_000_000_000));
        assert_eq!(a.small_image, None, "'none' placeholder is dropped");
        assert_eq!(a.large_image.as_deref(), Some("serverscript"));
    }

    #[test]
    fn close_and_empty_clear() {
        let mut st = StudioState::default();
        let now = Instant::now();
        apply_request(&mut st, &body("Developing"), now);
        assert_eq!(
            apply_request(&mut st, r#"{"updateType":"CLOSE"}"#, now),
            Outcome::Cleared { had_activity: true, close: true }
        );
        assert!(st.activity.is_none());
        apply_request(&mut st, &body("Developing"), now);
        assert_eq!(apply_request(&mut st, "", now), Outcome::Cleared { had_activity: true, close: false });
        apply_request(&mut st, &body("Developing"), now);
        assert_eq!(apply_request(&mut st, "{not json", now), Outcome::Cleared { had_activity: true, close: false });
    }

    #[test]
    fn testing_holds_for_three_seconds() {
        let mut st = StudioState::default();
        let t0 = Instant::now();
        apply_request(&mut st, &body("Developing"), t0);
        assert_eq!(apply_request(&mut st, &body("Testing"), t0), Outcome::Set { connected: false, playtest: Some(true) });
        assert_eq!(apply_request(&mut st, &body("Main (Line 1 / 2)"), t0 + Duration::from_millis(2900)), Outcome::Ignored);
        assert_eq!(st.activity.as_ref().unwrap().details.as_deref(), Some("Testing"));
        assert_eq!(
            apply_request(&mut st, &body("Main (Line 1 / 2)"), t0 + Duration::from_millis(3100)),
            Outcome::Set { connected: false, playtest: Some(false) }
        );
    }

    #[test]
    fn small_text_falls_back_to_typo_field() {
        let v: Value = serde_json::from_str(
            r#"{"details":"Testing","assets":{"small_image":"playtesticon","small_image_key":"Testing"}}"#,
        )
        .unwrap();
        let a = activity_from_plugin(v.as_object().unwrap());
        assert_eq!(a.small_image.as_deref(), Some("playtesticon"));
        assert_eq!(a.small_text.as_deref(), Some("Testing"));
    }

    #[test]
    fn timeout_requires_silence_and_no_studio() {
        let mut st = StudioState::default();
        let t0 = Instant::now();
        apply_request(&mut st, &body("Developing"), t0);
        let later = t0 + Duration::from_secs(11);
        st.running = true;
        assert!(!should_time_out(&st, later), "idle in Studio keeps presence");
        st.running = false;
        assert!(!should_time_out(&st, t0 + Duration::from_secs(5)));
        assert!(should_time_out(&st, later));
    }

    #[test]
    fn parses_plugin_text() {
        let p = parse(&sample(Kind::Editing));
        assert_eq!(p.kind, Kind::Editing);
        assert_eq!(p.script.as_deref(), Some("DataStoreHandler"));
        assert_eq!((p.line, p.lines), (Some(42), Some(310)));
        assert_eq!(p.script_type, Some("server script"));
        assert_eq!(p.place.as_deref(), Some("Obby Tycoon"));

        let closed = Activity { details: Some("Main (120 lines)".into()), ..Default::default() };
        let p = parse(&closed);
        assert_eq!((p.kind, p.line, p.lines), (Kind::Editing, None, Some(120)));

        let anim = parse(&sample(Kind::Animating));
        assert_eq!((anim.kind, anim.tool.as_deref()), (Kind::Animating, Some("Moon Animator 2")));
        let vfx = parse(&sample(Kind::Vfx));
        assert_eq!((vfx.kind, vfx.tool.as_deref()), (Kind::Vfx, Some("Qwinkle's Part-Icles")));
        assert_eq!(parse(&Activity { details: Some("Something new".into()), ..Default::default() }).kind, Kind::Other);
    }

    #[test]
    fn classic_style_matches_original() {
        let s = SourceSettings::default();
        let raw = sample(Kind::Editing);
        let a = render(&raw, &s, None);
        assert_eq!(a.details, raw.details);
        assert_eq!(a.state, raw.state);
        assert_eq!(a.large_image.as_deref(), Some("serverscript"));
    }

    #[test]
    fn privacy_options() {
        let s = SourceSettings { hide_script: true, hide_place: true, ..Default::default() };
        let a = render(&sample(Kind::Editing), &s, None);
        assert_eq!(a.details.as_deref(), Some("A script (Line 42 / 310)"));
        assert_eq!(a.state.as_deref(), Some("Workspace: a private project"));

        let s = SourceSettings { hide_lines: true, hide_place: true, place_alias: "My secret game".into(), ..Default::default() };
        let a = render(&sample(Kind::Editing), &s, None);
        assert_eq!(a.details.as_deref(), Some("DataStoreHandler"));
        assert_eq!(a.state.as_deref(), Some("Workspace: My secret game"));
    }

    #[test]
    fn styles_and_custom_templates() {
        let s = SourceSettings { style: "detailed".into(), ..Default::default() };
        let a = render(&sample(Kind::Editing), &s, None);
        assert_eq!(a.details.as_deref(), Some("Editing DataStoreHandler"));
        assert_eq!(a.state.as_deref(), Some("Line 42 of 310 · Obby Tycoon"));

        let mut s = SourceSettings { style: "custom".into(), ..Default::default() };
        s.templates.insert("editing".into(), Template { top: "{action} a {type}".into(), bottom: "in {place}".into() });
        let a = render(&sample(Kind::Editing), &s, None);
        assert_eq!(a.details.as_deref(), Some("Editing a server script"));
        assert_eq!(a.state.as_deref(), Some("In Obby Tycoon"));
        // Kinds without a custom template fall back to classic.
        assert_eq!(render(&sample(Kind::Testing), &s, None).details.as_deref(), Some("Testing"));
    }

    #[test]
    fn hidden_places_playtests_and_idle() {
        let mut st = StudioState::default();
        let t0 = Instant::now();
        apply_request(&mut st, &body("Main (Line 1 / 2)"), t0);
        apply_request(&mut st, &body("Testing"), t0);

        let s = SourceSettings { show_testing: false, ..Default::default() };
        let p = present(&st, &s, false);
        assert_eq!(p.activity.unwrap().details.as_deref(), Some("Main (Line 1 / 2)"), "falls back to last edit");

        let s = SourceSettings { hidden_places: vec!["my obby".into()], ..Default::default() };
        assert!(present(&st, &s, false).activity.is_none());

        // Away from the keyboard: Idle with the Zzz icon (own Discord app).
        let away = SourceSettings { application_id: "157263149535120589".into(), ..Default::default() };
        let a = present(&st, &away, true).activity.unwrap();
        assert_eq!(a.details.as_deref(), Some("Idle"));
        assert_eq!(a.state.as_deref(), Some("Workspace: My Obby"));
        assert_eq!(a.large_image.as_deref(), Some("icon_idle"));
        assert!(present(&st, &away, false).activity.unwrap().details.as_deref() != Some("Idle"));
    }

    #[test]
    fn combo_kind() {
        let p = parse(&sample(Kind::Combo));
        assert_eq!(p.kind, Kind::Combo);
        assert_eq!(p.tool.as_deref(), Some("Eclipse Animator + Qwinkle's Part-Icles"));
        let s = SourceSettings::default();
        assert_eq!(render(&sample(Kind::Combo), &s, None).details.as_deref(), Some("Animating & making VFX"));
    }

    #[test]
    fn hide_tools() {
        let s = SourceSettings { hide_tools: true, style: "detailed".into(), ..Default::default() };
        let a = render(&sample(Kind::Combo), &s, None);
        assert!(!a.state.as_deref().unwrap_or("").contains("Eclipse"), "{:?}", a.state);
        assert!(!a.details.as_deref().unwrap_or("").contains("Eclipse"), "{:?}", a.details);
    }

    #[test]
    fn fixed_icons() {
        let own = SourceSettings { application_id: "157263149535120589".into(), ..Default::default() };
        let pairs = [
            (Kind::Developing, "icon_building"),
            (Kind::Testing, "icon_playtest"),
            (Kind::Animating, "icon_animating"),
            (Kind::Vfx, "icon_vfx"),
            (Kind::Combo, "icon_combo"),
            (Kind::Editing, "serverscript"),
        ];
        for (k, img) in pairs {
            let a = render(&sample(k), &own, None);
            assert_eq!(a.large_image.as_deref(), Some(img), "{k:?}");
            assert_eq!(a.small_image.as_deref(), Some("studio"), "badge is always the app's icon");
        }
        // StudioPresence's original app doesn't have the new images.
        let original = SourceSettings { application_id: "1028311936854675458".into(), ..Default::default() };
        assert_eq!(render(&sample(Kind::Animating), &original, None).large_image.as_deref(), Some("studio"));
    }

    #[test]
    fn hover_text_names_this_app() {
        let s = SourceSettings::default();
        for k in KINDS {
            let a = render(&sample(k), &s, None);
            assert_eq!(a.large_text.as_deref(), Some("UltimatePresence by aqua"), "{k:?}");
            assert_eq!(a.small_text.as_deref(), Some("Roblox Studio"), "{k:?}");
        }
        let mut raw = sample(Kind::Developing);
        raw.large_text = Some("StudioPresence by iArxic".into());
        assert_eq!(parse(&raw).tool, None);
    }

    #[test]
    fn timer_modes() {
        let raw = sample(Kind::Testing);
        let s = SourceSettings { timer: "off".into(), ..Default::default() };
        assert_eq!(render(&raw, &s, Some(5)).start_timestamp, None);
        let s = SourceSettings { timer: "activity".into(), ..Default::default() };
        assert_eq!(render(&raw, &s, Some(5)).start_timestamp, Some(5));
    }

    #[test]
    fn chunked_decoding() {
        assert_eq!(decode_chunked(b"4\r\nWiki\r\n5\r\npedia\r\n0\r\n\r\n").unwrap(), b"Wikipedia");
        assert!(decode_chunked(b"4\r\nWi").is_none());
    }
}
