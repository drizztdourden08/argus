//! A worktree whose item closed: once the grace is over, the ones of a thread go as one removal request of mode
//! `item-closed`. The request worker judges each one again right before removing it, so a reopen or a relink inside
//! the grace (or after the request was filed) cancels the removal.

use super::memory::{HygieneState, Memory};
use crate::board::{BoardFacts, Caller};
use crate::requests::RequestInput;
use crate::service::Service;
use std::collections::BTreeMap;

/// One thread's closing worktrees: the worktree id and the closed item it belongs to.
type ByThread = BTreeMap<String, Vec<(String, String)>>;

/// The closing marks whose grace is over and that no request carries, by the thread told.
fn due_by_thread(memory: &Memory, now: i64, grace: i64) -> ByThread {
    let mut out = ByThread::new();
    for (id, mark) in &memory.marks {
        let due = mark.state == HygieneState::Closing
            && mark.request.is_none()
            && now - mark.since >= grace;
        if let (true, Some(thread), Some(item)) = (due, &mark.thread, &mark.item) {
            out.entry(thread.clone())
                .or_default()
                .push((id.clone(), item.clone()));
        }
    }
    out
}

impl Service {
    /// The closing worktrees whose grace ended go to the request worker: one request per thread.
    pub(super) fn hand_on_closed(&self, project: &str, board: &BoardFacts) {
        let (now, grace) = (self.now(), self.opts.item_close_grace_ms);
        let due = self.with_hygiene(project, |memory| due_by_thread(memory, now, grace));
        for (thread, list) in due {
            if board.live_thread(&thread).is_none() {
                continue;
            }
            let mut input = RequestInput::new(project, Caller::Thread(thread));
            input.targets = list.iter().map(|(id, _)| id.clone()).collect();
            input.items = list.iter().cloned().collect();
            input.reason =
                Some("their item closed: removed with it, the full backup first".to_string());
            let Ok(queued) = self.request_item_closed(&input) else {
                // The thread may not remove it (the request is refused and logged): it stays closing.
                continue;
            };
            self.with_hygiene(project, |memory| {
                for (id, _) in &list {
                    if let Some(mark) = memory.marks.get_mut(id) {
                        mark.request = Some(queued.id.clone());
                    }
                }
            });
        }
    }
}
