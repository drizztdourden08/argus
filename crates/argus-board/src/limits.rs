//! Length limits for every free-text field on the board, and the checks that use them. The AI
//! side keeps these short limits; the owner's own text has only the safety net
//! [`OWNER_TEXT_MAX`].

use argus_core::js;
use serde_json::Value;

use crate::error::{BoardError, refuse};

const LIMITS: [(&str, usize); 45] = [
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

/// The most characters the owner's own posts may hold (a safety net against a pasted file).
pub const OWNER_TEXT_MAX: usize = 50_000;

const CONTEXT_MIN: usize = 20;
const CONTEXT_HELP: &str = "--context is required: in 20 to 600 chars say what this is about in plain words for someone who has not followed the work, why it needs the owner, and what you will assume if it is not answered. Example: --context \"The login form can remember the email. That changes what is stored on the user's machine, so it is your call. If you do not answer I will not store anything.\"";

/// The limit of a kind of text (`LIMITS[kind]`), when it has one.
#[must_use]
pub fn limit(kind: &str) -> Option<usize> {
    LIMITS.iter().find(|(k, _)| *k == kind).map(|(_, n)| *n)
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
