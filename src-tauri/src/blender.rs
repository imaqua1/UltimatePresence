//! Blender source. The UltimatePresence add-on (integrations/blender/ultimate_presence.py) POSTs
//! Blender's raw context to http://127.0.0.1:4455/blender whenever it changes, plus a
//! heartbeat every 10 s. This module decides the category (modeling, sculpting, …) so
//! detection can improve without reinstalling the add-on.
//!
//! Settings reuse the shared per-app shape: `hide_place` / `place_alias` /
//! `hidden_places` apply to the .blend file name, `hide_script` to object names,
//! `hide_lines` to frame numbers, and `show_testing` to renders.

use crate::activity::{Activity, Button};
use crate::settings::SourceSettings;
use crate::studio::Presented;
use crate::template;
use serde::Deserialize;
use std::time::{Duration, Instant};

/// No message for this long (and Blender not running) clears the status.
pub const SILENCE_TIMEOUT: Duration = Duration::from_secs(30);

/// What the add-on sends.
#[derive(Clone, Debug, Default, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase", default)]
pub struct Payload {
    pub version: String,
    /// .blend file name, None while unsaved.
    pub file: Option<String>,
    pub dirty: bool,
    pub workspace: String,
    /// bpy.context.mode, e.g. "OBJECT", "EDIT_MESH", "SCULPT", "POSE".
    pub mode: String,
    pub object: Option<String>,
    pub object_type: Option<String>,
    /// Editors on screen, e.g. "VIEW_3D", "IMAGE_EDITOR:UV", "NODE_EDITOR:GeometryNodeTree".
    pub areas: Vec<String>,
    pub playing: bool,
    pub rendering: bool,
    pub render_frame: Option<i64>,
    pub render_start: Option<i64>,
    pub render_end: Option<i64>,
    pub frame: Option<i64>,
    /// When Blender started (unix seconds).
    pub started: Option<f64>,
    /// Sent when Blender is closing or the add-on is turned off.
    pub closing: bool,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Kind {
    Modeling,
    Sculpting,
    Uv,
    Texturing,
    Shading,
    Animating,
    Rigging,
    Rendering,
    Compositing,
    GeoNodes,
    Drawing,
    Scripting,
    Video,
}

pub const KINDS: [Kind; 13] = [
    Kind::Modeling, Kind::Sculpting, Kind::Uv, Kind::Texturing, Kind::Shading, Kind::Animating, Kind::Rigging,
    Kind::Rendering, Kind::Compositing, Kind::GeoNodes, Kind::Drawing, Kind::Scripting, Kind::Video,
];

impl Kind {
    pub fn key(self) -> &'static str {
        match self {
            Kind::Modeling => "modeling",
            Kind::Sculpting => "sculpting",
            Kind::Uv => "uv",
            Kind::Texturing => "texturing",
            Kind::Shading => "shading",
            Kind::Animating => "animating",
            Kind::Rigging => "rigging",
            Kind::Rendering => "rendering",
            Kind::Compositing => "compositing",
            Kind::GeoNodes => "geonodes",
            Kind::Drawing => "drawing",
            Kind::Scripting => "scripting",
            Kind::Video => "video",
        }
    }

    /// The verb used for {action} and as the badge hover text.
    pub fn action(self) -> &'static str {
        match self {
            Kind::Modeling => "Modeling",
            Kind::Sculpting => "Sculpting",
            Kind::Uv => "UV unwrapping",
            Kind::Texturing => "Texture painting",
            Kind::Shading => "Shading",
            Kind::Animating => "Animating",
            Kind::Rigging => "Rigging",
            Kind::Rendering => "Rendering",
            Kind::Compositing => "Compositing",
            Kind::GeoNodes => "Building geometry nodes",
            Kind::Drawing => "Drawing",
            Kind::Scripting => "Scripting",
            Kind::Video => "Editing video",
        }
    }

    fn has_object(self) -> bool {
        matches!(self, Kind::Modeling | Kind::Sculpting | Kind::Uv | Kind::Texturing | Kind::Shading | Kind::Rigging | Kind::Drawing)
    }
}

