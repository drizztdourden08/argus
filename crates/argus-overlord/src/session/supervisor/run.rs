//! What the supervisor keeps between children: the state, the held messages, whether a turn runs.

use std::sync::Arc;

use tokio::sync::watch;

use super::super::inbox::Inbox;
use super::super::state::{Event as Change, State, apply};
use super::flight::Flight;
use crate::ports::ChildHost;
use crate::protocol::encode_user_message;

pub(super) struct Run<H: ChildHost> {
    pub(super) host: Arc<H>,
    pub(super) state: State,
    state_tx: watch::Sender<State>,
    pub(super) inbox: Inbox,
    /// The tool calls the child has started and not finished.
    pub(super) flight: Flight,
    /// True from a write until its `result`. The state machine cannot say it: the first turn runs
    /// while the status is still `Starting`, because `init` comes after the first message.
    pub(super) busy: bool,
    /// The combined message the running turn was written from, kept so a crash does not lose it.
    in_flight: Option<String>,
    pub(super) crashes: u32,
    pub(super) resume: Option<String>,
}

impl<H: ChildHost> Run<H> {
    pub(super) fn new(
        host: Arc<H>,
        state_tx: watch::Sender<State>,
        (inbox, flight): (Inbox, Flight),
    ) -> Self {
        Self {
            host,
            state: State::default(),
            state_tx,
            inbox,
            flight,
            busy: false,
            in_flight: None,
            crashes: 0,
            resume: None,
        }
    }

    pub(super) fn apply(&mut self, change: &Change) {
        self.state = apply(&self.state, change);
        let _ = self.state_tx.send(self.state.clone());
    }

    /// Writes everything held as one message when no turn runs.
    pub(super) async fn flush(&mut self, pid: u32) {
        if self.busy {
            return;
        }
        let Some(text) = self.inbox.take_combined() else {
            return;
        };
        let line = encode_user_message(&text);
        if self.host.write_line(pid, line.trim_end()).await.is_ok() {
            self.busy = true;
            self.in_flight = Some(text);
            self.apply(&Change::TurnStarted);
        } else {
            self.inbox.requeue(text);
        }
    }

    /// A `result` ended the turn.
    pub(super) fn turn_done(&mut self, cost_usd: Option<f64>) {
        self.busy = false;
        self.in_flight = None;
        self.crashes = 0;
        let cost_usd = cost_usd.unwrap_or(self.state.last_cost_usd);
        self.apply(&Change::ResultReceived { cost_usd });
    }

    /// The child is gone: a turn that never got its `result` goes back to the front of the inbox.
    pub(super) fn child_gone(&mut self, requested: bool) {
        if let Some(text) = self.in_flight.take() {
            self.inbox.requeue(text);
        }
        self.busy = false;
        self.flight.clear();
        self.apply(&Change::Exited { requested });
    }
}
