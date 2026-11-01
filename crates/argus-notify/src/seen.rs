//! The markers of what the owner was already told, kept in kv `notify:seen`.
// test-tag: critical

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

const ASKED_KEEP: usize = 200;

/// The markers of one project: the handed build (`<tag>|<handedAt>`) and the last questions told.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Marker {
    /// The build handed when last seen.
    #[serde(default)]
    pub handed: Option<String>,
    /// The ids of the last 200 questions seen.
    #[serde(default)]
    pub asked: Vec<String>,
}

impl Marker {
    /// Remembers a question, keeping only the last 200.
    pub fn remember(&mut self, id: &str) {
        self.asked.push(id.to_owned());
        let extra = self.asked.len().saturating_sub(ASKED_KEEP);
        self.asked.drain(..extra);
    }
}

/// The markers of every project.
pub type SeenMap = BTreeMap<String, Marker>;

/// The markers saved in `json`; none when there are none or they cannot be read.
#[must_use]
pub fn parse(json: Option<&str>) -> SeenMap {
    json.and_then(|text| serde_json::from_str(text).ok())
        .unwrap_or_default()
}

/// The JSON to save.
#[must_use]
pub fn render(seen: &SeenMap) -> String {
    serde_json::to_string(seen).unwrap_or_default()
}

#[cfg(test)]
mod tests {
    use super::{Marker, SeenMap, parse, render};

    #[test]
    fn the_markers_round_trip_in_the_shape_of_the_node_value() {
        let mut seen = SeenMap::new();
        seen.insert(
            "rotp".into(),
            Marker {
                handed: Some("b1|2026".into()),
                asked: vec!["Q-1".into()],
            },
        );
        let json = render(&seen);
        assert_eq!(json, r#"{"rotp":{"handed":"b1|2026","asked":["Q-1"]}}"#);
        assert_eq!(parse(Some(&json)), seen);
    }

    #[test]
    fn a_value_that_cannot_be_read_is_no_markers() {
        assert!(parse(Some("[1,2]")).is_empty());
        assert!(parse(Some("{")).is_empty());
        assert!(parse(None).is_empty());
    }

    #[test]
    fn only_the_last_200_questions_are_kept() {
        let mut marker = Marker::default();
        for n in 0..205 {
            marker.remember(&format!("Q-{n}"));
        }
        assert_eq!(marker.asked.len(), 200);
        assert_eq!(marker.asked[0], "Q-5");
    }
}
