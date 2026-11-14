//! The "would do" log: for each turn of the shadow, the wakes it heard and what it said it would
//! do, and each call it was refused. One JSON object per entry.

use serde_json::{Value, json};

use crate::protocol::{Block, Event, decode_line};

/// What one line of the child's output means to the log.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Seen {
    /// Text the model said.
    Said(String),
    /// The turn ended.
    TurnEnd,
    Nothing,
}

/// Reads one stdout line of the child.
#[must_use]
pub fn seen(line: &str) -> Seen {
    match decode_line(line) {
        Ok(Event::Assistant(message)) => {
            let text: Vec<&str> = message
                .blocks
                .iter()
                .filter_map(|block| match block {
                    Block::Text { text } => Some(text.as_str()),
                    _ => None,
                })
                .collect();
            if text.is_empty() {
                Seen::Nothing
            } else {
                Seen::Said(text.join("\n"))
            }
        }
        Ok(Event::Result(_)) => Seen::TurnEnd,
        _ => Seen::Nothing,
    }
}

/// The wakes and the words of the turn that runs.
#[derive(Debug, Default)]
pub struct Turn {
    wakes: Vec<String>,
    said: Vec<String>,
}

impl Turn {
    /// A wake the child took.
    pub fn wake(&mut self, text: String) {
        self.wakes.push(text);
    }

    /// Words of the model.
    pub fn said(&mut self, text: String) {
        self.said.push(text);
    }

    /// The entry of the turn that just ended, and a fresh turn. None when nothing happened.
    pub fn finish(&mut self, at: &str) -> Option<Value> {
        let turn = std::mem::take(self);
        if turn.wakes.is_empty() && turn.said.is_empty() {
            return None;
        }
        Some(json!({
            "kind": "would-do",
            "at": at,
            "wakes": turn.wakes,
            "said": turn.said.join("\n"),
        }))
    }
}

/// The entry of a call the shadow was refused.
#[must_use]
pub fn denial_entry(at: &str, tool: &str, summary: &str) -> Value {
    json!({ "kind": "denied", "at": at, "tool": tool, "summary": summary })
}
