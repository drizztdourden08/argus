//! A short line of what a tool call is about, for the table and the tab.

use serde_json::Value;

const MAX_CHARS: usize = 200;
const KEYS: [&str; 5] = ["command", "file_path", "path", "url", "pattern"];

/// The first of `command`, `file_path`, `path`, `url`, `pattern` in the input, else the input as
/// compact JSON, cut to 200 characters.
#[must_use]
pub fn summarize(input: &Value) -> String {
    let text = KEYS
        .iter()
        .find_map(|key| input.get(key).and_then(Value::as_str))
        .map_or_else(|| input.to_string(), str::to_owned);
    text.chars().take(MAX_CHARS).collect()
}
