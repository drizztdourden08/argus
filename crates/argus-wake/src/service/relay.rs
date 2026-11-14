//! The wake relay inside the service: each registry pass asks the relay what to do for the due threads and hands
//! the answers to the port the server plugged in.

use std::sync::Arc;

use super::WakeService;
use crate::registry::{DueThread, target_of};
use crate::relay::{Action, LastAttempt, RelayPort};

impl WakeService {
    /// Where the relay notes, the headless resumes and the owner notices go.
    pub fn set_relay_port(&mut self, port: Arc<dyn RelayPort>) {
        self.relay_port = Some(port);
    }

    /// One relay pass; returns how many actions were handed to the port. Does nothing without a port.
    pub fn relay_tick(&mut self) -> usize {
        let Some(port) = self.relay_port.clone() else {
            return 0;
        };
        let now = self.clock.now_ms();
        let (due, live) =
            self.with_env(|registry, env| (registry.due(env), registry.relay_live(env)));
        let targets: Vec<_> = due.iter().map(target_of).collect();
        let actions = self.relay.step(now, &targets, &live);
        for action in &actions {
            match action {
                Action::Note { via, targets } => port.note(via, targets),
                Action::Headless(target) => port.headless(target),
                Action::Notice(target) => port.notice(target),
            }
        }
        actions.len()
    }

    /// The last relay attempt for a due thread, for the due list.
    #[must_use]
    pub fn relay_last(&self, due: &DueThread) -> Option<LastAttempt> {
        self.relay.last(&due.project, &due.thread)
    }
}
