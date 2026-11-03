//! Has a thread acknowledged a wake? Reading its inbox after the wake does; so does the thing the wake asked
//! for, and so does the end of the thread.

use super::key::read_key;
use super::sources::StallKv;
use crate::board::{BoardView, ThreadView};
use crate::wake::WakeRow;

/// What the check reads besides the project board.
pub(super) struct Acks<'a> {
    /// The inbox reads.
    pub kv: &'a dyn StallKv,
    /// The dependency ledger, when the portfolio board has one.
    pub ledger: Option<&'a BoardView>,
}

impl Acks<'_> {
    /// Is the wake (made at `at`, epoch ms) acknowledged?
    pub(super) fn covers(&self, board: &BoardView, row: &WakeRow, at: i64) -> bool {
        let gone = board
            .threads
            .get(&row.thread)
            .is_none_or(ThreadView::is_ended);
        gone || self.read_after(row, at) || self.answered(board, row)
    }

    fn read_after(&self, row: &WakeRow, at: i64) -> bool {
        self.kv
            .get(&read_key(&row.scope, &row.thread))
            .and_then(|read| read.parse::<i64>().ok())
            .is_some_and(|read| read > at)
    }

    fn answered(&self, board: &BoardView, row: &WakeRow) -> bool {
        let subject = row.subject.as_deref().unwrap_or("");
        match row.reason.as_str() {
            "dep-request" => self.dep_answered(board, subject),
            "item-assigned" => item_moved_on(board, subject, &row.thread),
            _ => false,
        }
    }

    /// The dependency is no longer open or was read, or its item is no longer waiting to be started.
    fn dep_answered(&self, board: &BoardView, subject: &str) -> bool {
        let id = subject.split('#').next().unwrap_or(subject);
        let Some(dep) = self.ledger.and_then(|l| l.deps.get(id)) else {
            return true;
        };
        if !dep.state.is("open") || dep.acked_at.some().is_some() {
            return true;
        }
        let item = dep.to.item.some().and_then(|item| board.items.get(item));
        item.is_none_or(|i| !(i.rec.is_live() && i.status.is("todo")))
    }
}

/// The item is started, gone, or in the hands of another thread.
fn item_moved_on(board: &BoardView, item: &str, thread: &str) -> bool {
    board
        .items
        .get(item)
        .is_none_or(|i| !(i.rec.is_live() && i.status.is("todo") && i.thread.is(thread)))
}
