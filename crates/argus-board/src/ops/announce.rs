//! Telling people what changed: the audit line, the inbox line to each owner, and the
//! "needs the human" flag of a thread.

use super::result::Notice;
use super::who::Who;
use crate::Board;
use crate::inbox::queue;
use crate::json::{doc, doc_mut, find_ids, set, slot, truthy};
use crate::store::{LogLine, append_log, touch};
use crate::text::clip_text;

const AUDIT_LINE: usize = 120;
const NOTICE_LINE: usize = 240;

/// One audit line (clipped to the log limit).
pub(crate) fn audit(board: &mut Board, actor: &str, text: &str) {
    let line = clip_text(text, AUDIT_LINE);
    append_log(
        board,
        "audit",
        &LogLine {
            thread: actor,
            item: None,
            text: &line,
        },
    );
}

/// A change made by someone other than the owning thread: one audit line and one inbox line to
/// each owner. Returns what was queued. A thread changing its own record leaves no trace here;
/// the owner's changes always do, so the threads see what moved under them.
pub(crate) fn announce(
    board: &mut Board,
    who: &Who,
    owners: &[Option<String>],
    text: &str,
    reference: Option<&str>,
) -> Vec<Notice> {
    let actor = who.actor_of();
    let mut to: Vec<&str> = Vec::new();
    for owner in owners.iter().flatten().filter(|o| !o.is_empty()) {
        if *owner != actor && owner != "human" && !to.contains(&owner.as_str()) {
            to.push(owner);
        }
    }
    if to.is_empty() && !who.is_human() {
        return Vec::new();
    }
    let line = clip_text(&format!("{} {text}", who.label()), NOTICE_LINE);
    audit(board, &actor, &line);
    to.into_iter()
        .filter(|o| queue(board, o, &actor, "change", &line, reference))
        .map(|o| Notice {
            to: o.to_owned(),
            text: line.clone(),
        })
        .collect()
}

/// A thread needs the human exactly while it has a pending action that is neither archived nor
/// in the trash.
pub(crate) fn refresh_needs_human(board: &mut Board, thread: &str) {
    if doc(board, "threads", thread).is_none() {
        return;
    }
    let needs = !find_ids(board, "actions", |a| {
        a["thread"].as_str() == Some(thread)
            && a["status"] == "pending"
            && !truthy(a, "deletedAt")
            && !truthy(a, "archivedAt")
    })
    .is_empty();
    let Some(t) = doc_mut(board, "threads", thread) else {
        return;
    };
    if !t["flags"].is_object() {
        set(
            t,
            "flags",
            serde_json::json!({"needsHuman": false, "blocked": false}),
        );
    }
    if t["flags"]["needsHuman"] == needs {
        return;
    }
    set(slot(t, "flags"), "needsHuman", needs);
    touch(board, "threads", thread);
}

/// The ids of the coordinator threads, only the live ones when asked.
pub(crate) fn coordinator_ids(board: &Board, live_only: bool) -> Vec<String> {
    find_ids(board, "threads", |t| {
        t["role"] == "coordinator"
            && (!live_only || !matches!(t["status"].as_str(), Some("done" | "archived")))
    })
}
