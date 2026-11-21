//! Persisted user settings (JSON in the app config dir).

use crate::sources::{self, SourceInfo, ROBLOX_STUDIO};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::path::Path;

/// Two lines of text Discord shows under the app name.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct Template {
    pub top: String,
    pub bottom: String,
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct ButtonSetting {
    pub label: String,
    pub url: String,
}

/// Per-app settings. Planned apps use the same shape so their pages can preview it.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct SourceSettings {
    /// Show this app's activity on Discord.
    pub enabled: bool,
    /// Discord application that provides the "Playing …" name and images.
    pub application_id: String,

    // Display
    /// "classic" | "minimal" | "detailed" | "custom"
    pub style: String,
    /// Custom wording per activity kind (editing, developing, testing, animating).
    pub templates: BTreeMap<String, Template>,
    /// "session" (since Studio opened) | "activity" (since the current task) | "off"
    pub timer: String,

    // Privacy
    pub hide_script: bool,
    pub hide_place: bool,
    pub hide_lines: bool,
    /// Keep animation / VFX plugin names out of your status ({tool} stays empty).
    pub hide_tools: bool,
    /// Shown instead of a hidden place name.
    pub place_alias: String,
    /// Never share while working in these places (case-insensitive names).
    pub hidden_places: Vec<String>,

    // Behaviour
    pub show_testing: bool,
    pub buttons: Vec<ButtonSetting>,

    /// v0.2 single privacy switch; migrated into hide_script + hide_place, never saved.
    #[serde(skip_serializing)]
    pub privacy: Option<bool>,
}

impl Default for SourceSettings {
    fn default() -> Self {
        Self {
            enabled: true,
            application_id: String::new(),
            style: "classic".into(),
            templates: BTreeMap::new(),
            timer: "session".into(),
            hide_script: false,
            hide_place: false,
            hide_lines: false,
            hide_tools: false,
            place_alias: String::new(),
            hidden_places: Vec::new(),
            show_testing: true,
            buttons: Vec::new(),
            privacy: None,
        }
    }
}

impl SourceSettings {
    fn for_source(info: &SourceInfo) -> Self {
        Self { application_id: info.default_application_id.to_owned(), ..Self::default() }
    }

    fn migrate(&mut self) {
        if let Some(p) = self.privacy.take() {
            if p {
                self.hide_script = true;
                self.hide_place = true;
            }
        }
    }

    /// Checks and tidies values coming from the window.
    pub fn validate(&mut self) -> Result<(), String> {
        if !self.application_id.is_empty() && !is_valid_application_id(&self.application_id) {
            return Err("An Application ID is 17–20 digits. Copy it from the Discord Developer Portal.".into());
        }
        if !["classic", "minimal", "detailed", "custom"].contains(&self.style.as_str()) {
            self.style = "classic".into();
        }
        if !["session", "activity", "off"].contains(&self.timer.as_str()) {
            self.timer = "session".into();
        }
        self.place_alias = self.place_alias.trim().chars().take(64).collect();
        for t in self.templates.values_mut() {
            t.top = t.top.chars().take(128).collect();
            t.bottom = t.bottom.chars().take(128).collect();
        }
        self.hidden_places = self
            .hidden_places
            .iter()
            .map(|p| p.trim().to_owned())
            .filter(|p| !p.is_empty())
            .take(50)
            .collect();
        self.buttons.retain(|b| !(b.label.trim().is_empty() && b.url.trim().is_empty()));
        self.buttons.truncate(2);
        for b in &mut self.buttons {
            b.label = b.label.trim().chars().take(32).collect();
            b.url = b.url.trim().to_owned();
            if b.label.is_empty() {
                return Err("Each button needs a label.".into());
            }
            if !(b.url.starts_with("https://") || b.url.starts_with("http://")) || b.url.len() > 512 {
                return Err(format!("“{}” needs a link starting with https://", b.label));
            }
        }
        Ok(())
    }
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct Settings {
    pub paused: bool,
    /// Show the window when the app starts at sign-in (normally it stays in the tray).
    pub open_window_on_login: bool,
    pub sources: BTreeMap<String, SourceSettings>,
    // v0.1 stored these at the top level; migrated into sources["roblox-studio"].
    #[serde(skip_serializing)]
    application_id: Option<String>,
    #[serde(skip_serializing)]
    privacy_mode: Option<bool>,
}

pub fn is_valid_application_id(id: &str) -> bool {
    (17..=20).contains(&id.len()) && id.bytes().all(|b| b.is_ascii_digit())
}

impl Settings {
    pub fn load(path: &Path) -> Self {
        let mut s: Settings = std::fs::read_to_string(path)
            .ok()
            .and_then(|t| serde_json::from_str(&t).ok())
            .unwrap_or_default();
        s.normalize();
        s
    }

