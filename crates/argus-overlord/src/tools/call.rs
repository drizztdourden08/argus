//! The arguments of the Overlord's own tools, checked. A bad argument is a sentence the child reads.

use serde_json::Value;

use crate::rules::{RuleInput, RulePatch, parse_input, parse_patch};

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
    /// `rules_list`.
    RulesList,
    /// `rule_add`: a timed rule made by the Overlord.
    RuleAdd { input: RuleInput },
    /// `rule_update`: the fields given change, the rest stay.
    RuleUpdate { id: String, change: RulePatch },
    /// `rule_remove`.
    RuleRemove { id: String },
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

fn rule_update(args: &Value) -> Result<Call, String> {
    Ok(Call::RuleUpdate {
        id: word(args, "id")?,
        change: parse_patch(args)?,
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
        "rules_list" => Some(Ok(Call::RulesList)),
        "rule_add" => Some(parse_input(args).map(|input| Call::RuleAdd { input })),
        "rule_update" => Some(rule_update(args)),
        "rule_remove" => Some(word(args, "id").map(|id| Call::RuleRemove { id })),
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
        let call = parse_call("thread_message", &args)
            .expect("ours")
            .expect("ok");
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
        let fault = parse_call("session_message", &args)
            .expect("ours")
            .unwrap_err();
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

    #[test]
    fn a_rule_add_reads_its_input_and_a_bad_one_names_the_field() {
        let args = json!({ "title": "Sweep", "prompt": "look", "everyMin": 30 });
        let call = parse_call("rule_add", &args).expect("ours").expect("ok");
        let Call::RuleAdd { input } = call else {
            panic!("a rule add");
        };
        assert_eq!(input.every_min, 30);
        assert!(input.enabled);
        let args = json!({ "title": "Sweep", "prompt": "look", "everyMin": "soon" });
        let fault = parse_call("rule_add", &args).expect("ours").unwrap_err();
        assert_eq!(fault, "everyMin must be a whole number");
    }

    #[test]
    fn a_rule_update_and_a_remove_need_an_id() {
        let args = json!({ "id": "OR-2", "enabled": false });
        let call = parse_call("rule_update", &args).expect("ours").expect("ok");
        let Call::RuleUpdate { id, change } = call else {
            panic!("a rule update");
        };
        assert_eq!(id, "OR-2");
        assert_eq!(change.enabled, Some(false));
        let fault = parse_call("rule_remove", &json!({}))
            .expect("ours")
            .unwrap_err();
        assert_eq!(fault, "id is required, as text");
    }
}
