//! `all_from`: whether a wake was made by the listener itself. The wake line text lives in
//! `argus_wake::line`, so the hook listener and the hosted Overlord say the same words.

use serde_json::Value;

/// Whether every line of the event data `data` was sent by the thread `id` itself (`<id>: ...`),
/// so a lead listening to every thread need not wake for the wakes it made.
pub fn all_from(data: &str, id: &str) -> bool {
    let prefix = format!("{id}: ");
    let Ok(Value::Object(event)) = serde_json::from_str::<Value>(data) else {
        return false;
    };
    match event.get("lines") {
        Some(Value::Array(lines)) => {
            !lines.is_empty()
                && lines
                    .iter()
                    .all(|l| l.as_str().is_some_and(|t| t.starts_with(&prefix)))
        }
        _ => false,
    }
}
