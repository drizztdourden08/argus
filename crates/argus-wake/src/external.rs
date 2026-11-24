//! The checks of the wake routes that need no HTTP: who may post a wake from outside the policy, and who
//! may clear a fault. The server slice maps a [`RequestError`] to its status code.

use argus_core::ids::is_key;
use argus_core::lead::canonical;
use serde_json::Value;

use crate::board::BoardView;
use crate::reasons::EXTERNAL_REASONS;
use crate::registry::{NOTE_MAX, NOTE_MIN};
use crate::text::{cut, one_line};
use crate::wake::WakeInput;

pub use crate::overlord::overlord_only;

/// The longest text an outside wake carries.
pub const MAX_WAKE_TEXT: usize = 300;
/// The longest label a listener announces.
pub const MAX_LABEL: usize = 80;
/// The longest tab id a presence stream announces.
pub const MAX_TAB: usize = 40;

/// Why a request was refused.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RequestError {
    /// 400.
    BadRequest(String),
    /// 403.
    Forbidden(String),
    /// 404.
    NotFound(String),
}

impl RequestError {
    /// The HTTP status.
    #[must_use]
    pub fn status(&self) -> u16 {
        match self {
            Self::BadRequest(_) => 400,
            Self::Forbidden(_) => 403,
            Self::NotFound(_) => 404,
        }
    }

    /// The message for the error body.
    #[must_use]
    pub fn message(&self) -> &str {
        match self {
            Self::BadRequest(m) | Self::Forbidden(m) | Self::NotFound(m) => m,
        }
    }
}

fn bad(message: &str) -> RequestError {
    RequestError::BadRequest(message.to_owned())
}

/// A thread id: 1 to 60 of `A-Za-z0-9._~:@+-`.
#[must_use]
pub fn valid_thread_id(thread: &str) -> bool {
    (1..=60).contains(&thread.chars().count())
        && thread
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || "._~:@+-".contains(c))
}

/// A thread key a stream listens on: `*` or a thread id.
#[must_use]
pub fn valid_thread_key(thread: &str) -> bool {
    thread == "*" || valid_thread_id(thread)
}

/// The thread of a stream request: `*` when none was given.
///
/// # Errors
/// When the thread is neither `*` nor a thread id.
pub fn stream_thread(raw: Option<&str>) -> Result<String, RequestError> {
    let thread = raw.filter(|t| !t.is_empty()).unwrap_or("*");
    if valid_thread_key(thread) {
        Ok(canonical(thread).into_owned())
    } else {
        Err(bad("thread must be a thread id or *"))
    }
}

/// The thread of a presence request: a thread id, never `*`.
///
/// # Errors
/// When the thread is not a thread id.
pub fn presence_thread(raw: Option<&str>) -> Result<String, RequestError> {
    let thread = stream_thread(raw)?;
    if thread == "*" {
        Err(bad("thread must be a thread id"))
    } else {
        Ok(thread)
    }
}

/// The project of a request.
///
/// # Errors
/// When the key is not a project key.
pub fn scope_of(key: &str) -> Result<&str, RequestError> {
    if is_key(key) {
        Ok(key)
    } else {
        Err(bad(&format!("invalid project key \"{key}\"")))
    }
}

fn is_separator(c: char) -> bool {
    c.is_control() || c.is_whitespace()
}

/// A listener label: control characters and runs of white space become one space, 80 characters.
#[must_use]
pub fn clean_label(raw: &str) -> Option<String> {
    let spaced: String = raw
        .chars()
        .map(|c| if is_separator(c) { ' ' } else { c })
        .collect();
    let label: String = one_line(&spaced).chars().take(MAX_LABEL).collect();
    (!label.is_empty()).then_some(label)
}

