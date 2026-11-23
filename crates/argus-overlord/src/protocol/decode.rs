//! Line to [`Event`].

use serde::de::DeserializeOwned;
use serde_json::Value;

use super::block::parse_blocks;
use super::event::{Event, Message, Raw};

/// Why a line could not be read at all. A well formed line of an unknown type is not an error.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DecodeError {
    Blank,
    Malformed(String),
}

fn raw(value: Value) -> Event {
    let kind = value
        .get("type")
        .and_then(Value::as_str)
        .unwrap_or_default()
        .to_string();
    Event::Other(Raw { kind, value })
}

fn typed<T: DeserializeOwned>(value: &Value, wrap: impl FnOnce(T) -> Event) -> Option<Event> {
    serde_json::from_value::<T>(value.clone()).ok().map(wrap)
}

fn message(value: &Value) -> Message {
    let session_id = value
        .get("session_id")
        .and_then(Value::as_str)
        .unwrap_or_default()
        .to_string();
    let parent_tool_use_id = value
        .get("parent_tool_use_id")
        .and_then(Value::as_str)
        .map(str::to_string);
    let body = value.get("message");
    let blocks = parse_blocks(body.and_then(|m| m.get("content")));
    let usage = body
        .and_then(|m| m.get("usage"))
        .and_then(|u| serde_json::from_value(u.clone()).ok());
    Message {
        session_id,
        parent_tool_use_id,
        blocks,
        usage,
    }
}

fn classify(value: Value) -> Event {
    let kind = value.get("type").and_then(Value::as_str).unwrap_or("");
    let subtype = value.get("subtype").and_then(Value::as_str).unwrap_or("");
    let known = match (kind, subtype) {
        ("system", "init") => typed(&value, Event::Init),
        ("assistant", _) => Some(Event::Assistant(message(&value))),
        ("user", _) => Some(Event::User(message(&value))),
        ("stream_event", _) => Some(Event::Stream(
            value.get("event").cloned().unwrap_or(Value::Null),
        )),
        ("result", _) => typed(&value, Event::Result),
        _ => None,
    };
    known.unwrap_or_else(|| raw(value))
}

