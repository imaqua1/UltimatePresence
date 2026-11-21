// No console window on Windows (debug builds keep it for logs).
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod activity;
mod blender;
mod commands;
mod discord;
mod integrations;
mod platform;
mod settings;
mod sources;
mod state;
mod studio;
mod template;
mod tray;
mod updates;

use settings::Settings;
use state::{update, AppState, Core};
use std::sync::{Arc, Mutex};
use std::time::Duration;
use tauri::{AppHandle, Manager, WebviewUrl, WebviewWindowBuilder};
use tauri_plugin_autostart::{MacosLauncher, ManagerExt as _};

const MINIMIZED_ARG: &str = "--minimized";

/// Shows the settings window, creating it on demand. The window is destroyed when
/// closed so the WebView doesn't use memory while the app sits in the tray.
pub fn show_window(app: &AppHandle) {
    if let Some(w) = app.get_webview_window("main") {
        let _ = w.unminimize();
        let _ = w.show();
        let _ = w.set_focus();
        return;
    }
    let app = app.clone();
    // Build off the main thread; building inside some event callbacks can deadlock on Windows.
    tauri::async_runtime::spawn(async move {
        let built = WebviewWindowBuilder::new(&app, "main", WebviewUrl::App("index.html".into()))
            .title("UltimatePresence")
            .inner_size(1180.0, 780.0)
            .min_inner_size(760.0, 560.0)
            .center()
            .visible(false)
            .build();
        if let Ok(w) = built {
            // Fallback in case the page never reports ready.
            tokio::time::sleep(Duration::from_millis(1500)).await;
            if !w.is_visible().unwrap_or(true) {
                let _ = w.show();
            }
        }
    });
}

/// Clears the presence, then exits.
pub fn quit(app: &AppHandle) {
    let app = app.clone();
    tauri::async_runtime::spawn(async move {
        update(&app, |c| c.info("Quitting, clearing presence"));
        let discord = app.state::<AppState>().discord.clone();
        discord.shutdown(Duration::from_millis(1500)).await;
        app.exit(0);
    });
}

fn main() {
    tauri::Builder::default()
        .plugin(tauri_plugin_single_instance::init(|app, args, _cwd| {
            if !args.iter().any(|a| a == MINIMIZED_ARG) {
                show_window(app);
            }
        }))
        .plugin(tauri_plugin_autostart::init(MacosLauncher::LaunchAgent, Some(vec![MINIMIZED_ARG])))
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_updater::Builder::new().build())
        .plugin(tauri_plugin_process::init())
        .invoke_handler(tauri::generate_handler![
            commands::get_state,
            commands::get_log,
            commands::clear_log,
            commands::set_paused,
            commands::set_autostart,
            commands::update_settings,
            commands::update_source_settings,
            commands::reset_source_settings,
            commands::render_preview,
            commands::get_presets,
            commands::get_integrations,
            commands::install_integration,
            commands::open_external,
            updates::check_for_update,
            updates::install_update,
            commands::ui_ready,
        ])
        .setup(|app| {
            let handle = app.handle().clone();
            let settings_path = app.path().app_config_dir()?.join("settings.json");
            let settings = Settings::load(&settings_path);
            let open_on_login = settings.open_window_on_login;

            // Keep the start-up entry pointing at this exe (e.g. if the portable exe moved).
            let launcher = app.autolaunch();
            let mut autostart = launcher.is_enabled().unwrap_or(false);
            if autostart {
                autostart = launcher.enable().is_ok();
            }

            let core = Core::new(settings, settings_path, autostart);
            let (discord, worker) = discord::new(core.client_id().to_owned());
            app.manage(AppState {
                core: Mutex::new(core),
                discord: Arc::new(discord),
            });

            let snap = state::lock_core(&handle).snapshot();
            tray::create(&handle, &snap)?;

            update(&handle, |c| c.info(format!("UltimatePresence {} started", env!("CARGO_PKG_VERSION"))));

            let sink_app = handle.clone();
            worker.start(move |ev| state::on_discord_event(&sink_app, ev));
            studio::start(&handle);
            updates::start(&handle);

            if open_on_login || !std::env::args().any(|a| a == MINIMIZED_ARG) {
                show_window(&handle);
            }
            Ok(())
        })
        .build(tauri::generate_context!())
        .expect("failed to start UltimatePresence")
        .run(|_app, event| {
            // Closing the window shouldn't quit; only the tray's Quit (exit code set) does.
            if let tauri::RunEvent::ExitRequested { code, api, .. } = event {
                if code.is_none() {
                    api.prevent_exit();
                }
            }
        });
}
