//! The checks of the fields a pull request op carries.

use argus_core::js;
use serde_json::Value;

use crate::Board;
use crate::error::{BoardError, refuse};
use crate::json::{display, doc, shown};
use crate::limits::require_text_max;

const TITLE_MAX: usize = 200;
const BODY_MAX: usize = 20_000;
const BRANCH_MAX: usize = 120;
const COMMIT_MIN: usize = 7;
const COMMIT_MAX: usize = 40;

/// The title: a non-empty text.
pub(super) fn title(value: &Value) -> Result<String, BoardError> {
    require_text_max("title", value, TITLE_MAX)
}

/// The body: a text, empty when missing.
pub(super) fn body(value: &Value) -> Result<String, BoardError> {
    match value {
        Value::Null => Ok(String::new()),
        Value::String(s) if js::len(s) <= BODY_MAX => Ok(js::trim(s).to_owned()),
        Value::String(s) => Err(refuse(format!(
            "body is {} chars, the limit is {BODY_MAX}. Shorten it.",
            js::len(s)
        ))),
        _ => Err(refuse("body must be a text.")),
    }
}

fn branch_chars(name: &str) -> bool {
    name.chars()
        .all(|c| c.is_ascii_alphanumeric() || "._/-".contains(c))
}

fn branch_shape(name: &str) -> bool {
    !name.starts_with(['-', '/'])
        && !name.ends_with(['/', '.'])
        && !name.contains("..")
        && !name.contains("//")
}

/// A branch name: plain characters only, so it is safe to hand to git.
pub(super) fn branch(what: &str, value: &Value) -> Result<String, BoardError> {
    let name = value.as_str().map_or("", js::trim);
    let ok =
        !name.is_empty() && js::len(name) <= BRANCH_MAX && branch_chars(name) && branch_shape(name);
    if ok {
        Ok(name.to_owned())
    } else {
        Err(refuse(format!(
            "{what} must be a branch name (letters, digits, . _ / -)."
        )))
    }
}

/// A commit: 7 to 40 hex characters, lower cased. Missing, null and blank are `None`.
pub(super) fn commit(what: &str, value: &Value) -> Result<Option<String>, BoardError> {
    let text = match value {
        Value::Null => return Ok(None),
        Value::String(s) => js::trim(s),
        _ => return Err(refuse(format!("{what} must be a commit hash."))),
    };
    if text.is_empty() {
        return Ok(None);
    }
    let sized = (COMMIT_MIN..=COMMIT_MAX).contains(&text.len());
    if sized && text.bytes().all(|b| b.is_ascii_hexdigit()) {
        Ok(Some(text.to_ascii_lowercase()))
    } else {
        Err(refuse(format!(
            "{what} must be a commit hash ({COMMIT_MIN} to {COMMIT_MAX} hex characters)."
        )))
    }
}

fn numbered(id: &str, prefix: &str) -> bool {
    id.strip_prefix(prefix)
        .is_some_and(|n| !n.is_empty() && n.bytes().all(|b| b.is_ascii_digit()))
}

/// A check run id, `C-n`.
pub(super) fn check_id(value: &Value) -> Result<String, BoardError> {
    let id = value.as_str().unwrap_or_default();
    if numbered(id, "C-") {
        Ok(id.to_owned())
    } else {
        Err(refuse("checks must be a list of check run ids like C-3."))
    }
}

/// The id of the pull request an op points at, which must be on the board.
pub(super) fn existing(board: &Board, op: &Value) -> Result<String, BoardError> {
    let id = shown(op["target"].get("id"));
    if numbered(&id, "PR-") && doc(board, "prs", &id).is_some() {
        Ok(id)
    } else {
        Err(refuse(format!("no pull request \"{id}\".")))
    }
}

/// The id of the open pull request an op points at.
pub(super) fn open(board: &Board, op: &Value) -> Result<String, BoardError> {
    let id = existing(board, op)?;
    let status = display(&board["prs"][id.as_str()]["status"]);
    if status == "open" {
        Ok(id)
    } else {
        Err(refuse(format!(
            "{id} is {status}; only an open pull request changes."
        )))
    }
}
