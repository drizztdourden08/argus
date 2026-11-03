//! Wakes and owner notices for what happens to items. Every export of a project board is compared with
//! the one before ([`argus_board::item_events`]), so a write by a lead's CLI wakes the thread it concerns
//! as an owner post does. The table is in `docs/wake-policy.md`:
//!
//! * `item-assigned`, `item-unassigned`, `todo-assigned`: an item or a to-do changed hands;
//! * `item-status`: the stage of an item changed under its assignee;
//! * `item-ask`: another thread asked a question or wants a decision on an item;
//! * `item-unblocked`, `children-done`: the last blocker settled, or every child is done or dropped;
//! * `plan-run-done`: a run of a test plan ended, with the counts;
//! * `todo-quiet`: a to-do of another lead's board that blocks an item stayed quiet for a day;
//! * for the owner, a notice: an item or a to-do assigned to him, a bug he reported ready to verify,
//!   a test plan ready to run. An item he holds that was unblocked is a line of the activity log.
//!
//! The first export of a project after a start only seeds: nothing is fired for what was already
//! there, and a write made while Argus is down is not compared. Each event fires once: its subject is
//! `<id>:<rev>:<kind>` and the wakes table is asked for it, as `dep_wakes` does, so a restart that
//! replays an export sends nothing twice. The owner's own posts keep their path in [`crate::policy`]:
//! an `item-assigned` wake sent in the last minute for the same item is not sent again here. A
//! dependency that landed is told by `dep_wakes` from the ledger, not here.

mod quiet;
mod route;
mod text;

use std::collections::{HashMap, HashSet};
use std::sync::Arc;

use argus_board::item_events::{EventKind, ItemEvent, item_events};

use crate::activity::ActivityLine;
use crate::board::BoardView;
use crate::item_wakes::LastSent;
use crate::reasons::PORTFOLIO;
use crate::reminders::OwnerNotice;
use crate::source::BoardSource;
use crate::wake::WakeInput;
use route::{Told, Where, route};

/// An `item-assigned` wake this recent for the same item came from the owner's post, and is not sent twice.
const POLICY_SENT_MS: i64 = 60_000;

/// The events remembered in memory; the wakes table remembers the wakes for good.
const SENT_KEEP: usize = 4000;

/// What a tick reads.
pub struct WatchEnv<'a> {
    /// The project boards, the one just exported among them.
    pub boards: &'a dyn BoardSource,
    /// The last wake the wakes table knows, so a restart does not send it again.
    pub last_sent: LastSent<'a>,
    /// Now, in epoch milliseconds.
    pub now: i64,
}

impl WatchEnv<'_> {
    /// When the wakes table last sent a wake with this reason and subject to the thread of `wake`; 0 when never.
    fn stored(&self, wake: &WakeInput, reason: &str, subject: &str) -> i64 {
        (self.last_sent)(&wake.scope, &wake.thread, reason, subject)
    }
}

/// What an export of a board came to.
#[derive(Debug, Default)]
pub struct Watched {
    /// The wakes to submit.
    pub wakes: Vec<WakeInput>,
    /// The notices for the owner.
    pub owner: Vec<OwnerNotice>,
    /// The lines for the activity log.
    pub activity: Vec<ActivityLine>,
}

/// The item events of every project board.
#[derive(Default)]
pub struct ItemWatch {
    last: HashMap<String, Arc<BoardView>>,
    sent: HashSet<String>,
}

impl ItemWatch {
    /// A watch that has seen no board yet.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// A board of a project was exported again: what changed since the last export, told once. The
    /// first export of a project only remembers the board.
    pub fn on_board(&mut self, scope: &str, env: &WatchEnv<'_>) -> Watched {
        let mut out = Watched::default();
        let Some(now) = env.boards.board(scope).filter(|_| scope != PORTFOLIO) else {
            return out;
        };
        let Some(before) = self.last.insert(scope.to_owned(), Arc::clone(&now)) else {
            return out;
        };
        if before.json() == now.json() {
            return out;
        }
        let at = Where {
            scope,
            board: &now,
            boards: env.boards,
        };
        for event in item_events(before.json(), now.json()) {
            let told = route(&at, &event);
            if self.is_new(env, &event, scope, told.as_ref()) {
                collect(told, &mut out);
            }
        }
        out
    }

    /// The quiet to-dos of every project that block an item, each told once to the lead it names.
    pub fn quiet(&mut self, env: &WatchEnv<'_>) -> Vec<WakeInput> {
        let mut out = Vec::new();
        for scope in env.boards.scopes().iter().filter(|s| *s != PORTFOLIO) {
            let Some(board) = env.boards.board(scope) else {
                continue;
            };
            for wake in quiet::quiet_todos(env.boards, scope, &board, env.now) {
                let subject = wake.subject.clone().unwrap_or_default();
                let key = format!("{}/{}/{subject}", wake.scope, wake.thread);
                if env.stored(&wake, &wake.reason, &subject) == 0 && self.sent.insert(key) {
                    out.push(wake);
                }
            }
        }
        out
    }

    /// Is this event not told before: not in memory, not in the wakes table, and not already sent by the
    /// owner's post? The event is remembered either way.
    fn is_new(
        &mut self,
        env: &WatchEnv<'_>,
        event: &ItemEvent,
        scope: &str,
        told: Option<&Told>,
    ) -> bool {
        if self.sent.len() > SENT_KEEP {
            self.sent.clear();
        }
        if !self.sent.insert(format!("{scope}/{}", event.subject())) {
            return false;
        }
        match told {
            Some(Told::Wake(wake)) => !stored_before(env, wake, event),
            _ => true,
        }
    }
}

/// Has the wakes table this wake already, or did the owner's post send the assignment a moment ago?
fn stored_before(env: &WatchEnv<'_>, wake: &WakeInput, event: &ItemEvent) -> bool {
    let subject = wake.subject.as_deref().unwrap_or_default();
    if env.stored(wake, &wake.reason, subject) > 0 {
        return true;
    }
    let posted = env.stored(wake, "item-assigned", &event.id);
    event.kind == EventKind::Assigned && posted > 0 && env.now - posted < POLICY_SENT_MS
}

fn collect(told: Option<Told>, out: &mut Watched) {
    match told {
        Some(Told::Wake(wake)) => out.wakes.push(wake),
        Some(Told::Owner(notice)) => out.owner.push(notice),
        Some(Told::Line(line)) => out.activity.push(line),
        None => {}
    }
}
