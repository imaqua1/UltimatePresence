//! Actions shared by the tray menu and the window (exposed as Tauri commands).

use crate::activity::Activity;
use crate::settings::{merge, Settings, SourceSettings, Template};
use crate::sources;
use crate::state::{lock_core, update, LogEntry, Snapshot};
use crate::blender;
use crate::studio::{self, Kind, KINDS};
use serde::Serialize;
use std::collections::BTreeMap;
use tauri::{AppHandle, Manager};
use tauri_plugin_autostart::ManagerExt as _;
use tauri_plugin_opener::OpenerExt as _;

fn live_source(id: &str) -> Result<&'static sources::SourceInfo, String> {
    match sources::find(id) {
        Some(info) if info.available => Ok(info),
        Some(info) => Err(format!("{} support is coming soon.", info.name)),
        None => Err(format!("Unknown app: {id}")),
    }
}

pub fn apply_paused(app: &AppHandle, paused: bool) {
    update(app, |c| {
        if c.settings.paused != paused {
            c.settings.paused = paused;
            c.save_settings();
            c.info(if paused { "Sharing paused, presence cleared" } else { "Sharing resumed" });
        }
    });
}

pub fn apply_autostart(app: &AppHandle, enabled: bool) -> Result<bool, String> {
    let launcher = app.autolaunch();
    let result = if enabled { launcher.enable() } else { launcher.disable() };
    let actual = launcher.is_enabled().unwrap_or(false);
    update(app, |c| {
        c.autostart = actual;
        match &result {
            Ok(()) => c.info(if actual { "Will start with your computer" } else { "Won't start with your computer" }),
            Err(e) => c.error(format!("Couldn't change start-up setting: {e}")),
        }
    });
    result.map(|_| actual).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn get_state(app: AppHandle) -> Snapshot {
    lock_core(&app).snapshot()
}

#[tauri::command]
pub fn get_log(app: AppHandle) -> Vec<LogEntry> {
    lock_core(&app).log_entries()
}

#[tauri::command]
pub fn clear_log(app: AppHandle) {
    lock_core(&app).clear_log();
}

#[tauri::command]
pub fn set_paused(app: AppHandle, paused: bool) {
    apply_paused(&app, paused);
}

#[tauri::command]
pub fn set_autostart(app: AppHandle, enabled: bool) -> Result<bool, String> {
    apply_autostart(&app, enabled)
}

/// App-wide settings other than pause / autostart (e.g. `{ "openWindowOnLogin": true }`).
#[tauri::command]
pub fn update_settings(app: AppHandle, patch: serde_json::Value) -> Result<(), String> {
    let allowed = ["openWindowOnLogin"];
    if patch.as_object().is_some_and(|o| o.keys().any(|k| !allowed.contains(&k.as_str()))) {
        return Err("That setting can't be changed here.".into());
    }
    update(&app, |c| {
        let next: Settings = merge(&c.settings, &patch)?;
        c.settings.open_window_on_login = next.open_window_on_login;
        c.save_settings();
        Ok(())
    })
}

/// Changes one app's settings with a partial object, e.g. `{ "hidePlace": true }`.
#[tauri::command]
pub fn update_source_settings(app: AppHandle, source: String, patch: serde_json::Value) -> Result<(), String> {
    let info = live_source(&source)?;
    update(&app, |c| {
        let current = c.settings.source(info.id);
        let mut next: SourceSettings = merge(&current, &patch)?;
        // The Discord application is built in.
        next.application_id = info.default_application_id.to_owned();
        next.validate()?;
        if next == current {
            return Ok(());
        }
        if next.enabled != current.enabled {
            c.info(format!("{} {}", info.name, if next.enabled { "turned on" } else { "turned off" }));
        }
        if next.application_id != current.application_id {
            let note = if next.application_id == info.default_application_id { " (default)" } else { "" };
            c.info(format!("{}: Discord Application ID set to {}{note}", info.name, next.application_id));
        }
        *c.settings.source_mut(info.id) = next;
        c.save_settings();
        Ok(())
    })
}

/// Restores one app's settings to defaults (keeps the Application ID).
#[tauri::command]
pub fn reset_source_settings(app: AppHandle, source: String) -> Result<(), String> {
    let info = live_source(&source)?;
    update(&app, |c| {
        let s = c.settings.source_mut(info.id);
        *s = SourceSettings { application_id: s.application_id.clone(), ..SourceSettings::default() };
        c.save_settings();
        c.info(format!("{}: settings reset to defaults", info.name));
    });
    Ok(())
}

/// Renders a preview with the current (optionally patched) settings. Uses the live
/// Studio activity when there is one, otherwise sample data for `kind`.
#[tauri::command]
pub fn render_preview(
    app: AppHandle,
    source: String,
    patch: Option<serde_json::Value>,
    kind: Option<String>,
) -> Result<Activity, String> {
    let info = live_source(&source)?;
    let c = lock_core(&app);
    let mut s = c.settings.source(info.id);
    if let Some(p) = patch {
        s = merge(&s, &p)?;
        let _ = s.validate();
    }
    let idle = kind.as_deref() == Some("idle");
    match info.id {
        sources::BLENDER => {
            let sample_kind = kind.as_deref().and_then(|k| blender::KINDS.iter().copied().find(|x| x.key() == k));
            let raw = match (sample_kind, c.blender.payload.clone()) {
                (None, Some(live)) if !idle => live,
                (None, Some(live)) => live,
                (k, _) => blender::sample(k.unwrap_or(blender::Kind::Modeling)),
            };
            Ok(blender::render_as(&raw, &s, c.blender.task_since_ms, idle))
        }
        _ => {
            let sample_kind = kind.as_deref().and_then(|k| KINDS.iter().copied().find(|x| x.key() == k));
            let raw = match (sample_kind, c.studio.activity.clone()) {
                (None, Some(live)) => live,
                (k, _) => studio::sample(k.unwrap_or(Kind::Editing)),
            };
            Ok(studio::render_as(&raw, &s, c.studio.task_since_ms, idle))
        }
    }
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Presets {
    /// style -> kind -> template
    styles: BTreeMap<&'static str, BTreeMap<&'static str, Template>>,
    kinds: Vec<&'static str>,
}

/// Built-in wording for one app's states (including "idle").
#[tauri::command]
pub fn get_presets(source: Option<String>) -> Presets {
    let styles = ["classic", "minimal", "detailed"];
    match source.as_deref() {
        Some(sources::BLENDER) => Presets {
            styles: styles
                .into_iter()
                .map(|style| {
                    let mut m: BTreeMap<_, _> = blender::KINDS.iter().map(|k| (k.key(), blender::preset(style, *k))).collect();
                    m.insert("idle", blender::idle_preset(style));
                    (style, m)
                })
                .collect(),
            kinds: blender::KINDS.iter().map(|k| k.key()).chain(["idle"]).collect(),
        },
        _ => Presets {
            styles: styles
                .into_iter()
                .map(|style| {
                    let mut m: BTreeMap<_, _> = KINDS.iter().map(|k| (k.key(), studio::preset(style, *k))).collect();
                    m.insert("idle", studio::idle_preset(style));
                    (style, m)
                })
                .collect(),
            kinds: KINDS.iter().map(|k| k.key()).chain(["idle"]).collect(),
        },
    }
}

#[tauri::command]
pub fn get_integrations(app: AppHandle) -> crate::integrations::Integrations {
    crate::integrations::status(&app)
}

#[tauri::command]
pub fn install_integration(app: AppHandle, which: String) -> Result<String, String> {
    let result = match which.as_str() {
        "studio-plugin" => crate::integrations::install_studio_plugin(&app),
        "blender" => crate::integrations::install_blender(&app),
        _ => Err("Unknown integration".into()),
    };
    update(&app, |c| match &result {
        Ok(msg) => c.info(msg.clone()),
        Err(e) => c.warn(e.clone()),
    });
    result
}

/// Opens one of a fixed set of destinations (no arbitrary URLs from the page).
#[tauri::command]
pub fn open_external(app: AppHandle, target: String) -> Result<(), String> {
    let opener = app.opener();
    match target.as_str() {
        "settings-folder" => {
            let dir = lock_core(&app).settings_dir().ok_or("no settings folder")?;
            let _ = std::fs::create_dir_all(&dir);
            opener.open_path(dir.to_string_lossy(), None::<&str>)
        }
        _ => return Err("unknown target".into()),
    }
    .map_err(|e| e.to_string())
}

/// Called by the page once it has rendered, so the window never flashes blank.
#[tauri::command]
pub fn ui_ready(window: tauri::WebviewWindow) {
    let _ = window.show();
    let _ = window.set_focus();
}
