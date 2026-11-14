//! The line written to the child's stdin.

use serde_json::json;

/// One user message as a single stream-json line, with the trailing newline.
#[must_use]
pub fn encode_user_message(text: &str) -> String {
    let value = json!({
        "type": "user",
        "message": {
            "role": "user",
            "content": [{ "type": "text", "text": text }],
        },
    });
    format!("{value}\n")
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::Value;

    #[test]
    fn user_message_is_one_json_line() {
        let line = encode_user_message("hello\nworld \"q\"");
        assert!(line.ends_with('\n'));
        assert_eq!(line.trim_end().lines().count(), 1);
        let value: Value = serde_json::from_str(line.trim_end()).expect("json");
        assert_eq!(value["type"], "user");
        assert_eq!(value["message"]["role"], "user");
        assert_eq!(value["message"]["content"][0]["type"], "text");
        assert_eq!(value["message"]["content"][0]["text"], "hello\nworld \"q\"");
    }
}
