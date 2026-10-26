//! The live state of one sub-agent, as it is shown under `live` on its agents doc.

use serde::Serialize;

/// The tool in use.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct LastTool {
    /// The tool name.
    pub name: String,
    /// What the call is about.
    pub summary: String,
}

/// Token counts summed over the transcript.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Tokens {
    /// Input tokens.
    pub input: i64,
    /// Output tokens.
    pub output: i64,
    /// Tokens read from the cache.
    pub cache_read: i64,
}

impl Tokens {
    pub(super) fn plus(self, other: Self) -> Self {
        Self {
            input: self.input + other.input,
            output: self.output + other.output,
            cache_read: self.cache_read + other.cache_read,
        }
    }

    pub(super) fn minus(self, other: Self) -> Self {
        Self {
            input: self.input - other.input,
            output: self.output - other.output,
            cache_read: self.cache_read - other.cache_read,
        }
    }
}

/// What the watcher knows about a sub-agent right now.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Live {
    /// The last assistant text, cut.
    pub last_message: Option<String>,
    /// The last tool call.
    pub last_tool: Option<LastTool>,
    /// `tool`, `thinking` or `waiting`.
    pub activity: Option<String>,
    /// Since when the activity holds.
    pub activity_since: Option<String>,
    /// The time of the last transcript line.
    pub last_activity_at: Option<String>,
    /// Assistant messages so far.
    pub turns: u64,
    /// Tool calls so far.
    pub tool_calls: u64,
    /// The model the agent really runs on.
    pub model_used: Option<String>,
    /// Token sums.
    pub tokens: Tokens,
    /// Whether a working agent has gone quiet.
    pub stalled: bool,
    /// A note, such as that no transcript was found.
    pub live_note: Option<String>,
}
