//! What a tool call is about, in a few words for the Now cell of the Agents view.

use crate::text::clip;
use serde_json::Value;

const FALLBACK_KEYS: [&str; 8] = [
    "file_path",
    "path",
    "pattern",
    "query",
    "url",
    "description",
    "command",
    "prompt",
];

fn field(input: &Value, key: &str) -> String {
    input
        .get(key)
        .and_then(Value::as_str)
        .unwrap_or("")
        .to_owned()
}

fn file_field(input: &Value) -> String {
    let file = input.get("file_path").filter(|v| !v.is_null());
    let value = file.or_else(|| input.get("notebook_path"));
    value.and_then(Value::as_str).unwrap_or("").to_owned()
}

fn first_text(input: &Value) -> String {
    FALLBACK_KEYS
        .iter()
        .map(|key| field(input, key))
        .find(|text| !text.is_empty())
        .map_or_else(String::new, |text| clip(&text, 80))
}

/// A few words on what the call does: the command, the path, the pattern or the description.
#[must_use]
pub fn tool_summary(name: &str, input: &Value) -> String {
    match name {
        "Bash" => clip(&field(input, "command"), 80),
        "Edit" | "Write" | "Read" | "MultiEdit" | "NotebookEdit" => clip(&file_field(input), 300),
        "Grep" | "Glob" => clip(&field(input, "pattern"), 80),
        "Agent" | "Task" => clip(&field(input, "description"), 80),
        _ => first_text(input),
    }
}
