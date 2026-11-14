//! Content blocks of an assistant or user message.

use serde_json::Value;

/// One content block. Anything not listed is kept whole in [`Block::Unknown`].
#[derive(Debug, Clone, PartialEq)]
pub enum Block {
    Text {
        text: String,
    },
    Thinking {
        thinking: String,
    },
    ToolUse {
        id: String,
        name: String,
        input: Value,
    },
    ToolResult {
        tool_use_id: String,
        content: Value,
        is_error: bool,
    },
    Unknown(Value),
}

fn str_field(value: &Value, key: &str) -> String {
    value
        .get(key)
        .and_then(Value::as_str)
        .unwrap_or_default()
        .to_string()
}

fn parse_block(value: Value) -> Block {
    let kind = value.get("type").and_then(Value::as_str).unwrap_or("");
    match kind {
        "text" => Block::Text {
            text: str_field(&value, "text"),
        },
        "thinking" => Block::Thinking {
            thinking: str_field(&value, "thinking"),
        },
        "tool_use" => Block::ToolUse {
            id: str_field(&value, "id"),
            name: str_field(&value, "name"),
            input: value.get("input").cloned().unwrap_or(Value::Null),
        },
        "tool_result" => Block::ToolResult {
            tool_use_id: str_field(&value, "tool_use_id"),
            content: value.get("content").cloned().unwrap_or(Value::Null),
            is_error: value
                .get("is_error")
                .and_then(Value::as_bool)
                .unwrap_or(false),
        },
        _ => Block::Unknown(value),
    }
}

/// Turn a message `content` (a block array, or a plain string) into blocks.
pub(super) fn parse_blocks(content: Option<&Value>) -> Vec<Block> {
    match content {
        Some(Value::Array(items)) => items.iter().cloned().map(parse_block).collect(),
        Some(Value::String(text)) => vec![Block::Text { text: text.clone() }],
        _ => Vec::new(),
    }
}
