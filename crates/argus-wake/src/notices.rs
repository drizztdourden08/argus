//! The passive notices of a thread: the rows the wake policy kept off the listener, counted by
//! reason with the newest few, and the one digest line the prompt hook prints.

use serde::Serialize;

use crate::text::clip;
use crate::time::ms_or_zero;
use crate::wake::{WakeRow, WakeStatus};

/// How many of the newest notices the digest names.
pub const NEWEST: usize = 3;
/// How long each named notice may be in the digest line.
const NOTICE_CLIP: usize = 80;

/// How many notices of one reason.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct KindCount {
    /// The reason.
    pub reason: String,
    /// How many.
    pub count: u32,
}

/// One notice.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Notice {
    /// `W-n`.
    pub id: String,
    /// The thread it was for.
    pub thread: String,
    /// Its reason.
    pub reason: String,
    /// Its text.
    pub text: String,
    /// When it was made (ISO).
    pub at: String,
}

/// The notices since a moment.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct NoticeDigest {
    /// How many.
    pub count: u32,
    /// How many of each reason, the most first.
    pub kinds: Vec<KindCount>,
    /// The newest few, newest first.
    pub newest: Vec<Notice>,
    /// The line the prompt hook prints; `None` with no notice.
    pub line: Option<String>,
}

fn notice_of(row: &WakeRow) -> Notice {
    Notice {
        id: row.id.clone(),
        thread: row.thread.clone(),
        reason: row.reason.clone(),
        text: row.text.clone(),
        at: row.at.clone(),
    }
}

fn kinds_of(rows: &[&WakeRow]) -> Vec<KindCount> {
    let mut kinds: Vec<KindCount> = Vec::new();
    for row in rows {
        match kinds.iter_mut().find(|k| k.reason == row.reason) {
            Some(kind) => kind.count += 1,
            None => kinds.push(KindCount {
                reason: row.reason.clone(),
                count: 1,
            }),
        }
    }
    kinds.sort_by(|a, b| b.count.cmp(&a.count).then_with(|| a.reason.cmp(&b.reason)));
    kinds
}

/// The digest line: `Board notices since your last turn: 3 (kanban-stale 2, worktree-review 1).
/// Newest: "..."; "...". Not wakes: the board header lists them.`
#[must_use]
pub fn digest_line(count: u32, kinds: &[KindCount], newest: &[Notice]) -> Option<String> {
    if count == 0 {
        return None;
    }
    let by_kind: Vec<String> = kinds
        .iter()
        .map(|k| format!("{} {}", k.reason, k.count))
        .collect();
    let named: Vec<String> = newest
        .iter()
        .map(|n| format!("\"{}\"", clip(&n.text, NOTICE_CLIP)))
        .collect();
    Some(format!(
        "Board notices since your last turn: {count} ({}). Newest: {}. Not wakes: the board header lists them.",
        by_kind.join(", "),
        named.join("; ")
    ))
}

/// The passive notices among `rows` (newest first, as [`crate::store::WakeStore::rows`] lists
/// them) made after `since_ms`.
#[must_use]
pub fn digest(rows: &[WakeRow], since_ms: i64) -> NoticeDigest {
    let passive: Vec<&WakeRow> = rows
        .iter()
        .filter(|r| r.status == WakeStatus::Passive && ms_or_zero(Some(&r.at)) > since_ms)
        .collect();
    let count = u32::try_from(passive.len()).unwrap_or(u32::MAX);
    let kinds = kinds_of(&passive);
    let newest: Vec<Notice> = passive
        .iter()
        .copied()
        .take(NEWEST)
        .map(notice_of)
        .collect();
    let line = digest_line(count, &kinds, &newest);
    NoticeDigest {
        count,
        kinds,
        newest,
        line,
    }
}
