//! The lines of a Claude transcript: JSON objects, only the user and assistant ones matter.

use crate::fs::FileSystem;
use serde_json::Value;
use std::path::Path;

const TAIL_BYTES: u64 = 256 * 1024;

/// Who wrote a transcript line.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Role {
    /// The model.
    Assistant,
    /// The user side: the prompt and the tool results.
    User,
}

fn role_named(name: &str) -> Option<Role> {
    match name {
        "assistant" => Some(Role::Assistant),
        "user" => Some(Role::User),
        _ => None,
    }
}

/// The role of a line, from its `type` or else its message's role; other lines have none.
#[must_use]
pub fn role_of(entry: &Value) -> Option<Role> {
    let by_type = entry
        .get("type")
        .and_then(Value::as_str)
        .and_then(role_named);
    by_type.or_else(|| {
        let name = entry.pointer("/message/role")?.as_str()?;
        role_named(name)
    })
}

/// The content blocks of a line's message; a plain string is one text block.
#[must_use]
pub fn blocks_of(entry: &Value) -> Vec<Value> {
    match entry.pointer("/message/content") {
        Some(Value::Array(blocks)) => blocks.clone(),
        Some(Value::String(text)) => vec![serde_json::json!({ "type": "text", "text": text })],
        _ => Vec::new(),
    }
}

/// The text of a text block.
#[must_use]
pub fn block_text(block: &Value) -> Option<&str> {
    if block.get("type")?.as_str()? != "text" {
        return None;
    }
    block.get("text")?.as_str()
}

/// The last user or assistant line of a transcript.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LastTurn {
    /// Who wrote it.
    pub role: Role,
    /// Its `stop_reason`, when it has one.
    pub stop: Option<String>,
    /// Its text blocks, trimmed and joined by line breaks.
    pub text: String,
}

fn turn_of(entry: &Value) -> Option<LastTurn> {
    let role = role_of(entry)?;
    let text = blocks_of(entry)
        .iter()
        .filter_map(|b| block_text(b).map(str::trim))
        .filter(|t| !t.is_empty())
        .collect::<Vec<_>>()
        .join("\n");
    let stop = entry
        .pointer("/message/stop_reason")
        .and_then(Value::as_str)
        .map(str::to_owned);
    Some(LastTurn { role, stop, text })
}

/// The last turn of a transcript of `size` bytes; only its tail is read.
#[must_use]
pub fn last_turn(fs: &dyn FileSystem, file: &Path, size: u64) -> Option<LastTurn> {
    let n = size.min(TAIL_BYTES);
    let bytes = fs.read_at(file, size - n, usize::try_from(n).ok()?).ok()?;
    let text = String::from_utf8_lossy(&bytes);
    let mut lines: Vec<&str> = text.split('\n').collect();
    if size > n {
        lines.remove(0);
    }
    lines
        .iter()
        .rev()
        .filter_map(|line| serde_json::from_str::<Value>(line).ok())
        .find_map(|entry| turn_of(&entry))
}
