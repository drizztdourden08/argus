//! The Overlord's own tools on its private MCP endpoint (I-192): message or wake a thread, list the
//! desktop sessions, resume one. This is the pure side (the arguments, the listing, the result shape);
//! the server reaches the boards, the wake engine and the CLI.

mod call;
mod schema;

pub use call::{Call, MAX_TEXT, parse_call};
pub use schema::specs;

use serde_json::{Value, json};

/// The JSON-RPC answer of a tool call: `text` as the tool's text content, flagged as an error when
/// the call could not do what it was asked.
#[must_use]
pub fn text_result(id: &Value, text: &str, is_error: bool) -> Value {
    json!({
        "jsonrpc": "2.0",
        "id": id,
        "result": { "content": [{ "type": "text", "text": text }], "isError": is_error }
    })
}

// test-tag: critical
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_result_carries_the_text_and_the_error_flag() {
        let answer = text_result(&json!(7), "done", false);
        assert_eq!(answer["id"], 7);
        assert_eq!(answer["result"]["content"][0]["text"], "done");
        assert_eq!(answer["result"]["isError"], false);
        assert_eq!(
            text_result(&json!(7), "no", true)["result"]["isError"],
            true
        );
    }
}
