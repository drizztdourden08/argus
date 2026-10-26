//! One tracked sub-agent: where its transcript was read to, and what the lines said.

use super::live::{LastTool, Live, Tokens};
use crate::iso::iso_or;
use crate::record::truthy;
use crate::tool_summary::tool_summary;
use crate::transcript::{Role, blocks_of, role_of};
use serde_json::Value;
use std::collections::HashSet;
use std::path::PathBuf;

/// How far a transcript was read.
#[derive(Debug, Default)]
pub(super) struct Cursor {
    pub(super) offset: u64,
    pub(super) carry: Vec<u8>,
    pub(super) skipping: bool,
}

/// The reading state and the live state of one agent.
#[derive(Debug)]
pub(super) struct Tracked {
    pub(super) project: String,
    pub(super) id: String,
    pub(super) file: PathBuf,
    pub(super) cursor: Cursor,
    pub(super) seen: bool,
    pub(super) closed: bool,
    pub(super) noted: bool,
    pub(super) live: Live,
    pub(super) published: Option<String>,
    pub(super) emitted_at: i64,
    pending: HashSet<String>,
    last_msg_id: Option<String>,
    last_usage: Option<Tokens>,
}

fn usage_of(usage: &Value) -> Tokens {
    let count = |key: &str| usage.get(key).and_then(Value::as_i64).unwrap_or(0);
    Tokens {
        input: count("input_tokens"),
        output: count("output_tokens"),
        cache_read: count("cache_read_input_tokens"),
    }
}

fn string_field<'a>(value: &'a Value, key: &str) -> Option<&'a str> {
    value.get(key)?.as_str()
}

impl Tracked {
    pub(super) fn new(project: &str, id: &str, file: PathBuf) -> Self {
        Self {
            project: project.to_owned(),
            id: id.to_owned(),
            file,
            cursor: Cursor::default(),
            seen: false,
            closed: false,
            noted: false,
            live: Live::default(),
            published: None,
            emitted_at: 0,
            pending: HashSet::new(),
            last_msg_id: None,
            last_usage: None,
        }
    }

    /// Starts reading again from the top, as after a truncated or replaced file.
    pub(super) fn restart(&mut self) {
        self.cursor = Cursor::default();
        self.seen = false;
        self.pending.clear();
        self.last_msg_id = None;
        self.last_usage = None;
        self.live = Live::default();
    }

    /// Whether the doc shows anything yet.
    pub(super) fn visible(&self) -> bool {
        self.seen || self.live.live_note.is_some()
    }

    fn set_activity(&mut self, activity: &str, at: &str, force: bool) {
        if self.live.activity.as_deref() != Some(activity) || force {
            self.live.activity = Some(activity.to_owned());
            self.live.activity_since = Some(at.to_owned());
        }
    }

    fn activity_after(&self, idle: &'static str) -> &'static str {
        if self.pending.is_empty() {
            idle
        } else {
            "tool"
        }
    }

    /// One transcript line into the state. Anything but a user or assistant line is ignored.
    pub(super) fn apply_line(&mut self, line: &str, now_ms: i64, text_max: usize) {
        let Ok(entry) = serde_json::from_str::<Value>(line) else {
            return;
        };
        let Some(role) = role_of(&entry) else {
            return;
        };
        let at = iso_or(string_field(&entry, "timestamp"), now_ms);
        self.seen = true;
        self.live.last_activity_at = Some(at.clone());
        match role {
            Role::Assistant => self.apply_assistant(&entry, &at, text_max),
            Role::User => self.apply_user(&entry, &at),
        }
    }

    fn apply_user(&mut self, entry: &Value, at: &str) {
        for block in blocks_of(entry) {
            if string_field(&block, "type") != Some("tool_result") {
                continue;
            }
            if let Some(id) = string_field(&block, "tool_use_id") {
                self.pending.remove(id);
            }
        }
        let activity = self.activity_after("waiting");
        self.set_activity(activity, at, false);
    }

    fn apply_assistant(&mut self, entry: &Value, at: &str, text_max: usize) {
        let message = entry.get("message").unwrap_or(&Value::Null);
        let id = string_field(message, "id")
            .filter(|s| !s.is_empty())
            .map(str::to_owned);
        let again = id.is_some() && id == self.last_msg_id;
        if !again {
            self.live.turns += 1;
        }
        self.add_usage(message, again);
        self.last_msg_id = id;
        if let Some(model) = string_field(message, "model")
            && !model.is_empty()
            && !model.starts_with('<')
        {
            self.live.model_used = Some(model.to_owned());
        }
        let used = self.apply_blocks(entry, text_max);
        let activity = self.activity_after("thinking");
        self.set_activity(activity, at, used);
    }

    fn add_usage(&mut self, message: &Value, again: bool) {
        let usage = message.get("usage").filter(|u| truthy(u));
        let Some(usage) = usage else {
            if !again {
                self.last_usage = None;
            }
            return;
        };
        let u = usage_of(usage);
        if again && let Some(last) = self.last_usage {
            self.live.tokens = self.live.tokens.minus(last);
        }
        self.live.tokens = self.live.tokens.plus(u);
        self.last_usage = Some(u);
    }

    fn apply_blocks(&mut self, entry: &Value, text_max: usize) -> bool {
        let mut used = false;
        for block in blocks_of(entry) {
            match string_field(&block, "type") {
                Some("text") => self.note_text(&block, text_max),
                Some("tool_use") => {
                    used = true;
                    self.note_tool(&block);
                }
                _ => {}
            }
        }
        used
    }

    fn note_text(&mut self, block: &Value, text_max: usize) {
        let Some(text) = string_field(block, "text").map(str::trim) else {
            return;
        };
        if !text.is_empty() {
            self.live.last_message = Some(text.chars().take(text_max).collect());
        }
    }

    fn note_tool(&mut self, block: &Value) {
        self.live.tool_calls += 1;
        if let Some(id) = string_field(block, "id") {
            self.pending.insert(id.to_owned());
        }
        let name = string_field(block, "name").unwrap_or("");
        let input = block.get("input").unwrap_or(&Value::Null);
        self.live.last_tool = Some(LastTool {
            name: name.to_owned(),
            summary: tool_summary(name, input),
        });
    }
}
