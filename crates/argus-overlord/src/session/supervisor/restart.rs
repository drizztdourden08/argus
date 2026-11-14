//! The loop around the lives of the child: the wait before a restart and the end of the line.

use std::time::Duration;

use super::super::state::{Event as Change, Restart, restart_decision};
use super::Command;
use super::child::{Ended, Wiring};
use super::run::Run;
use crate::ports::ChildHost;

/// Runs children one after another until the supervisor is stopped, the guard refuses one or the
/// restarts are used up.
pub(super) async fn drive<H: ChildHost>(mut run: Run<H>, mut wiring: Wiring) {
    run.resume.clone_from(&wiring.config.resume_from);
    loop {
        match run.life(&mut wiring).await {
            Ended::Requested | Ended::Guard => return,
            Ended::Exited => {
                if !run.restart_after_crash(&mut wiring).await {
                    return;
                }
            }
        }
    }
}

impl<H: ChildHost> Run<H> {
    /// Waits out the backoff. False when no restart follows.
    async fn restart_after_crash(&mut self, w: &mut Wiring) -> bool {
        self.crashes += 1;
        match restart_decision(self.crashes, w.config.restart_max) {
            Restart::Exhausted => {
                self.apply(&Change::RestartExhausted);
                false
            }
            Restart::After(wait) => {
                if !self.wait(w, wait).await {
                    return false;
                }
                self.apply(&Change::RestartAllowed);
                true
            }
        }
    }

    /// Sleeps while still taking messages into the inbox. False when asked to stop.
    async fn wait(&mut self, w: &mut Wiring, wait: Duration) -> bool {
        let sleep = tokio::time::sleep(wait);
        tokio::pin!(sleep);
        loop {
            tokio::select! {
                () = &mut sleep => return true,
                command = w.rx.recv() => match command {
                    Some(Command::Message(text)) => self.inbox.push(text),
                    Some(Command::Stop) | None => return false,
                },
            }
        }
    }
}
