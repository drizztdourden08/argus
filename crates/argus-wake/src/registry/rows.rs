//! The rows of the registry: one per live board thread, and one per thread that has a fault.

use std::collections::BTreeSet;

use serde::Serialize;

use super::Pending;
use super::watch::listener_of;
use super::{Env, Key, Registry};
use crate::board::ThreadView;
use crate::engine::ListenerInfo;
use crate::faults::Fault;
use crate::lenient::Text;
use crate::time::iso;

/// Where a thread stands.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum RowState {
    /// A wake failed and nobody cleared it.
    Fault,
    /// A wake waits for a reaction.
    Waking,
    /// A listener covers it and nothing is pending.
    Listening,
    /// No listener, but the thread is in a turn.
    Busy,
    /// No listener and nothing pending.
    Idle,
}

/// The wake a row waits on.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct PendingWake {
    /// The event id.
    pub id: String,
    /// When the registry started waiting (ISO).
    pub since: String,
    /// Why the wake was made.
    pub reason: Option<String>,
}

/// The open fault of a row.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct FaultSummary {
    /// When it opened (ISO).
    pub since: String,
    /// The reason code.
    pub reason: String,
    /// The wake that failed.
    pub wake_id: Option<String>,
    /// What happened.
    pub detail: Option<String>,
}

/// Whether a listener covers a thread.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub struct ListenFlags {
    /// Is a listener covering it?
    pub listening: bool,
    /// An active session with no wake-stream.
    pub deaf: bool,
}

/// Whether a thread's board must be and is open.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BoardFlags {
    /// Must its board be open?
    pub board_required: bool,
    /// Is its board open?
    pub board_open: bool,
    /// Since when its board is open (ISO).
    pub board_since: Option<String>,
}

/// One thread as the registry strip shows it.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RegistryRow {
    /// The doc id (`<project>/<thread>` on the portfolio page).
    pub id: String,
    /// Project key.
    pub project: String,
    /// The thread.
    pub thread: String,
    /// The session title.
    pub session: Option<String>,
    /// `lead` or `thread`.
    pub role: String,
    /// The listener's label.
    pub listener_label: Option<String>,
    /// Since when it is in this state (ISO).
    pub since: Option<String>,
    /// Where it stands.
    pub state: RowState,
    /// The wake it waits on.
    pub pending_wake: Option<PendingWake>,
    /// The open fault.
    pub fault: Option<FaultSummary>,
    /// Listener and wake-stream flags.
    #[serde(flatten)]
    pub flags: ListenFlags,
    /// The board-open flags.
    #[serde(flatten)]
    pub board: BoardFlags,
    /// The last sign of life (ISO).
    pub last_seen: Option<String>,
    /// `working` or `idle`.
    pub activity: Option<String>,
}

impl Registry {
    /// The rows of a project, sorted by thread id.
    #[must_use]
    pub fn rows(&self, env: &Env<'_>, scope: &str) -> Vec<RegistryRow> {
        self.rows_with(env, scope, &self.open_faults())
    }

    pub(super) fn rows_with(
        &self,
        env: &Env<'_>,
        scope: &str,
        faults: &std::collections::HashMap<Key, Fault>,
    ) -> Vec<RegistryRow> {
        let board = self.boards.board(scope);
        let listeners: Vec<ListenerInfo> = env
            .engine
            .listeners_of(scope)
            .into_iter()
            .filter(|l| l.v >= 2)
            .collect();
        let mut ids: BTreeSet<String> = board
            .iter()
            .flat_map(|b| b.threads.keys().cloned())
            .collect();
        ids.extend(
            faults
                .values()
                .filter(|f| f.project == scope)
                .map(|f| f.thread.clone()),
        );
        ids.iter()
            .filter_map(|id| {
                let thread = board.as_ref().and_then(|b| b.threads.get(id));
                let fault = faults.get(&(scope.to_owned(), id.clone()));
                if fault.is_none() && !thread.is_some_and(ThreadView::is_live) {
                    return None;
                }
                Some(self.row_of(env, scope, id, thread, fault, &listeners))
            })
            .collect()
    }

    fn row_of(
        &self,
        env: &Env<'_>,
        scope: &str,
        id: &str,
        thread: Option<&ThreadView>,
        fault: Option<&Fault>,
        listeners: &[ListenerInfo],
    ) -> RegistryRow {
        let listener = listener_of(id, thread, listeners);
        let pending = self.pending.get(&(scope.to_owned(), id.to_owned()));
        let open = env.presence.and_then(|p| p.open_of(scope, id));
        let busy = listener.is_none() && thread.is_some_and(|t| self.in_turn(t));
        let live = thread.is_some_and(ThreadView::is_live);
        let known =
            |read: fn(&ThreadView) -> &Text| thread.and_then(|t| read(t).some()).map(str::to_owned);
        RegistryRow {
            id: id.to_owned(),
            project: scope.to_owned(),
            thread: id.to_owned(),
            session: known(|t| &t.session),
            role: known(|t| &t.role).unwrap_or_else(|| "thread".to_owned()),
            listener_label: listener.and_then(|l| l.label.clone()),
            since: since_of(fault, pending, listener),
            state: state_of(fault, pending, listener, busy),
            pending_wake: pending.map(PendingWake::of),
            fault: fault.map(FaultSummary::of),
            flags: ListenFlags {
                listening: listener.is_some(),
                deaf: live
                    && thread.is_some_and(|t| !t.is_hosted())
                    && listener.is_none()
                    && !busy
                    && thread.is_some_and(|t| self.heartbeat(t)),
            },
            board: BoardFlags {
                board_required: self.board_required(env, thread),
                board_open: open.is_some(),
                board_since: open.map(|o| iso(o.since)),
            },
            last_seen: known(|t| &t.last_seen),
            activity: known(|t| &t.activity),
        }
    }
}

fn state_of(
    fault: Option<&Fault>,
    pending: Option<&Pending>,
    listener: Option<&ListenerInfo>,
    busy: bool,
) -> RowState {
    if fault.is_some() {
        RowState::Fault
    } else if pending.is_some() {
        RowState::Waking
    } else if listener.is_some() {
        RowState::Listening
    } else if busy {
        RowState::Busy
    } else {
        RowState::Idle
    }
}

fn since_of(
    fault: Option<&Fault>,
    pending: Option<&Pending>,
    listener: Option<&ListenerInfo>,
) -> Option<String> {
    match (fault, pending, listener) {
        (Some(f), _, _) => Some(f.since.clone()),
        (None, Some(p), _) => Some(iso(p.since_ms)),
        (None, None, l) => l.map(|l| l.since.clone()),
    }
}

impl PendingWake {
    fn of(pending: &Pending) -> Self {
        Self {
            id: pending.id.clone(),
            since: iso(pending.since_ms),
            reason: pending.reason.clone(),
        }
    }
}

impl FaultSummary {
    fn of(fault: &Fault) -> Self {
        Self {
            since: fault.since.clone(),
            reason: fault.reason.clone(),
            wake_id: fault.wake_id.clone(),
            detail: fault.detail.clone(),
        }
    }
}
