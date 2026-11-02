//! Wakes about cross-board dependencies. The ledger of the portfolio board (collection `deps`) keeps
//! the last transitions of each dependency as `history` rows; each row that matters tells one side:
//!
//! * `dep-request`: `opening -> open`, to the assignee.
//! * `dep-resolved`: `done`, `dropped` or `gone`, to the requester (it resumes its item).
//! * `dep-blocked`: `blocked`, to the requester, with the reason.
//! * `dep-cancelled`: `cancelled`, to the assignee.
//! * `dep-reopened`: `blocked -> open` or `done -> open`, to the requester.
//!
//! The target is the thread of the item on that side while it is live, else the lead of that project
//! (the rule of `kanban-stale`). A paused thread is skipped and hears it once it resumes.
//!
//! Each row wakes at most once: its subject is `D-n#seq`, and the wakes table is asked for it
//! (`last_sent`, as `item_wakes` does), so a restart sends nothing again. A row older than the days
//! the wakes table keeps ([`crate::store::KEEP_DAYS`]) is not news and is left out, since the table
//! could no longer tell whether it was sent.
//!
//! The producer only reads: the ledger and the project boards through [`BoardSource`]. It does not
//! depend on `argus-board`.

mod config;
mod kind;
mod text;
mod view;

pub use config::DepWakeConfig;
pub use view::{DepEnd, DepStep, DepView, RequestedBy, WaitsOn};

use std::collections::HashSet;

use crate::item_wakes::{LastSent, target_for};
use crate::reasons::PORTFOLIO;
use crate::source::BoardSource;
use crate::store::KEEP_DAYS;
use crate::time::ms_or_zero;
use crate::wake::WakeInput;
use kind::Kind;

const DAY_MS: i64 = 86_400_000;

/// What a tick reads.
pub struct DepEnv<'a> {
    /// The project boards and the portfolio board with the ledger.
    pub boards: &'a dyn BoardSource,
    /// The last wake the wakes table knows, so a restart does not send it again.
    pub last_sent: LastSent<'a>,
}

/// The dependency wakes.
pub struct DepWakes {
    config: DepWakeConfig,
    sent: HashSet<String>,
}

impl DepWakes {
    /// Dependency wakes with a config.
    #[must_use]
    pub fn new(config: DepWakeConfig) -> Self {
        Self {
            config,
            sent: HashSet::new(),
        }
    }

    /// The timing.
    #[must_use]
    pub fn config(&self) -> &DepWakeConfig {
        &self.config
    }

    /// The wakes due now: one for each history row not sent before.
    pub fn tick(&mut self, now: i64, env: &DepEnv<'_>) -> Vec<WakeInput> {
        if !self.config.enabled {
            return Vec::new();
        }
        let Some(ledger) = env.boards.board(PORTFOLIO) else {
            return Vec::new();
        };
        let mut out = Vec::new();
        for dep in ledger.deps.values() {
            for step in &dep.history {
                out.extend(self.wake_for(now, env, dep, step));
            }
        }
        out
    }

    fn wake_for(
        &mut self,
        now: i64,
        env: &DepEnv<'_>,
        dep: &DepView,
        step: &DepStep,
    ) -> Option<WakeInput> {
        let kind = Kind::of(step).filter(|_| step.seq > 0 && known(dep) && fresh(now, step))?;
        let subject = format!("{}#{}", dep.id.as_str(), step.seq);
        if self.sent.contains(&subject) {
            return None;
        }
        let end = kind.audience(dep);
        let (scope, thread) = hearer(env.boards, end)?;
        if (env.last_sent)(&scope, &thread, kind.reason(), &subject) > 0 {
            self.sent.insert(subject);
            return None;
        }
        let wake = WakeInput::about(
            &scope,
            &thread,
            kind.reason(),
            &subject,
            kind.text(dep, step),
        );
        self.sent.insert(subject);
        Some(wake)
    }
}

/// Does the record name both items, so that its words are whole?
fn known(dep: &DepView) -> bool {
    dep.id.some().is_some()
        && [&dep.from, &dep.to]
            .iter()
            .all(|end| end.project.some().is_some() && end.item.some().is_some())
}

/// Is the row young enough that the wakes table still holds its wake, so that a restart can tell?
fn fresh(now: i64, step: &DepStep) -> bool {
    let at = ms_or_zero(step.at.some());
    at == 0 || now - at <= KEEP_DAYS * DAY_MS
}

/// The project and the thread that hears for an end: the item's thread while live, else the lead.
/// Nobody when the board or a thread to wake is missing, or the thread is paused.
fn hearer(boards: &dyn BoardSource, end: &DepEnd) -> Option<(String, String)> {
    let scope = end.project.some()?;
    let board = boards.board(scope)?;
    let owner = end
        .item
        .some()
        .and_then(|item| board.items.get(item))
        .map_or(end.thread.as_str(), |item| item.thread.as_str());
    let thread = target_for(&board, owner)?;
    let paused = board
        .threads
        .get(&thread)
        .is_some_and(|t| t.status.is("paused"));
    (!paused).then(|| (scope.to_owned(), thread))
}
