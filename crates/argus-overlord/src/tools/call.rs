//! The arguments of the Overlord's own tools, checked. A bad argument is a sentence the child reads.

use serde_json::Value;

/// The longest text a message or a wake line may carry.
pub const MAX_TEXT: usize = 8_000;

/// One call of an Overlord tool, its arguments read.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Call {
    /// `thread_message`: an inbox note for a thread, then a wake.
    ThreadMessage {
        project: String,
        thread: String,
        text: String,
    },
    /// `thread_wake`: a wake only.
    ThreadWake {
        project: String,
        thread: String,
        reason: String,
    },
    /// `sessions_list`.
    SessionsList,
    /// `session_message`: a headless resume of a desktop session with a text.
    SessionMessage { local_id: String, text: String },
}

fn word(args: &Value, name: &str) -> Result<String, String> {
    match args[name].as_str().map(str::trim) {
        Some(found) if !found.is_empty() => Ok(found.to_owned()),
        _ => Err(format!("{name} is required, as text")),
    }
}

fn text(args: &Value, name: &str) -> Result<String, String> {
    let found = word(args, name)?;
    if found.chars().count() > MAX_TEXT {
        return Err(format!("{name} is longer than {MAX_TEXT} characters"));
    }
    Ok(found)
}

fn thread_message(args: &Value) -> Result<Call, String> {
    Ok(Call::ThreadMessage {
        project: word(args, "project")?,
        thread: word(args, "thread")?,
        text: text(args, "text")?,
    })
}

fn thread_wake(args: &Value) -> Result<Call, String> {
    Ok(Call::ThreadWake {
        project: word(args, "project")?,
        thread: word(args, "thread")?,
        reason: text(args, "reason")?,
    })
}

fn session_message(args: &Value) -> Result<Call, String> {
    Ok(Call::SessionMessage {
        local_id: word(args, "localId")?,
        text: text(args, "text")?,
    })
}

/// The call a tool name and its arguments make. `None` when the name is not an Overlord tool.
#[must_use]
pub fn parse_call(name: &str, args: &Value) -> Option<Result<Call, String>> {
    match name {
        "thread_message" => Some(thread_message(args)),
        "thread_wake" => Some(thread_wake(args)),
        "sessions_list" => Some(Ok(Call::SessionsList)),
        "session_message" => Some(session_message(args)),
        _ => None,
    }
}

// test-tag: critical
#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::*;

    #[test]
    fn a_thread_message_reads_its_three_arguments() {
        let args = json!({ "project": " brock ", "thread": "coordinator", "text": "hi" });
        let call = parse_call("thread_message", &args).expect("ours").expect("ok");
        assert_eq!(
            call,
            Call::ThreadMessage {
                project: "brock".to_owned(),
                thread: "coordinator".to_owned(),
                text: "hi".to_owned()
            }
        );
    }

    #[test]
    fn a_missing_or_blank_argument_names_itself() {
        let args = json!({ "project": "brock", "thread": "  " });
        let fault = parse_call("thread_wake", &args).expect("ours").unwrap_err();
        assert_eq!(fault, "thread is required, as text");
        let args = json!({ "localId": "local_a" });
        let fault = parse_call("session_message", &args).expect("ours").unwrap_err();
        assert_eq!(fault, "text is required, as text");
    }

    #[test]
    fn a_text_past_the_limit_is_refused() {
        let args = json!({ "localId": "local_a", "text": "x".repeat(MAX_TEXT + 1) });
        assert!(parse_call("session_message", &args).expect("ours").is_err());
    }

    #[test]
    fn the_list_takes_no_arguments_and_another_name_is_not_ours() {
        assert_eq!(
            parse_call("sessions_list", &json!({})),
            Some(Ok(Call::SessionsList))
        );
        assert_eq!(parse_call("permission_prompt", &json!({})), None);
    }
}
