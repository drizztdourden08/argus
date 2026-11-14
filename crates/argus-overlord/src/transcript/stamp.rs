//! Which transcript files stay. The modified times arrive as data, so the rule is pure.

/// The last sessions kept on disk.
pub const KEEP_SESSIONS: usize = 10;

/// A session file and when it was last written (milliseconds since the epoch).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Stamp {
    pub session: String,
    pub modified_ms: u64,
}

fn newest_first(files: &[Stamp]) -> Vec<&Stamp> {
    let mut sorted: Vec<&Stamp> = files.iter().collect();
    sorted.sort_by(|a, b| (b.modified_ms, &b.session).cmp(&(a.modified_ms, &a.session)));
    sorted
}

/// The sessions to delete so only the `keep` most recently written stay.
#[must_use]
pub fn stale(files: &[Stamp], keep: usize) -> Vec<String> {
    newest_first(files)
        .into_iter()
        .skip(keep)
        .map(|s| s.session.clone())
        .collect()
}

/// The most recently written session.
#[must_use]
pub fn newest(files: &[Stamp]) -> Option<String> {
    newest_first(files).first().map(|s| s.session.clone())
}
