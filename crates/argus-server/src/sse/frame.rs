//! The frame writers. Legacy frames carry an optional `id:` line; mux frames never do, because
//! one `Last-Event-ID` cannot describe many scopes.

use serde_json::Value;

/// One frame: `id: <id>\nevent: <name>\ndata: <json>\n\n`.
#[must_use]
pub fn frame(name: &str, data: &Value, id: Option<i64>) -> String {
    frame_with(name, data, id.map(|id| id.to_string()).as_deref())
}

/// A frame whose id is text, such as the wake ids `W-12`.
#[must_use]
pub fn frame_with(name: &str, data: &Value, id: Option<&str>) -> String {
    let head = id.map(|id| format!("id: {id}\n")).unwrap_or_default();
    format!("{head}{}", mux_frame(name, data))
}

/// A frame with no `id:` line, as the mux stream writes: `event: <name>\ndata: <json>\n\n`.
#[must_use]
pub fn mux_frame(name: &str, data: &Value) -> String {
    let data = argus_core::json_text::to_string(data).unwrap_or_default();
    format!("event: {name}\ndata: {data}\n\n")
}

// test-tag: critical
#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::*;

    #[test]
    fn a_mux_frame_has_no_id_line() {
        let text = mux_frame("ready", &json!({"scope": "a", "seq": 3}));
        assert_eq!(text, "event: ready\ndata: {\"scope\":\"a\",\"seq\":3}\n\n");
        assert!(!text.contains("id:"));
    }

    #[test]
    fn a_mux_frame_is_one_data_line_ended_by_a_blank_line() {
        let text = mux_frame("snapshot", &json!({"docs": [{"a": 1}, {"b": 2}]}));
        assert!(text.ends_with("\n\n"));
        assert_eq!(text.lines().filter(|l| l.starts_with("data: ")).count(), 1);
    }

    #[test]
    fn the_legacy_frames_keep_their_id_line() {
        let data = json!({"seq": 1});
        assert_eq!(
            frame("changes", &data, Some(7)),
            "id: 7\nevent: changes\ndata: {\"seq\":1}\n\n"
        );
        assert_eq!(
            frame_with("wake", &data, Some("W-12")),
            "id: W-12\nevent: wake\ndata: {\"seq\":1}\n\n"
        );
    }

    #[test]
    fn a_legacy_frame_without_an_id_matches_the_mux_frame() {
        let data = json!({"seq": 1});
        assert_eq!(frame("ping", &data, None), mux_frame("ping", &data));
    }
}
