//! Central app state. Every change goes through `update`, which picks the active
//! source, pushes its activity (and Discord application) to the IPC worker,
//! refreshes the tray and notifies the window.

use crate::activity::Activity;
use crate::discord::{DiscordEvent, DiscordHandle, DiscordStatus};
use crate::settings::{Settings, SourceSettings};
use crate::blender::{self, BlenderState};
use crate::sources::{self, BLENDER, ROBLOX_STUDIO};
use crate::studio::{self, StudioState};
use std::collections::HashMap;
use serde::Serialize;
use std::collections::VecDeque;
use std::path::PathBuf;
use std::sync::{Arc, Mutex, MutexGuard, PoisonError};
use std::time::{Instant, SystemTime, UNIX_EPOCH};
use tauri::{AppHandle, Emitter, Manager};

const MAX_LOG: usize = 300;
const MAX_HISTORY: usize = 40;

fn now_ms() -> u64 {
    SystemTime::now().duration_since(UNIX_EPOCH).map(|d| d.as_millis() as u64).unwrap_or(0)
}

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LogEntry {
    pub id: u64,
    pub time_ms: u64,
    pub level: &'static str,
    pub message: String,
}

/// One step in this session's timeline ("Editing Main", "Stopped sharing", …).
#[derive(Clone, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct HistoryItem {
    pub time_ms: u64,
    pub source: Option<&'static str>,
    pub title: String,
    pub subtitle: Option<String>,
}

pub struct Core {
    pub settings: Settings,
    settings_path: PathBuf,
    pub studio: StudioState,
    pub blender: BlenderState,
    /// When each app's window was last in front. The focused app wins.
    pub focus: HashMap<&'static str, Instant>,
    /// The app whose window is in front right now, if it's one we know.
    pub focused: Option<&'static str>,
    /// A newer UltimatePresence version on GitHub, if any.
    pub update_available: Option<String>,
    pub discord: DiscordStatus,
    pub autostart: bool,
    /// Discord application the IPC worker is using.
    client_id: String,
    active_source: Option<&'static str>,
    shown: Option<Activity>,
    history: VecDeque<HistoryItem>,
    history_key: String,
    started_ms: u64,
    log: VecDeque<LogEntry>,
    pending_log: Vec<LogEntry>,
    next_log_id: u64,
}

/// One app as the window and tray see it.
#[derive(Clone, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct SourceView {
    pub id: &'static str,
    pub name: &'static str,
    pub available: bool,
    pub default_application_id: &'static str,
    pub settings: SourceSettings,
    /// "sharing" | "waiting" | "idle" | "held" | "disabled" | "error" | "soon"
    pub status: &'static str,
    pub status_text: String,
    /// Longer explanation (why the port is busy, why the status is held back).
    pub detail: Option<String>,
    /// This app's activity after its settings (shown even while paused, for previews).
    pub activity: Option<Activity>,
}

#[derive(Clone, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Snapshot {
    pub version: &'static str,
    /// "connected" | "disconnected" | "rejected"
    pub discord_status: &'static str,
    pub discord_user: Option<String>,
    /// Your Discord profile picture.
    pub discord_avatar: Option<String>,
    pub discord_message: Option<String>,
    /// Application ID the Discord connection currently uses.
    pub discord_application_id: String,
    pub paused: bool,
    pub autostart: bool,
    pub open_window_on_login: bool,
    /// The app whose activity is on Discord right now.
    pub active_source: Option<&'static str>,
    /// What Discord is asked to show.
    pub shown: Option<Activity>,
    pub sources: Vec<SourceView>,
    /// Newest first.
    pub history: Vec<HistoryItem>,
    pub started_ms: u64,
    /// The app whose window is in front right now (if it's one we know).
    pub focused: Option<&'static str>,
    /// A newer UltimatePresence version on GitHub, if any.
    pub update_available: Option<String>,
}

impl Snapshot {
    pub fn source(&self, id: &str) -> Option<&SourceView> {
        self.sources.iter().find(|s| s.id == id)
    }
}

impl Core {
    pub fn new(settings: Settings, settings_path: PathBuf, autostart: bool) -> Self {
        let client_id = settings.source(ROBLOX_STUDIO).application_id;
        Self {
            settings,
            settings_path,
            studio: StudioState::default(),
            blender: BlenderState::default(),
            focus: HashMap::new(),
            focused: None,
            update_available: None,
            discord: DiscordStatus::Disconnected,
            autostart,
            client_id,
            active_source: None,
            shown: None,
            history: VecDeque::new(),
            history_key: String::new(),
            started_ms: now_ms(),
            log: VecDeque::new(),
            pending_log: Vec::new(),
            next_log_id: 1,
        }
    }