/// Decides what you're doing from Blender's context. Most specific first.
pub fn categorize(p: &Payload) -> Kind {
    let ws = p.workspace.to_ascii_lowercase();
    let has_area = |prefix: &str| p.areas.iter().any(|a| a.starts_with(prefix));
    let mode = p.mode.as_str();

    if p.rendering {
        return Kind::Rendering;
    }
    match mode {
        "SCULPT" | "SCULPT_CURVES" => return Kind::Sculpting,
        "PAINT_TEXTURE" | "PAINT_VERTEX" => return Kind::Texturing,
        "PAINT_WEIGHT" | "EDIT_ARMATURE" => return Kind::Rigging,
        m if m.contains("GPENCIL") || m.contains("GREASE_PENCIL") => return Kind::Drawing,
        _ => {}
    }
    if mode == "POSE" {
        return if p.playing || ws.contains("animation") { Kind::Animating } else { Kind::Rigging };
    }
    if ws.contains("uv") || (mode == "EDIT_MESH" && has_area("IMAGE_EDITOR:UV")) {
        return Kind::Uv;
    }
    if ws.contains("texture paint") {
        return Kind::Texturing;
    }
    if ws.contains("sculpt") {
        return Kind::Sculpting;
    }
    if ws.contains("geometry nodes") || (ws.contains("node") && has_area("NODE_EDITOR:GeometryNodeTree")) {
        return Kind::GeoNodes;
    }
    if ws.contains("shading") {
        return Kind::Shading;
    }
    if ws.contains("compositing") {
        return Kind::Compositing;
    }
    if ws.contains("scripting") {
        return Kind::Scripting;
    }
    if ws.contains("video editing") || has_area("SEQUENCE_EDITOR") && !has_area("VIEW_3D") {
        return Kind::Video;
    }
    if ws.contains("2d animation") {
        return Kind::Drawing;
    }
    if ws.contains("animation") || p.playing {
        return Kind::Animating;
    }
    if ws.contains("rendering") {
        return Kind::Rendering;
    }
    Kind::Modeling
}

/// Built-in wording styles; `custom` falls back to classic for kinds left empty.
pub fn preset(style: &str, kind: Kind) -> crate::settings::Template {
    let t = |top: &str, bottom: &str| crate::settings::Template { top: top.into(), bottom: bottom.into() };
    match style {
        "minimal" => t("{action}", "In Blender"),
        "detailed" => match kind {
            Kind::Rendering => t("Rendering[ {file}]", "[Frame {frame} of {frames} · ][{progress}% · ]Blender {version}"),
            Kind::Animating => t("Animating[ {object}]", "[Frame {frame} · ][{file} · ]Blender {version}"),
            k if k.has_object() => t("{action}[ {object}]", "[{file} · ][{mode} · ]Blender {version}"),
            _ => t("{action}", "[{file} · ]Blender {version}"),
        },
        _ => match kind {
            Kind::Rendering => t("Rendering[ {file}]", "[Frame {frame} of {frames}]"),
            k if k.has_object() => t("{action}[ {object}]", "[{file}]"),
            _ => t("{action}", "[{file}]"),
        },
    }
}

fn template_for(s: &SourceSettings, kind: Kind) -> crate::settings::Template {
    if s.style == "custom" {
        if let Some(t) = s.templates.get(kind.key()).filter(|t| !t.top.trim().is_empty() || !t.bottom.trim().is_empty()) {
            return t.clone();
        }
    }
    preset(&s.style, kind)
}

fn human_mode(mode: &str) -> String {
    let m = match mode {
        "OBJECT" => "Object mode",
        "SCULPT" => "Sculpt mode",
        "POSE" => "Pose mode",
        "PAINT_TEXTURE" => "Texture Paint",
        "PAINT_VERTEX" => "Vertex Paint",
        "PAINT_WEIGHT" => "Weight Paint",
        m if m.starts_with("EDIT") => "Edit mode",
        "" => return String::new(),
        other => return other.replace('_', " ").to_lowercase(),
    };
    m.to_owned()
}