/// A presence tab id: control characters and white space removed, 40 characters, `tab` when nothing is left.
#[must_use]
pub fn clean_tab(raw: &str) -> String {
    let tab: String = raw
        .chars()
        .filter(|c| !is_separator(*c))
        .take(MAX_TAB)
        .collect();
    if tab.is_empty() {
        "tab".to_owned()
    } else {
        tab
    }
}

/// The message of a project that is over its listener cap.
#[must_use]
pub fn listener_cap_message(scope: &str) -> String {
    format!(
        "too many wake listeners on {scope} (max {})",
        crate::engine::MAX_LISTENERS_PER_SCOPE
    )
}

fn text_field<'a>(body: &'a Value, key: &str) -> Option<&'a str> {
    body.get(key).and_then(Value::as_str)
}

/// A wake posted from outside the policy (`coord lead`, `coord inbox send`): the body is checked and the wake made.
/// `known` says whether a thread exists on the board of the project.
///
/// # Errors
/// 400 for a reason, thread or text that is not acceptable; 404 for a thread the board does not have.
pub fn external_wake(
    scope: &str,
    body: &Value,
    known: &dyn Fn(&str, &str) -> bool,
) -> Result<WakeInput, RequestError> {
    let reason = text_field(body, "reason").filter(|r| EXTERNAL_REASONS.contains(r));
    let Some(reason) = reason else {
        return Err(bad(&format!(
            "reason must be one of {}",
            EXTERNAL_REASONS.join(", ")
        )));
    };
    let Some(thread) = text_field(body, "thread").filter(|t| valid_thread_id(t)) else {
        return Err(bad("thread must be a thread id"));
    };
    let thread = canonical(thread);
    let thread = &*thread;
    let Some(text) = text_field(body, "text").filter(|t| !t.trim().is_empty()) else {
        return Err(bad("text is required"));
    };
    if !known(scope, thread) {
        return Err(RequestError::NotFound(format!(
            "no thread \"{thread}\" on {scope}"
        )));
    }
    Ok(WakeInput::about(
        scope,
        thread,
        reason,
        thread,
        cut(text, MAX_WAKE_TEXT),
    ))
}

/// A checked request to clear a wake fault.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ClearRequest {
    /// Project key.
    pub project: String,
    /// The thread.
    pub thread: String,
    /// Why it is cleared.
    pub note: String,
    /// Who clears it.
    pub by: String,
}

/// Check a request to clear a wake fault: the shape, the note, and that `by` is a live lead of the portfolio board.
/// (A browser request is refused before this, by the route.)
///
/// # Errors
/// 400 for a bad project, thread or note; 403 when `by` is not the Overlord (`overlord_board` is the id of his board).
pub fn clear_request(
    body: &Value,
    overlord_board: &str,
    portfolio: Option<&BoardView>,
) -> Result<ClearRequest, RequestError> {
    let project = text_field(body, "project")
        .filter(|p| is_key(p))
        .ok_or_else(|| bad("project must be a project key"))?;
    let thread = text_field(body, "thread")
        .filter(|t| valid_thread_id(t))
        .ok_or_else(|| bad("thread must be a thread id"))?;
    let note = one_line(text_field(body, "note").unwrap_or(""));
    let length = note.chars().count();
    if !(NOTE_MIN..=NOTE_MAX).contains(&length) {
        return Err(bad(&format!(
            "note must be {NOTE_MIN} to {NOTE_MAX} characters: say what was wrong and what you did"
        )));
    }
    let by = text_field(body, "by").unwrap_or("");
    overlord_only(
        &canonical(by),
        overlord_board,
        portfolio,
        "clear a wake fault",
    )?;
    Ok(ClearRequest {
        project: project.to_owned(),
        thread: canonical(thread).into_owned(),
        note,
        by: canonical(by).into_owned(),
    })
}

/// The message of a clear that found nothing open.
#[must_use]
pub fn no_open_fault(project: &str, thread: &str) -> RequestError {
    RequestError::NotFound(format!("no open wake fault for {project}/{thread}"))
}