    pub fn client_id(&self) -> &str {
        &self.client_id
    }

    fn push_log(&mut self, level: &'static str, message: String) {
        #[cfg(debug_assertions)]
        eprintln!("[{level}] {message}");
        let entry = LogEntry { id: self.next_log_id, time_ms: now_ms(), level, message };
        self.next_log_id += 1;
        self.log.push_back(entry.clone());
        while self.log.len() > MAX_LOG {
            self.log.pop_front();
        }
        self.pending_log.push(entry);
    }

    pub fn info(&mut self, msg: impl Into<String>) {
        self.push_log("info", msg.into());
    }
    pub fn warn(&mut self, msg: impl Into<String>) {
        self.push_log("warn", msg.into());
    }
    pub fn error(&mut self, msg: impl Into<String>) {
        self.push_log("error", msg.into());
    }

    pub fn log_entries(&self) -> Vec<LogEntry> {
        self.log.iter().cloned().collect()
    }

    pub fn clear_log(&mut self) {
        self.log.clear();
    }

    pub fn save_settings(&mut self) {
        if let Err(e) = self.settings.save(&self.settings_path) {
            self.warn(format!("Couldn't save settings: {e}"));
        }
    }

    pub fn settings_dir(&self) -> Option<PathBuf> {
        self.settings_path.parent().map(PathBuf::from)
    }

    /// A source's activity after its own settings, ignoring pause.
    fn presented(&self, id: &str) -> studio::Presented {
        let s = self.settings.source(id);
        if !s.enabled {
            return studio::Presented { activity: None, reason: None };
        }
        match id {
            ROBLOX_STUDIO => studio::present(&self.studio, &s, Instant::now()),
            BLENDER => blender::present(&self.blender, &s, Instant::now()),
            // Planned sources have no runtime yet.
            _ => studio::Presented { activity: None, reason: None },
        }
    }

    /// Sources that currently have something to show, with when they last updated.
    fn candidates(&self) -> Vec<(&'static str, Activity, Instant)> {
        sources::CATALOG
            .iter()
            .filter(|info| info.available)
            .filter_map(|info| {
                let activity = self.presented(info.id).activity?;
                let updated = match info.id {
                    ROBLOX_STUDIO => self.studio.last_request?,
                    BLENDER => self.blender.last_request?,
                    _ => return None,
                };
                Some((info.id, activity, updated))
            })
            .collect()
    }

    /// The most recently updated source wins.
    fn active(&self) -> Option<(&'static str, Activity)> {
        if self.settings.paused {
            return None;
        }
        // The app you're focused on wins; when you switch to something else (Discord,
        // a browser), the one you used last stays. Ties fall back to the latest update.
        self.candidates()
            .into_iter()
            .max_by_key(|(id, _, updated)| (self.focus.get(id).copied(), *updated))
            .map(|(id, a, _)| (id, a))
    }

    fn source_view(&self, info: &'static sources::SourceInfo) -> SourceView {
        let s = self.settings.source(info.id);
        let presented = self.presented(info.id);
        let mut detail = None;
        let (status, status_text) = if !info.available {
            ("soon", "Coming soon".to_owned())
        } else if !s.enabled {
            ("disabled", "Turned off".to_owned())
        } else {
            match info.id {
                ROBLOX_STUDIO => {
                    if !self.studio.server_ok {
                        detail = self.studio.server_error.clone();
                        ("error", "Port 4455 is busy".to_owned())
                    } else if presented.activity.is_some() {
                        ("sharing", "Connected".to_owned())
                    } else if let Some(reason) = presented.reason {
                        detail = Some(reason.clone());
                        ("held", reason)
                    } else if self.studio.running {
                        ("waiting", "Open, waiting for the plugin".to_owned())
                    } else {
                        ("idle", "Not running".to_owned())
                    }
                }
                BLENDER => {
                    if !self.studio.server_ok {
                        detail = self.studio.server_error.clone();
                        ("error", "Port 4455 is busy".to_owned())
                    } else if presented.activity.is_some() {
                        ("sharing", "Connected".to_owned())
                    } else if let Some(reason) = presented.reason {
                        detail = Some(reason.clone());
                        ("held", reason)
                    } else if self.blender.running {
                        ("waiting", "Open, waiting for the add-on".to_owned())
                    } else {
                        ("idle", "Not running".to_owned())
                    }
                }
                _ => ("idle", "Not running".to_owned()),
            }
        };
        SourceView {
            id: info.id,
            name: info.name,
            available: info.available,
            default_application_id: info.default_application_id,
            settings: s,
            status,
            status_text,
            detail,
            activity: presented.activity,
        }
    }