fn file_stem(file: &str) -> &str {
    file.strip_suffix(".blend").unwrap_or(file)
}

pub fn is_hidden_file(s: &SourceSettings, file: Option<&str>) -> bool {
    file.is_some_and(|f| {
        let stem = file_stem(f);
        s.hidden_places.iter().any(|h| h.eq_ignore_ascii_case(f) || file_stem(h).eq_ignore_ascii_case(stem))
    })
}

/// Premade large image for a category. Related categories share a picture; the status
/// text still names the exact one.
pub fn state_icon(kind: Kind) -> String {
    match kind {
        Kind::Modeling => "bl_modeling",
        Kind::Sculpting => "bl_sculpting",
        Kind::Uv => "bl_uv",
        Kind::Texturing | Kind::Drawing => "bl_painting",
        Kind::Shading => "bl_shading",
        Kind::Animating | Kind::Rigging => "bl_animating",
        Kind::Rendering => "bl_rendering",
        Kind::Compositing | Kind::GeoNodes => "bl_nodes",
        Kind::Scripting | Kind::Video => "bl_blender",
    }
    .to_owned()
}

/// The small badge is always Blender's own icon.
pub const BADGE: &str = "bl_blender";

pub const HOVER: &str = "UltimatePresence by aqua";

#[cfg(test)]
pub fn render(p: &Payload, s: &SourceSettings, task_since_ms: Option<i64>) -> Activity {
    render_as(p, s, task_since_ms, false)
}

pub fn idle_preset(style: &str) -> crate::settings::Template {
    let t = |top: &str, bottom: &str| crate::settings::Template { top: top.into(), bottom: bottom.into() };
    match style {
        "minimal" => t("Idle", "In Blender"),
        "detailed" => t("Idle", "[{file} · ]Blender {version}"),
        _ => t("Idle", "[{file}]"),
    }
}

