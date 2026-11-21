//! One-click setup for the in-app integrations: the Roblox Studio plugin and the
//! Blender script. Both ship inside the app (see `bundle.resources` in tauri.conf.json).

use serde::Serialize;
use std::fs;
use std::path::{Path, PathBuf};
use tauri::{AppHandle, Manager};

const PLUGIN_FILE: &str = "UltimatePresence.rbxmx";
const ORIGINAL_PLUGIN: &str = "studiopresence.rbxm";
const BLENDER_FILE: &str = "ultimate_presence.py";

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Integrations {
    /// "installed" | "outdated" | "original" (only StudioPresence's plugin) | "missing"
    pub studio_plugin: &'static str,
    /// "installed" | "outdated" | "missing" | "no-blender" (Blender never opened)
    pub blender_addon: &'static str,
    /// Blender versions the script is installed for, e.g. ["5.3"].
    pub blender_versions: Vec<String>,
}

/// A file that ships with the app, falling back to the dev checkout.
fn bundled(app: &AppHandle, resource: &str, dev: &str) -> Option<PathBuf> {
    app.path()
        .resource_dir()
        .ok()
        .map(|d| d.join(resource))
        .filter(|p| p.exists())
        .or_else(|| {
            let p = Path::new(env!("CARGO_MANIFEST_DIR")).join(dev);
            p.exists().then_some(p)
        })
}

fn plugin_source(app: &AppHandle) -> Option<PathBuf> {
    bundled(app, "studio-plugin/UltimatePresence.rbxmx", "../dist/UltimatePresence.rbxmx")
}

fn blender_source(app: &AppHandle) -> Option<PathBuf> {
    bundled(app, "blender-addon/ultimate_presence.py", "../blender/ultimate_presence.py")
}

fn roblox_plugins_dir() -> Option<PathBuf> {
    std::env::var_os("LOCALAPPDATA").map(|d| PathBuf::from(d).join("Roblox").join("Plugins"))
}

/// Blender's per-version config folders, e.g. %APPDATA%\Blender Foundation\Blender\5.3.
fn blender_version_dirs() -> Vec<PathBuf> {
    let Some(root) = std::env::var_os("APPDATA").map(|d| PathBuf::from(d).join("Blender Foundation").join("Blender")) else {
        return Vec::new();
    };
    let mut dirs: Vec<PathBuf> = fs::read_dir(root)
        .map(|rd| {
            rd.filter_map(Result::ok)
                .map(|e| e.path())
                .filter(|p| p.is_dir() && p.file_name().and_then(|n| n.to_str()).is_some_and(|n| n.chars().next().is_some_and(|c| c.is_ascii_digit())))
                .collect()
        })
        .unwrap_or_default();
    dirs.sort();
    dirs
}

fn same_file(a: &Path, b: &Path) -> bool {
    match (fs::read(a), fs::read(b)) {
        (Ok(x), Ok(y)) => x == y,
        _ => false,
    }
}

pub fn status(app: &AppHandle) -> Integrations {
    let studio_plugin = match roblox_plugins_dir() {
        Some(dir) => {
            let ours = dir.join(PLUGIN_FILE);
            if ours.exists() {
                match plugin_source(app) {
                    Some(src) if !same_file(&src, &ours) => "outdated",
                    _ => "installed",
                }
            } else if dir.join(ORIGINAL_PLUGIN).exists() {
                "original"
            } else {
                "missing"
            }
        }
        None => "missing",
    };

    let versions = blender_version_dirs();
    let src = blender_source(app);
    let mut installed = Vec::new();
    let mut outdated = false;
    for dir in &versions {
        let target = dir.join("scripts").join("startup").join(BLENDER_FILE);
        if target.exists() {
            installed.push(dir.file_name().unwrap_or_default().to_string_lossy().into_owned());
            if src.as_ref().is_some_and(|s| !same_file(s, &target)) {
                outdated = true;
            }
        }
    }
    let blender_addon = if versions.is_empty() {
        "no-blender"
    } else if installed.is_empty() {
        "missing"
    } else if outdated {
        "outdated"
    } else {
        "installed"
    };
    Integrations { studio_plugin, blender_addon, blender_versions: installed }
}

/// Copies the plugin into Studio's local Plugins folder and turns off StudioPresence's
/// original plugin (renamed, not deleted) so both don't send updates.
pub fn install_studio_plugin(app: &AppHandle) -> Result<String, String> {
    let src = plugin_source(app).ok_or("The plugin file is missing from this install. Reinstall UltimatePresence.")?;
    let dir = roblox_plugins_dir().ok_or("Couldn't find Roblox's Plugins folder.")?;
    fs::create_dir_all(&dir).map_err(|e| format!("Couldn't create {}: {e}", dir.display()))?;
    fs::copy(&src, dir.join(PLUGIN_FILE)).map_err(|e| format!("Couldn't copy the plugin: {e}"))?;
    let original = dir.join(ORIGINAL_PLUGIN);
    let mut note = String::new();
    if original.exists() {
        let _ = fs::rename(&original, dir.join(format!("{ORIGINAL_PLUGIN}.disabled")));
        note = " The original StudioPresence plugin was turned off.".into();
    }
    Ok(format!("Plugin installed. Restart Roblox Studio to load it.{note}"))
}

/// Puts the Blender script in every Blender version's startup folder, so it runs
/// automatically with nothing to enable.
pub fn install_blender(app: &AppHandle) -> Result<String, String> {
    let src = blender_source(app).ok_or("The Blender script is missing from this install. Reinstall UltimatePresence.")?;
    let versions = blender_version_dirs();
    if versions.is_empty() {
        return Err("Couldn't find Blender's settings. Open Blender once, close it, then try again.".into());
    }
    let mut done = Vec::new();
    for dir in &versions {
        let startup = dir.join("scripts").join("startup");
        fs::create_dir_all(&startup).map_err(|e| format!("Couldn't create {}: {e}", startup.display()))?;
        fs::copy(&src, startup.join(BLENDER_FILE)).map_err(|e| format!("Couldn't copy the script: {e}"))?;
        done.push(dir.file_name().unwrap_or_default().to_string_lossy().into_owned());
    }
    Ok(format!("Installed for Blender {}. Restart Blender and it connects by itself.", done.join(", ")))
}
