//! The file name of a session transcript.

const PREFIX: &str = "transcript-";
const SUFFIX: &str = ".jsonl";
const LONGEST: usize = 128;

/// Whether `id` is safe in a file name: letters, digits, `-` and `_` only.
#[must_use]
pub fn valid_session(id: &str) -> bool {
    !id.is_empty()
        && id.len() <= LONGEST
        && id
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_')
}

/// `transcript-<id>.jsonl`, or `None` for an id that is not safe.
#[must_use]
pub fn file_name(session: &str) -> Option<String> {
    valid_session(session).then(|| format!("{PREFIX}{session}{SUFFIX}"))
}

/// The session id of a transcript file name, or `None` for any other file.
#[must_use]
pub fn session_of(file: &str) -> Option<String> {
    let id = file.strip_prefix(PREFIX)?.strip_suffix(SUFFIX)?;
    valid_session(id).then(|| id.to_owned())
}