/// Decode one line of the child's stdout.
///
/// # Errors
/// [`DecodeError::Blank`] for an empty line, [`DecodeError::Malformed`] when the line is not JSON
/// or not a JSON object.
pub fn decode_line(line: &str) -> Result<Event, DecodeError> {
    let trimmed = line.trim();
    if trimmed.is_empty() {
        return Err(DecodeError::Blank);
    }
    match serde_json::from_str::<Value>(trimmed) {
        Ok(value) if value.is_object() => Ok(classify(value)),
        Ok(_) => Err(DecodeError::Malformed("not a JSON object".to_string())),
        Err(err) => Err(DecodeError::Malformed(err.to_string())),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::protocol::Block;

    // test-tag: critical
    #[test]
    fn init_carries_session_and_key_source() {
        let line = r#"{"type":"system","subtype":"init","session_id":"s1","apiKeySource":"ANTHROPIC_API_KEY","model":"sonnet","tools":["Bash"],"mcp_servers":[{"name":"x"}],"cwd":"X:/"}"#;
        let Ok(Event::Init(init)) = decode_line(line) else {
            panic!("not init");
        };
        assert_eq!(init.session_id, "s1");
        assert_eq!(init.api_key_source.as_deref(), Some("ANTHROPIC_API_KEY"));
        assert_eq!(init.tools, vec!["Bash"]);
        assert_eq!(init.mcp_servers.len(), 1);
        assert!(init.extra.contains_key("cwd"));
    }

    #[test]
    fn assistant_text_and_tool_use() {
        let line = r#"{"type":"assistant","session_id":"s1","parent_tool_use_id":null,"message":{"role":"assistant","content":[{"type":"text","text":"hi"},{"type":"tool_use","id":"t1","name":"Bash","input":{"command":"ls"}}]}}"#;
        let Ok(Event::Assistant(msg)) = decode_line(line) else {
            panic!("not assistant");
        };
        assert_eq!(msg.session_id, "s1");
        assert_eq!(msg.blocks.len(), 2);
        assert_eq!(msg.blocks[0], Block::Text { text: "hi".into() });
        let Block::ToolUse { id, name, input } = &msg.blocks[1] else {
            panic!("not tool_use");
        };
        assert_eq!((id.as_str(), name.as_str()), ("t1", "Bash"));
        assert_eq!(input["command"], "ls");
    }

    #[test]
    fn user_tool_result() {
        let line = r#"{"type":"user","session_id":"s1","message":{"role":"user","content":[{"type":"tool_result","tool_use_id":"t1","content":"ok","is_error":true}]}}"#;
        let Ok(Event::User(msg)) = decode_line(line) else {
            panic!("not user");
        };
        let Block::ToolResult {
            tool_use_id,
            content,
            is_error,
        } = &msg.blocks[0]
        else {
            panic!("not tool_result");
        };
        assert_eq!(tool_use_id, "t1");
        assert_eq!(content, "ok");
        assert!(*is_error);
    }

    #[test]
    fn user_string_content_and_unknown_block() {
        let line = r#"{"type":"user","message":{"content":"plain"}}"#;
        let Ok(Event::User(msg)) = decode_line(line) else {
            panic!("not user");
        };
        assert_eq!(
            msg.blocks,
            vec![Block::Text {
                text: "plain".into()
            }]
        );
        let odd = r#"{"type":"assistant","message":{"content":[{"type":"image","x":1}]}}"#;
        let Ok(Event::Assistant(msg)) = decode_line(odd) else {
            panic!("not assistant");
        };
        assert!(matches!(msg.blocks[0], Block::Unknown(_)));
    }

    #[test]
    fn stream_event_keeps_payload() {
        let line = r#"{"type":"stream_event","event":{"type":"content_block_delta","delta":{"text":"a"}}}"#;
        let Ok(Event::Stream(payload)) = decode_line(line) else {
            panic!("not stream");
        };
        assert_eq!(payload["type"], "content_block_delta");
    }

    #[test]
    fn assistant_carries_its_usage() {
        let line = r#"{"type":"assistant","message":{"content":[],"usage":{"input_tokens":3,"cache_read_input_tokens":40,"cache_creation_input_tokens":5}}}"#;
        let Ok(Event::Assistant(msg)) = decode_line(line) else {
            panic!("not assistant");
        };
        let usage = msg.usage.expect("usage");
        assert_eq!(usage.cache_creation_input_tokens, 5);
        let bare = r#"{"type":"assistant","message":{"content":[]}}"#;
        let Ok(Event::Assistant(msg)) = decode_line(bare) else {
            panic!("not assistant");
        };
        assert_eq!(msg.usage, None);
    }

    #[test]
    fn result_carries_cost_and_usage() {
        let line = r#"{"type":"result","subtype":"success","is_error":false,"session_id":"s1","result":"done","total_cost_usd":0.0123,"duration_ms":900,"num_turns":2,"usage":{"input_tokens":10,"output_tokens":5,"cache_read_input_tokens":7},"future":true}"#;
        let Ok(Event::Result(out)) = decode_line(line) else {
            panic!("not result");
        };
        assert_eq!(out.subtype, "success");
        assert!(!out.is_error);
        assert_eq!(out.total_cost_usd, Some(0.0123));
        let usage = out.usage.expect("usage");
        assert_eq!((usage.input_tokens, usage.cache_read_input_tokens), (10, 7));
        assert!(out.extra.contains_key("future"));
    }

    #[test]
    fn unknown_type_is_kept_raw() {
        let Ok(Event::Other(raw)) = decode_line(r#"{"type":"rate_limit_event","a":1}"#) else {
            panic!("not other");
        };
        assert_eq!(raw.kind, "rate_limit_event");
        assert_eq!(raw.value["a"], 1);
    }

    #[test]
    fn misshapen_known_type_falls_back_to_raw() {
        let line = r#"{"type":"result","is_error":"nope"}"#;
        assert!(matches!(decode_line(line), Ok(Event::Other(_))));
    }

    #[test]
    fn malformed_and_blank_lines() {
        assert!(matches!(
            decode_line(r#"{"type":"res"#),
            Err(DecodeError::Malformed(_))
        ));
        assert!(matches!(decode_line("[1]"), Err(DecodeError::Malformed(_))));
        assert_eq!(decode_line("  \r\n"), Err(DecodeError::Blank));
    }
}
