//! App-wide activity model and conversion to Discord's SET_ACTIVITY payload.

use serde::Serialize;
use serde_json::{json, Map, Value};

#[derive(Clone, Debug, Default, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Activity {
    pub details: Option<String>,
    pub state: Option<String>,
    /// Unix time in milliseconds.
    pub start_timestamp: Option<i64>,
    pub large_image: Option<String>,
    pub large_text: Option<String>,
    pub small_image: Option<String>,
    pub small_text: Option<String>,
    pub buttons: Vec<Button>,
}

#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct Button {
    pub label: String,
    pub url: String,
}

/// Discord requires text fields to be 2..=128 characters.
fn text(value: &Option<String>) -> Option<String> {
    let s = value.as_deref()?.trim();
    if s.is_empty() {
        return None;
    }
    let mut out: String = if s.chars().count() > 128 {
        let mut t: String = s.chars().take(127).collect();
        t.push('…');
        t
    } else {
        s.to_owned()
    };
    while out.chars().count() < 2 {
        out.push('\u{2800}'); // braille blank: renders as whitespace, counts as a character
    }
    Some(out)
}

fn key(value: &Option<String>) -> Option<String> {
    let s = value.as_deref()?.trim();
    (!s.is_empty() && s.len() <= 256).then(|| s.to_owned())
}

impl Activity {
    pub fn to_discord_json(&self) -> Value {
        let mut a = Map::new();
        if let Some(v) = text(&self.details) {
            a.insert("details".into(), v.into());
        }
        if let Some(v) = text(&self.state) {
            a.insert("state".into(), v.into());
        }
        if let Some(ts) = self.start_timestamp {
            a.insert("timestamps".into(), json!({ "start": ts }));
        }

        let mut assets = Map::new();
        if let Some(k) = key(&self.large_image) {
            assets.insert("large_image".into(), k.into());
            if let Some(t) = text(&self.large_text) {
                assets.insert("large_text".into(), t.into());
            }
        }
        if let Some(k) = key(&self.small_image) {
            assets.insert("small_image".into(), k.into());
            if let Some(t) = text(&self.small_text) {
                assets.insert("small_text".into(), t.into());
            }
        }
        if !assets.is_empty() {
            a.insert("assets".into(), Value::Object(assets));
        }

        let buttons: Vec<Value> = self
            .buttons
            .iter()
            .filter(|b| {
                !b.label.trim().is_empty()
                    && b.url.len() <= 512
                    && (b.url.starts_with("https://") || b.url.starts_with("http://"))
            })
            .take(2)
            .map(|b| json!({ "label": b.label.trim().chars().take(32).collect::<String>(), "url": b.url }))
            .collect();
        if !buttons.is_empty() {
            a.insert("buttons".into(), Value::Array(buttons));
        }

        a.insert("instance".into(), false.into());
        Value::Object(a)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pads_short_and_truncates_long_text() {
        let a = Activity {
            details: Some("x".into()),
            state: Some("y".repeat(200)),
            ..Default::default()
        };
        let v = a.to_discord_json();
        assert_eq!(v["details"].as_str().unwrap().chars().count(), 2);
        assert_eq!(v["state"].as_str().unwrap().chars().count(), 128);
    }

    #[test]
    fn omits_empty_fields() {
        let v = Activity::default().to_discord_json();
        assert_eq!(v, json!({ "instance": false }));
    }

    #[test]
    fn full_payload() {
        let a = Activity {
            details: Some("Main (Line 4 / 20)".into()),
            state: Some("Workspace: My Game".into()),
            start_timestamp: Some(1_700_000_000_000),
            large_image: Some("serverscript".into()),
            large_text: Some("Editing a SERVER Script".into()),
            small_image: None,
            small_text: Some("ignored without image".into()),
            buttons: vec![],
        };
        assert_eq!(
            a.to_discord_json(),
            json!({
                "details": "Main (Line 4 / 20)",
                "state": "Workspace: My Game",
                "timestamps": { "start": 1_700_000_000_000i64 },
                "assets": { "large_image": "serverscript", "large_text": "Editing a SERVER Script" },
                "instance": false
            })
        );
    }
}
