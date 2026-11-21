//! System tray icon and menu.

use crate::sources::ROBLOX_STUDIO;
use crate::state::{lock_core, Snapshot};
use std::sync::Mutex;
use tauri::image::Image;
use tauri::menu::{CheckMenuItem, Menu, MenuItem, PredefinedMenuItem};
use tauri::tray::{MouseButton, MouseButtonState, TrayIcon, TrayIconBuilder, TrayIconEvent};
use tauri::{AppHandle, Manager, Wry};

const ICON_ACTIVE: &[u8] = include_bytes!("../icons/tray.png");
const ICON_IDLE: &[u8] = include_bytes!("../icons/tray-idle.png");

pub struct TrayState {
    tray: TrayIcon,
    discord: MenuItem<Wry>,
    studio: MenuItem<Wry>,
    activity: MenuItem<Wry>,
    pause: CheckMenuItem<Wry>,
    autostart: CheckMenuItem<Wry>,
    last: Mutex<Option<View>>,
}

#[derive(Clone, PartialEq)]
struct View {
    discord: String,
    studio: String,
    activity: String,
    tooltip: String,
    active_icon: bool,
}

fn clip(s: &str, max: usize) -> String {
    if s.chars().count() <= max {
        s.to_owned()
    } else {
        let mut t: String = s.chars().take(max - 1).collect();
        t.push('…');
        t
    }
}

fn view(s: &Snapshot) -> View {
    let discord = match s.discord_status {
        "connected" => format!("Discord: connected as {}", s.discord_user.as_deref().unwrap_or("?")),
        "rejected" => "Discord: Application ID rejected".to_owned(),
        _ => "Discord: not running".to_owned(),
    };
    let studio = s
        .source(ROBLOX_STUDIO)
        .map(|v| format!("{}: {}", v.name, v.status_text.to_lowercase()))
        .unwrap_or_default();
    let summary = s.shown.as_ref().map(|a| {
        [a.details.as_deref(), a.state.as_deref()].into_iter().flatten().collect::<Vec<_>>().join(" · ")
    });
    let activity = if s.paused {
        "Sharing paused".to_owned()
    } else {
        match &summary {
            Some(t) if !t.is_empty() => format!("Showing: {}", clip(t, 56)),
            Some(_) => "Showing activity".to_owned(),
            None => "Nothing shared right now".to_owned(),
        }
    };
    let tooltip = clip(
        &match (&summary, s.discord_status, s.paused) {
            (_, _, true) => "UltimatePresence · paused".to_owned(),
            (_, st, _) if st != "connected" => "UltimatePresence · waiting for Discord".to_owned(),
            (Some(t), _, _) => format!("UltimatePresence · {t}"),
            (None, _, _) => "UltimatePresence · idle".to_owned(),
        },
        120,
    );
    View { active_icon: s.shown.is_some() && s.discord_status == "connected", discord, studio, activity, tooltip }
}

pub fn create(app: &AppHandle, snap: &Snapshot) -> tauri::Result<()> {
    let discord = MenuItem::with_id(app, "status-discord", "Discord", false, None::<&str>)?;
    let studio = MenuItem::with_id(app, "status-studio", "Roblox Studio", false, None::<&str>)?;
    let activity = MenuItem::with_id(app, "status-activity", "", false, None::<&str>)?;
    let pause = CheckMenuItem::with_id(app, "pause", "Pause sharing", true, snap.paused, None::<&str>)?;
    let autostart_label = if cfg!(target_os = "macos") { "Open at login" } else { "Start with Windows" };
    let autostart = CheckMenuItem::with_id(app, "autostart", autostart_label, true, snap.autostart, None::<&str>)?;
    let open = MenuItem::with_id(app, "open", "Open UltimatePresence", true, None::<&str>)?;
    let quit = MenuItem::with_id(app, "quit", "Quit", true, None::<&str>)?;

    let menu = Menu::with_items(
        app,
        &[
            &discord,
            &studio,
            &activity,
            &PredefinedMenuItem::separator(app)?,
            &pause,
            &autostart,
            &PredefinedMenuItem::separator(app)?,
            &open,
            &quit,
        ],
    )?;

    let tray = TrayIconBuilder::with_id("main")
        .icon(Image::from_bytes(ICON_IDLE)?)
        .tooltip("UltimatePresence")
        .menu(&menu)
        .show_menu_on_left_click(false)
        .on_menu_event(|app, event| handle_menu(app, event.id().as_ref()))
        .on_tray_icon_event(|tray, event| {
            if let TrayIconEvent::Click { button: MouseButton::Left, button_state: MouseButtonState::Up, .. } = event {
                crate::show_window(tray.app_handle());
            }
        })
        .build(app)?;

    app.manage(TrayState { tray, discord, studio, activity, pause, autostart, last: Mutex::new(None) });
    refresh(app, snap);
    Ok(())
}

fn handle_menu(app: &AppHandle, id: &str) {
    let (paused, autostart) = {
        let c = lock_core(app);
        (c.settings.paused, c.autostart)
    };
    match id {
        "pause" => crate::commands::apply_paused(app, !paused),
        "autostart" => {
            let _ = crate::commands::apply_autostart(app, !autostart);
        }
        "open" => crate::show_window(app),
        "quit" => crate::quit(app),
        _ => {}
    }
}

pub fn refresh(app: &AppHandle, snap: &Snapshot) {
    let Some(t) = app.try_state::<TrayState>() else { return };
    // Check items toggle themselves when clicked; always restore the real value.
    let _ = t.pause.set_checked(snap.paused);
    let _ = t.autostart.set_checked(snap.autostart);

    let v = view(snap);
    let mut last = t.last.lock().unwrap_or_else(std::sync::PoisonError::into_inner);
    let prev = last.as_ref();
    if prev.map(|p| &p.discord) != Some(&v.discord) {
        let _ = t.discord.set_text(&v.discord);
    }
    if prev.map(|p| &p.studio) != Some(&v.studio) {
        let _ = t.studio.set_text(&v.studio);
    }
    if prev.map(|p| &p.activity) != Some(&v.activity) {
        let _ = t.activity.set_text(&v.activity);
    }
    if prev.map(|p| &p.tooltip) != Some(&v.tooltip) {
        let _ = t.tray.set_tooltip(Some(&v.tooltip));
    }
    if prev.map(|p| p.active_icon) != Some(v.active_icon) {
        if let Ok(img) = Image::from_bytes(if v.active_icon { ICON_ACTIVE } else { ICON_IDLE }) {
            let _ = t.tray.set_icon(Some(img));
        }
    }
    *last = Some(v);
}