/// Renders the context; with `idle` it uses the idle wording and Zzz icon instead.
pub fn render_as(p: &Payload, s: &SourceSettings, task_since_ms: Option<i64>, idle: bool) -> Activity {
    let kind = categorize(p);
    let alias = if s.place_alias.is_empty() { "a private project".to_owned() } else { s.place_alias.clone() };
    let frames = match (p.render_start, p.render_end) {
        (Some(a), Some(b)) if b > a => Some(b - a + 1),
        _ => None,
    };
    let render_pos = match (p.render_frame, p.render_start) {
        (Some(f), Some(a)) => Some(f - a + 1),
        _ => None,
    };
    let var = |name: &str| -> Option<String> {
        match name {
            "action" => Some(kind.action().to_owned()),
            "file" => p.file.as_ref().map(|f| if s.hide_place { alias.clone() } else { file_stem(f).to_owned() }),
            "object" => p.object.clone().filter(|_| !s.hide_script),
            "mode" => Some(human_mode(&p.mode)).filter(|m| !m.is_empty()),
            "workspace" => Some(p.workspace.clone()).filter(|w| !w.is_empty()),
            "version" => Some(p.version.clone()).filter(|v| !v.is_empty()),
            "frame" => {
                if s.hide_lines {
                    None
                } else if kind == Kind::Rendering {
                    render_pos.filter(|_| frames.is_some()).map(|n| n.to_string())
                } else {
                    p.frame.map(|n| n.to_string())
                }
            }
            "frames" => frames.filter(|_| !s.hide_lines).map(|n| n.to_string()),
            "progress" => match (render_pos, frames) {
                (Some(pos), Some(total)) if total > 0 => Some(((pos - 1).max(0) * 100 / total).to_string()),
                _ => None,
            },
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
        template_for(s, kind)
    };
    let top = template::render(&t.top, &var);
    let bottom = template::render(&t.bottom, &var);

    let large_image = Some(if idle { "icon_idle".to_owned() } else { state_icon(kind) });
    let small_image = Some(BADGE.to_owned());

    Activity {
        details: Some(top).filter(|t| !t.is_empty()),
        state: Some(bottom).filter(|t| !t.is_empty()),
        start_timestamp: match s.timer.as_str() {
            "off" => None,
            "activity" => task_since_ms.or_else(|| p.started.map(|t| (t * 1000.0) as i64)),
            _ => p.started.map(|t| (t * 1000.0) as i64),
        },
        // Large image credits this app; the badge names the app you're in.
        large_text: large_image.as_ref().map(|_| HOVER.to_owned()),
        small_text: small_image.as_ref().map(|_| "Blender".to_owned()),
        large_image,
        small_image,
        buttons: s.buttons.iter().map(|b| Button { label: b.label.clone(), url: b.url.clone() }).collect(),
    }
}

#[derive(Debug, Default)]
pub struct BlenderState {
    pub payload: Option<Payload>,
    /// Last payload that wasn't a render (used when renders are hidden).
    last_non_render: Option<Payload>,
    pub last_request: Option<Instant>,
    task_key: String,
    pub task_since_ms: Option<i64>,
    pub running: bool,
}

pub enum Outcome {
    Connected,
    Changed { kind: Kind, was: Option<Kind> },
    Same,
    Closed,
    Invalid,
}

fn now_ms() -> i64 {
    std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map(|d| d.as_millis() as i64).unwrap_or(0)
}

pub fn apply(st: &mut BlenderState, body: &str, now: Instant) -> Outcome {
    let Ok(p) = serde_json::from_str::<Payload>(body) else { return Outcome::Invalid };
    let first = st.payload.is_none();
    st.last_request = Some(now);
    if p.closing {
        st.payload = None;
        return Outcome::Closed;
    }
    let was = st.payload.as_ref().map(categorize);
    let kind = categorize(&p);
    let key = format!("{}|{}", kind.key(), p.object.as_deref().unwrap_or(""));
    if key != st.task_key {
        st.task_key = key;
        st.task_since_ms = Some(now_ms());
    }
    if kind != Kind::Rendering {
        st.last_non_render = Some(p.clone());
    }
    st.payload = Some(p);
    if first {
        Outcome::Connected
    } else if was != Some(kind) {
        Outcome::Changed { kind, was }
    } else {
        Outcome::Same
    }
}

/// Handles one POST from the add-on.
pub fn handle_post(app: &tauri::AppHandle, body: &str) {
    crate::state::update(app, |c| match apply(&mut c.blender, body, Instant::now()) {
        Outcome::Connected => c.info("Blender connected"),
        Outcome::Closed => c.info("Blender closed the session, presence cleared"),
        Outcome::Invalid => c.warn("Blender sent something UltimatePresence couldn't read. Is the add-on up to date?"),
        Outcome::Changed { kind, was } => {
            if was == Some(Kind::Rendering) {
                c.info("Blender render finished");
            } else if kind == Kind::Rendering {
                c.info("Blender render started");
            }
        }
        Outcome::Same => {}
    });
}

pub fn should_time_out(st: &BlenderState, now: Instant) -> bool {
    st.payload.is_some() && (!st.running || st.last_request.map_or(true, |t| now.duration_since(t) >= SILENCE_TIMEOUT))
        && st.last_request.map_or(true, |t| now.duration_since(t) >= Duration::from_secs(4))
}

/// `away`: no keyboard or mouse input for IDLE_AFTER. A render in progress never counts as idle.
pub fn present(st: &BlenderState, s: &SourceSettings, away: bool) -> Presented {
    let none = |reason: Option<String>| Presented { activity: None, reason };
    let Some(mut p) = st.payload.as_ref() else { return none(None) };
    if s.application_id.is_empty() {
        return none(Some("Needs a Discord application (see the Discord app tab)".into()));
    }
    if categorize(p) == Kind::Rendering && !s.show_testing {
        match st.last_non_render.as_ref() {
            Some(prev) => p = prev,
            None => return none(Some("Renders are hidden".into())),
        }
    }
    let idle = away && categorize(p) != Kind::Rendering;
    if is_hidden_file(s, p.file.as_deref()) {
        return none(Some("Hidden while this file is open".into()));
    }
    Presented { activity: Some(render_as(p, s, st.task_since_ms, idle)), reason: None }
}

/// Sample context for previews when Blender isn't connected.
pub fn sample(kind: Kind) -> Payload {
    let mut p = Payload {
        version: "4.2.3".into(),
        file: Some("spaceship_v3.blend".into()),
        workspace: "Layout".into(),
        mode: "OBJECT".into(),
        object: Some("Hull".into()),
        object_type: Some("MESH".into()),
        areas: vec!["VIEW_3D".into(), "OUTLINER".into(), "PROPERTIES".into()],
        frame: Some(48),
        started: Some((now_ms() as f64 / 1000.0) - 48.0 * 60.0),
        ..Default::default()
    };
    match kind {
        Kind::Modeling => p.mode = "EDIT_MESH".into(),
        Kind::Sculpting => p.mode = "SCULPT".into(),
        Kind::Uv => {
            p.workspace = "UV Editing".into();
            p.mode = "EDIT_MESH".into();
        }
        Kind::Texturing => p.mode = "PAINT_TEXTURE".into(),
        Kind::Shading => p.workspace = "Shading".into(),
        Kind::Animating => {
            p.workspace = "Animation".into();
            p.object = Some("Pilot".into());
        }
        Kind::Rigging => {
            p.mode = "POSE".into();
            p.object = Some("Pilot_Rig".into());
        }
        Kind::Rendering => {
            p.rendering = true;
            p.render_start = Some(1);
            p.render_end = Some(250);
            p.render_frame = Some(120);
        }
        Kind::Compositing => p.workspace = "Compositing".into(),
        Kind::GeoNodes => p.workspace = "Geometry Nodes".into(),
        Kind::Drawing => {
            p.mode = "PAINT_GREASE_PENCIL".into();
            p.object = Some("Storyboard".into());
        }
        Kind::Scripting => p.workspace = "Scripting".into(),
        Kind::Video => {
            p.workspace = "Video Editing".into();
            p.areas = vec!["SEQUENCE_EDITOR".into()];
        }
    }
    p
}

#[cfg(test)]
mod tests {
    use super::*;

    fn own() -> SourceSettings {
        SourceSettings { application_id: "157263149535120589".into(), ..Default::default() }
    }

    #[test]
    fn every_sample_lands_in_its_category() {
        for k in KINDS {
            assert_eq!(categorize(&sample(k)), k, "{k:?}");
        }
        // Plain object mode in Layout is modeling.
        let mut p = sample(Kind::Modeling);
        p.mode = "OBJECT".into();
        assert_eq!(categorize(&p), Kind::Modeling);
        // Pose mode while the timeline plays is animating.
        let mut p = sample(Kind::Rigging);
        p.playing = true;
        assert_eq!(categorize(&p), Kind::Animating);
    }

    /// Payloads captured from Blender 5.3's default workspaces.
    #[test]
    fn real_blender_5_3_workspaces() {
        let cases = [
            (r#"{"workspace":"Sculpting","mode":"SCULPT","object":"Cube","areas":["PROPERTIES","OUTLINER","VIEW_3D"]}"#, Kind::Sculpting),
            (r#"{"workspace":"UV Editing","mode":"EDIT_MESH","object":"Cube","areas":["PROPERTIES","OUTLINER","IMAGE_EDITOR:UV","VIEW_3D"]}"#, Kind::Uv),
            (r#"{"workspace":"Shading","mode":"OBJECT","object":"Cube","areas":["PROPERTIES","OUTLINER","IMAGE_EDITOR:VIEW","NODE_EDITOR:ShaderNodeTree","FILE_BROWSER","VIEW_3D"]}"#, Kind::Shading),
            (r#"{"workspace":"Layout","mode":"OBJECT","object":"Cube","areas":["PROPERTIES","OUTLINER","DOPESHEET_EDITOR","VIEW_3D"]}"#, Kind::Modeling),
        ];
        for (json, kind) in cases {
            let p: Payload = serde_json::from_str(json).unwrap();
            assert_eq!(categorize(&p), kind, "{json}");
        }
    }

    #[test]
    fn classic_wording_and_icons() {
        let a = render(&sample(Kind::Sculpting), &own(), None);
        assert_eq!(a.details.as_deref(), Some("Sculpting Hull"));
        assert_eq!(a.state.as_deref(), Some("spaceship_v3"));
        assert_eq!(a.large_image.as_deref(), Some("bl_sculpting"));
        assert_eq!(a.small_image.as_deref(), Some("bl_blender"));
        assert_eq!(a.large_text.as_deref(), Some("UltimatePresence by aqua"));
        assert_eq!(a.small_text.as_deref(), Some("Blender"));

        let r = render(&sample(Kind::Rendering), &own(), None);
        assert_eq!(r.details.as_deref(), Some("Rendering spaceship_v3"));
        assert_eq!(r.state.as_deref(), Some("Frame 120 of 250"));
        let d = render(&sample(Kind::Rendering), &SourceSettings { style: "detailed".into(), ..own() }, None);
        assert_eq!(d.state.as_deref(), Some("Frame 120 of 250 · 47% · Blender 4.2.3"));
    }

    #[test]
    fn privacy_and_hiding() {
        let s = SourceSettings { hide_place: true, hide_script: true, ..own() };
        let a = render(&sample(Kind::Modeling), &s, None);
        assert_eq!(a.details.as_deref(), Some("Modeling"));
        assert_eq!(a.state.as_deref(), Some("A private project"));

        let mut st = BlenderState { running: true, ..Default::default() };
        let t0 = Instant::now();
        apply(&mut st, &serde_json::to_string(&serde_json::json!({"file":"secret.blend","mode":"SCULPT","version":"4.2"})).unwrap(), t0);
        let hidden = SourceSettings { hidden_places: vec!["secret".into()], ..own() };
        assert!(present(&st, &hidden, false).activity.is_none());
        assert!(present(&st, &own(), false).activity.is_some());
        // Without a Discord application nothing is shown (it would appear under another app's name).
        assert!(present(&st, &SourceSettings::default(), false).activity.is_none());
    }

    #[test]
    fn idle_rendering_and_closing() {
        let mut st = BlenderState { running: true, ..Default::default() };
        let t0 = Instant::now();
        let body = r#"{"file":"a.blend","mode":"OBJECT","version":"4.2"}"#;
        assert!(matches!(apply(&mut st, body, t0), Outcome::Connected));
        assert!(matches!(apply(&mut st, body, t0 + Duration::from_secs(600)), Outcome::Same));
        // Away from the keyboard: Idle, with the Zzz icon and the Blender badge.
        let a = present(&st, &own(), true).activity.unwrap();
        assert_eq!((a.details.as_deref(), a.large_image.as_deref()), (Some("Idle"), Some("icon_idle")));
        assert_eq!(a.small_image.as_deref(), Some("bl_blender"));
        // A long render without input isn't idle.
        apply(&mut st, r#"{"file":"a.blend","mode":"OBJECT","version":"4.2","rendering":true}"#, t0);
        assert_eq!(present(&st, &own(), true).activity.unwrap().details.as_deref().map(|d| d.starts_with("Rendering")), Some(true));
        assert!(matches!(apply(&mut st, r#"{"closing":true}"#, t0), Outcome::Closed));
        assert!(st.payload.is_none());
    }
}