    pub fn snapshot(&self) -> Snapshot {
        let (discord_status, discord_user, discord_avatar, discord_message) = match &self.discord {
            DiscordStatus::Connected { user, avatar } => ("connected", Some(user.clone()), avatar.clone(), None),
            DiscordStatus::Disconnected => ("disconnected", None, None, None),
            DiscordStatus::Rejected(m) => ("rejected", None, None, Some(m.clone())),
        };
        Snapshot {
            version: env!("CARGO_PKG_VERSION"),
            discord_status,
            discord_user,
            discord_avatar,
            discord_message,
            discord_application_id: self.client_id.clone(),
            paused: self.settings.paused,
            autostart: self.autostart,
            open_window_on_login: self.settings.open_window_on_login,
            active_source: self.active_source,
            shown: self.shown.clone(),
            sources: sources::CATALOG.iter().map(|info| self.source_view(info)).collect(),
            history: self.history.iter().cloned().collect(),
            started_ms: self.started_ms,
            update_available: self.update_available.clone(),
            focused: self.focused,
        }
    }

    /// True when time-based rules (idle hiding) changed what should be shown.
    pub fn is_stale(&self) -> bool {
        self.active().map(|(_, a)| a) != self.shown
    }

    /// Adds a timeline step when what's shown meaningfully changes (not every line move).
    fn record_history(&mut self, source: Option<&'static str>, shown: Option<&Activity>) {
        let (key, title, subtitle) = match shown {
            Some(a) => {
                let title = a.details.clone().unwrap_or_else(|| "Sharing".into());
                // "Main (Line 4 / 80)" and "Main (Line 5 / 80)" are the same step.
                let base = title.split(" (Line ").next().unwrap_or(&title).to_owned();
                let key = format!("{}|{}|{}", source.unwrap_or(""), base, a.state.as_deref().unwrap_or(""));
                (key, base, a.state.clone())
            }
            None => ("".to_owned(), "Stopped sharing".to_owned(), None),
        };
        if key == self.history_key || (shown.is_none() && self.history.is_empty()) {
            return;
        }
        self.history_key = key;
        self.history.push_front(HistoryItem { time_ms: now_ms(), source, title, subtitle });
        self.history.truncate(MAX_HISTORY);
    }
}

pub struct AppState {
    pub core: Mutex<Core>,
    pub discord: Arc<DiscordHandle>,
}

pub fn lock_core(app: &AppHandle) -> MutexGuard<'_, Core> {
    app.state::<AppState>().inner().core.lock().unwrap_or_else(PoisonError::into_inner)
}

/// Apply a change, then sync Discord, the tray and the window.
pub fn update<R>(app: &AppHandle, f: impl FnOnce(&mut Core) -> R) -> R {
    let state = app.state::<AppState>();
    let (result, snap, logs) = {
        let mut c = state.core.lock().unwrap_or_else(PoisonError::into_inner);
        let result = f(&mut c);

        let active = c.active();
        // Use the active app's Discord application; with nothing active, the first
        // enabled live app's (so a newly entered ID gets checked straight away).
        let wanted_client = active
            .as_ref()
            .map(|(id, _)| *id)
            .or_else(|| {
                sources::CATALOG.iter().find(|i| i.available && c.settings.source(i.id).enabled).map(|i| i.id)
            })
            .map(|id| c.settings.source(id).application_id)
            .filter(|id| !id.is_empty())
            .unwrap_or_else(|| c.client_id.clone());
        if wanted_client != c.client_id {
            c.client_id = wanted_client.clone();
            state.discord.set_client_id(wanted_client);
        }

        let (active_id, eff) = match active {
            Some((id, a)) => (Some(id), Some(a)),
            None => (None, None),
        };
        if active_id != c.active_source {
            if let Some(id) = active_id {
                let name = sources::find(id).map_or(id, |i| i.name);
                c.info(format!("Now showing {name} on Discord"));
            }
            c.active_source = active_id;
        }
        if eff != c.shown {
            c.record_history(active_id, eff.as_ref());
            c.shown = eff.clone();
            state.discord.set_activity(eff);
        }
        (result, c.snapshot(), std::mem::take(&mut c.pending_log))
    };
    crate::tray::refresh(app, &snap);
    let _ = app.emit("state", &snap);
    for entry in logs {
        let _ = app.emit("log", &entry);
    }
    result
}

pub fn on_discord_event(app: &AppHandle, ev: DiscordEvent) {
    update(app, |c| match ev {
        DiscordEvent::Status(s) => c.discord = s,
        DiscordEvent::Log(level, msg) => c.push_log(level, msg),
    });
}
