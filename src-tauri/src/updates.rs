//! Automatic updates from GitHub Releases (signed; see `plugins.updater` in
//! tauri.conf.json). Checks shortly after start and every 6 hours; installing is one
//! click in the window.

use crate::state::{update, AppState};
use std::time::Duration;
use tauri::{AppHandle, Manager};
use tauri_plugin_updater::UpdaterExt;

pub fn start(app: &AppHandle) {
    let app = app.clone();
    tauri::async_runtime::spawn(async move {
        tokio::time::sleep(Duration::from_secs(8)).await;
        loop {
            let _ = check(&app).await;
            tokio::time::sleep(Duration::from_secs(6 * 60 * 60)).await;
        }
    });
}

/// Asks GitHub for a newer version. Returns it, if any.
pub async fn check(app: &AppHandle) -> Result<Option<String>, String> {
    let found = app
        .updater()
        .map_err(|e| e.to_string())?
        .check()
        .await
        .map_err(|e| format!("Couldn't check for updates: {e}"))?
        .map(|u| u.version.clone());
    update(app, |c| {
        if c.update_available != found {
            if let Some(v) = &found {
                c.info(format!("Update available: UltimatePresence {v}"));
            }
            c.update_available = found.clone();
        }
    });
    Ok(found)
}

#[tauri::command]
pub async fn check_for_update(app: AppHandle) -> Result<Option<String>, String> {
    check(&app).await
}

/// Downloads and installs the newest version, then restarts the app.
#[tauri::command]
pub async fn install_update(app: AppHandle) -> Result<(), String> {
    let upd = app
        .updater()
        .map_err(|e| e.to_string())?
        .check()
        .await
        .map_err(|e| format!("Couldn't check for updates: {e}"))?
        .ok_or("You're already on the newest version.")?;
    update(&app, |c| c.info(format!("Installing UltimatePresence {}…", upd.version)));
    let bytes = upd.download(|_, _| {}, || {}).await.map_err(|e| format!("Download failed: {e}"))?;
    // Clear the Discord status before the installer replaces the app.
    let discord = app.state::<AppState>().discord.clone();
    discord.shutdown(Duration::from_millis(1500)).await;
    upd.install(bytes).map_err(|e| format!("Install failed: {e}"))?;
    app.restart();
}
