//! Catalog of the apps UltimatePresence can show on Discord.
//!
//! Each app is a "source". A live source has runtime code (Roblox Studio: `studio.rs`,
//! Blender: `blender.rs`); a planned one is listed so the UI can show its page.
//! To add an app: implement its runtime next to those, flip `available`, and add it
//! to `Core::presented` / `candidates` / `source_view` in `state.rs`.

pub const ROBLOX_STUDIO: &str = "roblox-studio";
pub const BLENDER: &str = "blender";

/// UltimatePresence's own Discord applications. Their names ("Roblox Studio",
/// "Blender") are what Discord shows after "Playing", and they hold the icons.
pub const STUDIO_APPLICATION_ID: &str = "1572631495351205898";
pub const BLENDER_APPLICATION_ID: &str = "1573670951235620886";

pub struct SourceInfo {
    pub id: &'static str,
    pub name: &'static str,
    /// false = planned; shown as "Coming soon" with placeholder settings.
    pub available: bool,
    /// The Discord application that shows this app (empty for planned apps).
    pub default_application_id: &'static str,
}

pub const CATALOG: &[SourceInfo] = &[
    SourceInfo { id: ROBLOX_STUDIO, name: "Roblox Studio", available: true, default_application_id: STUDIO_APPLICATION_ID },
    SourceInfo { id: BLENDER, name: "Blender", available: true, default_application_id: BLENDER_APPLICATION_ID },
    SourceInfo { id: "substance-painter", name: "Substance Painter", available: false, default_application_id: "" },
    SourceInfo { id: "figma", name: "Figma", available: false, default_application_id: "" },
];

pub fn find(id: &str) -> Option<&'static SourceInfo> {
    CATALOG.iter().find(|s| s.id == id)
}
