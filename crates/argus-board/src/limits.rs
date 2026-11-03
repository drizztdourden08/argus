//! Length limits for every free-text field on the board, and the checks that use them. The AI
//! side keeps these short limits; the owner's own text has only the safety net
//! [`OWNER_TEXT_MAX`]. The numbers live in the item type registry (`limits` in
//! `registry/item-types.json`); this module reads them from there.

use argus_core::js;
use serde_json::Value;

use crate::error::{BoardError, refuse};
use crate::registry::registry;

/// The most characters the owner's own posts may hold (a safety net against a pasted file).
pub const OWNER_TEXT_MAX: usize = 50_000;

const CONTEXT_MIN: usize = 20;
const CONTEXT_HELP: &str = "--context is required: in 20 to 600 chars say what this is about in plain words for someone who has not followed the work, why it needs the owner, and what you will assume if it is not answered. Example: --context \"The login form can remember the email. That changes what is stored on the user's machine, so it is your call. If you do not answer I will not store anything.\"";

/// The limit of a kind of text (`limits[kind]` of the registry), when it has one.
#[must_use]
pub fn limit(kind: &str) -> Option<usize> {
    registry().limit(kind)
}

/// The trimmed text, refused when empty or longer than `max` (or the kind's own limit).
pub(crate) fn check(kind: &str, text: &str, max: Option<usize>) -> Result<String, BoardError> {
    let text = js::trim(text);
    if text.is_empty() {
        return Err(refuse(format!("{kind} text is required.")));
    }
    let len = js::len(text);
    match max.or_else(|| limit(kind)) {
        Some(max) if max > 0 && len > max => Err(refuse(format!(
            "{kind} is {len} chars, the limit is {max}. Shorten it."
        ))),
        _ => Ok(text.to_owned()),
    }
}

/// `requireText`: a non-empty text within the kind's limit.
///
/// # Errors
/// Refused when the value is not a text, is empty, or is over the limit.
pub fn require_text(kind: &str, value: &Value) -> Result<String, BoardError> {
    check(kind, value.as_str().unwrap_or(""), None)
}

/// `requireText` with an explicit cap in place of the kind's limit (the owner's cap).
///
/// # Errors
/// Refused when the value is not a text, is empty, or is over `max`.
pub fn require_text_max(kind: &str, value: &Value, max: usize) -> Result<String, BoardError> {
    check(kind, value.as_str().unwrap_or(""), Some(max))
}

/// `requireText` for text that is already a `&str`.
///
/// # Errors
/// Refused when the text is empty or over the limit.
pub fn require_str(kind: &str, text: &str) -> Result<String, BoardError> {
    check(kind, text, None)
}

/// `optionalText`: the checked text, or `None` for anything blank.
///
/// # Errors
/// Refused when a non-blank text is over the limit.
pub fn optional_text(kind: &str, value: &Value) -> Result<Option<String>, BoardError> {
    match value.as_str() {
        Some(s) if !js::trim(s).is_empty() => check(kind, s, None).map(Some),
        _ => Ok(None),
    }
}

/// `textList`: a list of short texts, each checked, at most `max` of them.
///
/// # Errors
/// Refused when an entry fails its check or the list is longer than `max`.
pub fn text_list(kind: &str, values: &Value, max: usize) -> Result<Vec<String>, BoardError> {
    let list = match values {
        Value::Null => Vec::new(),
        Value::Array(items) => items
            .iter()
            .map(|v| require_text(kind, v))
            .collect::<Result<Vec<_>, _>>()?,
        _ => return Err(refuse(format!("{kind} must be a list."))),
    };
    if list.len() > max {
        return Err(refuse(format!(
            "{} {kind} entries, the limit is {max}.",
            list.len()
        )));
    }
    Ok(list)
}

/// `requireContext`: the context a question or decision to the owner carries.
///
/// # Errors
/// Refused with the teaching message when it is missing, too short or too long.
pub fn require_context(value: &Value) -> Result<String, BoardError> {
    let text = js::trim(value.as_str().unwrap_or(""));
    if text.is_empty() {
        return Err(refuse(CONTEXT_HELP));
    }
    let len = js::len(text);
    if len < CONTEXT_MIN {
        return Err(refuse(format!(
            "--context is {len} chars, the minimum is {CONTEXT_MIN}. {CONTEXT_HELP}"
        )));
    }
    check("context", text, None)
}

/// `softContext`: the optional context of a notice or message, clipped to its limit.
#[must_use]
pub fn soft_context(value: &Value) -> Option<String> {
    let text = js::trim(value.as_str()?);
    if text.is_empty() {
        return None;
    }
    Some(js::slice(text, limit("context").unwrap_or(600)).to_owned())
}

// test-tag: critical
#[cfg(test)]
mod tests {
    use super::*;

    /// Every limit the board had before the registry held them, with its number.
    const BEFORE: [(&str, usize); 45] = [
        ("name", 40),
        ("subject", 40),
        ("note", 100),
        ("title", 80),
        ("description", 400),
        ("ac", 120),
        ("question", 240),
        ("option", 60),
        ("answer", 600),
        ("followup", 240),
        ("lead", 240),
        ("ping", 240),
        ("decision", 200),
        ("notice", 140),
        ("message", 240),
        ("action", 200),
        ("log", 120),
        ("brief", 1200),
        ("task", 120),
        ("covers", 80),
        ("context", 600),
        ("changed", 240),
        ("label", 40),
        ("setup", 300),
        ("command", 300),
        ("step", 240),
        ("expect", 200),
        ("testnote", 300),
        ("caption", 120),
        ("result", 300),
        ("reason", 120),
        ("model", 40),
        ("modelwhy", 200),
        ("branch", 80),
        ("worktree", 120),
        ("agentnote", 100),
        ("agentresult", 200),
        ("build", 40),
        ("buildnote", 600),
        ("buildcmd", 400),
        ("buildcwd", 260),
        ("launchercmd", 400),
        ("launchercwd", 260),
        ("launchernote", 300),
        ("commit", 40),
    ];

    #[test]
    fn every_existing_limit_keeps_its_number_and_none_is_added() {
        for (kind, n) in BEFORE {
            assert_eq!(limit(kind), Some(n), "{kind}");
        }
        assert_eq!(registry().limits.len(), BEFORE.len());
        assert_eq!(limit("nothing"), None);
    }

    #[test]
    fn a_field_of_the_registry_agrees_with_the_limit_of_the_same_text() {
        let same = [
            ("title", "title"),
            ("ac", "ac"),
            ("subject", "subject"),
            ("question", "question"),
            ("option", "option"),
            ("decision", "decision"),
            ("context", "context"),
            ("caption", "caption"),
            ("reason", "reason"),
            ("setup", "setup"),
            ("steps", "step"),
            ("pass", "expect"),
            ("fail", "expect"),
            ("resultNote", "result"),
        ];
        for (field, kind) in same {
            let max = registry().field(field).and_then(|f| f.max);
            assert_eq!(max, limit(kind), "{field} against {kind}");
        }
    }

    #[test]
    fn the_text_checks_still_refuse_over_the_registry_limit() {
        let err = require_str("title", &"a".repeat(81))
            .unwrap_err()
            .to_string();
        assert_eq!(err, "title is 81 chars, the limit is 80. Shorten it.");
        assert_eq!(require_str("title", &"a".repeat(80)).unwrap().len(), 80);
    }
}
