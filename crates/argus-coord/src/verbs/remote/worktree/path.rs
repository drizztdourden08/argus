//! The paths of the worktree verbs: what a rule path may be, and how a worktree is named (its
//! id of ten hex characters, or its exact path). The forms shared with the dashboard come from
//! `argus-worktrees`.

use argus_core::js;
use argus_worktrees::{clean_path, worktree_id};

use crate::cli::exit::CoordError;

const MAX_PATH: usize = 300;
const BAD_CHARS: &str = "?<>|\"[]{}";

/// The file system ignores case here, as on Windows.
#[must_use]
pub fn ignores_case() -> bool {
    cfg!(windows)
}

/// The form two paths are compared in.
#[must_use]
pub fn path_key(value: &str) -> String {
    argus_worktrees::path_key(value, ignores_case())
}

/// Any wildcard a rule path may not have.
#[must_use]
pub fn has_wildcard(value: &str) -> bool {
    value.contains(['*', '?', '[', '{'])
}

fn is_control(c: char) -> bool {
    c <= '\u{1f}' || c == '\u{7f}'
}

fn has_drive(path: &str) -> bool {
    let bytes = path.as_bytes();
    bytes.len() >= 3 && bytes[0].is_ascii_alphabetic() && bytes[1] == b':' && bytes[2] == b'/'
}

fn refuse(text: impl Into<String>) -> CoordError {
    CoordError::refuse(text)
}

/// The clean path, or the reason it is refused.
///
/// # Errors
/// The path is missing, too long, not absolute or holds what a folder name cannot have.
pub fn validate_path(value: &str) -> Result<String, CoordError> {
    let raw = js::trim(value);
    if raw.is_empty() {
        return Err(refuse(
            "give the path of the worktrees: the absolute parent folder that holds them (X:/project/.claude/worktrees).",
        ));
    }
    let len = js::len(raw);
    if len > MAX_PATH {
        return Err(refuse(format!(
            "the path is {len} characters, the limit is {MAX_PATH}."
        )));
    }
    if raw.chars().any(is_control) {
        return Err(refuse("the path holds a control character."));
    }
    if raw.starts_with('~') {
        return Err(refuse(
            "~ is not expanded: give the absolute path (X:/Users/you/...).",
        ));
    }
    let path = clean_path(raw);
    let drive = has_drive(&path);
    if !drive && !path.starts_with('/') {
        return Err(refuse(format!(
            "\"{raw}\" is not an absolute path: start with a drive (X:/...) or a slash."
        )));
    }
    check_segments(&path, drive)?;
    Ok(path)
}

fn check_segments(path: &str, drive: bool) -> Result<(), CoordError> {
    let segments: Vec<&str> = path.split('/').filter(|s| !s.is_empty()).collect();
    if segments.iter().any(|s| *s == ".." || *s == ".") {
        return Err(refuse("the path may not hold . or .. segments."));
    }
    let rest = if drive { &segments[1..] } else { &segments[..] };
    if rest.iter().any(|s| s.contains("**")) {
        return Err(refuse(
            "** is not supported: * stands for characters inside ONE path segment.",
        ));
    }
    let joined = rest.join("/");
    let odd_colon = !path.starts_with("//") && joined.contains(':');
    if joined.contains(|c| BAD_CHARS.contains(c)) || odd_colon {
        return Err(refuse(
            "the path holds a character a folder name cannot have (? < > | \" : [ ] { }).",
        ));
    }
    if drive && segments[0].contains('*') {
        return Err(refuse("the drive cannot be a glob."));
    }
    Ok(())
}

/// Ten lower case hex characters: the id of a worktree.
fn is_id(text: &str) -> bool {
    text.len() == 10 && text.bytes().all(|b| matches!(b, b'0'..=b'9' | b'a'..=b'f'))
}

/// What `keep`, `remove` and `prune` take for a worktree: its id, or its path.
///
/// # Errors
/// The path is refused, or names several worktrees (`*`).
pub fn target_of(value: &str) -> Result<(String, Option<String>), CoordError> {
    let text = js::trim(value);
    if is_id(text) {
        return Ok((text.to_owned(), None));
    }
    let path = validate_path(text)?;
    if path.contains('*') {
        return Err(refuse("name one worktree: its id or its exact path, no *."));
    }
    Ok((worktree_id(&path, ignores_case()), Some(path)))
}
