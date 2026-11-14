//! The typed events of the stream.

use serde::Deserialize;
use serde_json::{Map, Value};

use super::block::Block;

/// An event kept whole because its type is not one the session needs, or its shape did not fit.
#[derive(Debug, Clone, PartialEq)]
pub struct Raw {
    /// The `type` field, empty when the line had none.
    pub kind: String,
    pub value: Value,
}

/// `system/init`: the first event of a session.
#[derive(Debug, Clone, Default, PartialEq, Deserialize)]
#[serde(default)]
pub struct Init {
    pub session_id: String,
    /// `"none"` is the subscription; the start guard of story 4 reads this.
    #[serde(rename = "apiKeySource")]
    pub api_key_source: Option<String>,
    pub model: Option<String>,
    pub tools: Vec<String>,
    pub mcp_servers: Vec<Value>,
    /// Every field not named above.
    #[serde(flatten)]
    pub extra: Map<String, Value>,
}

/// An assistant or user message with its content blocks.
#[derive(Debug, Clone, PartialEq)]
pub struct Message {
    pub session_id: String,
    pub parent_tool_use_id: Option<String>,
    pub blocks: Vec<Block>,
}

/// Token counters of a `result`.
#[derive(Debug, Clone, Default, PartialEq, Eq, Deserialize)]
#[serde(default)]
pub struct Usage {
    pub input_tokens: u64,
    pub output_tokens: u64,
    pub cache_creation_input_tokens: u64,
    pub cache_read_input_tokens: u64,
}

/// `result`: the end of a turn, with its cost.
#[derive(Debug, Clone, Default, PartialEq, Deserialize)]
#[serde(default)]
pub struct Outcome {
    pub subtype: String,
    pub is_error: bool,
    pub session_id: String,
    pub result: Option<String>,
    /// Cumulative for the session, as the CLI reports it.
    pub total_cost_usd: Option<f64>,
    pub duration_ms: Option<u64>,
    pub num_turns: Option<u64>,
    pub usage: Option<Usage>,
    /// Every field not named above.
    #[serde(flatten)]
    pub extra: Map<String, Value>,
}

/// One decoded line.
#[derive(Debug, Clone, PartialEq)]
pub enum Event {
    Init(Init),
    Assistant(Message),
    User(Message),
    /// A partial-message event; its `event` payload is kept as is.
    Stream(Value),
    Result(Outcome),
    Other(Raw),
}