    /// Fills in every catalog source, migrates old fields, repairs bad values.
    fn normalize(&mut self) {
        for info in sources::CATALOG {
            self.sources.entry(info.id.to_owned()).or_insert_with(|| SourceSettings::for_source(info));
        }
        let studio = self.sources.get_mut(ROBLOX_STUDIO).expect("inserted above");
        if let Some(id) = self.application_id.take() {
            if is_valid_application_id(&id) {
                studio.application_id = id;
            }
        }
        if let Some(true) = self.privacy_mode.take() {
            studio.hide_script = true;
            studio.hide_place = true;
        }
        for info in sources::CATALOG {
            let s = self.sources.get_mut(info.id).expect("inserted above");
            s.migrate();
            // The Discord applications are built in (no setup for users), so stored
            // IDs from older versions are replaced.
            s.application_id = info.default_application_id.to_owned();
            let _ = s.validate();
        }
    }

    pub fn source(&self, id: &str) -> SourceSettings {
        self.sources.get(id).cloned().unwrap_or_else(|| {
            sources::find(id).map(SourceSettings::for_source).unwrap_or_default()
        })
    }

    pub fn source_mut(&mut self, id: &str) -> &mut SourceSettings {
        self.sources
            .entry(id.to_owned())
            .or_insert_with(|| sources::find(id).map(SourceSettings::for_source).unwrap_or_default())
    }

    pub fn save(&self, path: &Path) -> std::io::Result<()> {
        if let Some(dir) = path.parent() {
            std::fs::create_dir_all(dir)?;
        }
        let tmp = path.with_extension("json.tmp");
        std::fs::write(&tmp, serde_json::to_vec_pretty(self)?)?;
        std::fs::rename(tmp, path)
    }
}

/// Applies a partial JSON object (camelCase keys) on top of `current`.
pub fn merge<T: Serialize + serde::de::DeserializeOwned>(current: &T, patch: &serde_json::Value) -> Result<T, String> {
    let mut v = serde_json::to_value(current).map_err(|e| e.to_string())?;
    let (Some(obj), Some(p)) = (v.as_object_mut(), patch.as_object()) else {
        return Err("expected an object".into());
    };
    for (k, val) in p {
        if !obj.contains_key(k) {
            return Err(format!("unknown setting: {k}"));
        }
        obj.insert(k.clone(), val.clone());
    }
    serde_json::from_value(v).map_err(|e| format!("invalid setting: {e}"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::sources::STUDIO_APPLICATION_ID;

    #[test]
    fn validates_ids() {
        assert!(is_valid_application_id(STUDIO_APPLICATION_ID));
        assert!(!is_valid_application_id("123"));
        assert!(!is_valid_application_id("10283119368546754a8"));
    }

    #[test]
    fn migrates_v01_settings() {
        let mut s: Settings =
            serde_json::from_str(r#"{"applicationId":"123456789012345678","privacyMode":true,"paused":true}"#).unwrap();
        s.normalize();
        let studio = s.source(ROBLOX_STUDIO);
        assert_eq!(studio.application_id, STUDIO_APPLICATION_ID);
        assert!(studio.hide_script && studio.hide_place && studio.enabled && s.paused);
        assert!(s.sources.contains_key("blender") && s.sources.contains_key("figma"));
        let saved = serde_json::to_value(&s).unwrap();
        assert!(saved.get("privacyMode").is_none() && saved.get("applicationId").is_none(), "{saved}");
    }

    #[test]
    fn migrates_v02_privacy() {
        let mut s: Settings =
            serde_json::from_str(r#"{"sources":{"roblox-studio":{"enabled":true,"applicationId":"1028311936854675458","privacy":true}}}"#).unwrap();
        s.normalize();
        let studio = s.source(ROBLOX_STUDIO);
        assert!(studio.hide_script && studio.hide_place && !studio.hide_lines);
        assert_eq!(studio.style, "classic");
        assert!(serde_json::to_value(&studio).unwrap().get("privacy").is_none());
    }

    #[test]
    fn uses_built_in_applications() {
        let mut s: Settings = serde_json::from_str(r#"{"sources":{"roblox-studio":{"applicationId":"1028311936854675458"}}}"#).unwrap();
        s.normalize();
        assert_eq!(s.source(ROBLOX_STUDIO).application_id, STUDIO_APPLICATION_ID);
        assert_eq!(s.source(crate::sources::BLENDER).application_id, crate::sources::BLENDER_APPLICATION_ID);
    }

    #[test]
    fn merge_and_validate() {
        let base = SourceSettings::default();
        let mut m = merge(&base, &serde_json::json!({ "timer": "off", "hidePlace": true })).unwrap();
        assert!(m.validate().is_ok());
        assert_eq!(m.timer, "off");
        assert!(merge(&base, &serde_json::json!({ "nope": 1 })).is_err());
        let mut bad = merge(&base, &serde_json::json!({ "buttons": [{ "label": "Play", "url": "roblox.com" }] })).unwrap();
        assert!(bad.validate().is_err());
    }
}
