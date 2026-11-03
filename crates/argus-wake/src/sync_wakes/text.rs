//! The words of the remote sync wake.

use argus_core::clock::age_text;

/// How long ago the last sync was: `25h ago`, or `never` when the project has no record.
#[must_use]
pub fn ago_text(age_ms: Option<i64>) -> String {
    age_ms.map_or_else(|| "never".to_owned(), |ms| format!("{} ago", age_text(ms)))
}

/// The line the lead hears.
#[must_use]
pub fn due_text(ago: &str) -> String {
    format!("remote sync due: last {ago}, run the full sync, then coord remote synced")
}
