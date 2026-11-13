//! Opening a fault: the record, and the wake that tells the Overlord.

use std::collections::HashMap;

use tracing::warn;

use super::{Env, Key, Registry};
use crate::faults::{Fault, NewFault};
use crate::time::iso;
use crate::wake::WakeInput;

const ERROR_CLIP: usize = 200;
/// A thread with a fault open that is sent another wake tells the Overlord again, at most this often.
const REALERT_MS: i64 = 10 * 60 * 1000;
/// The one fault that clears by itself and is only a notice when it opens: the next reminder wakes the
/// Overlord if it stays open.
const NOTICE_ONLY: &str = "board-closed";

/// Why a wake failed.
pub(super) struct Failure<'a> {
    pub reason: &'a str,
    pub wake_id: Option<&'a str>,
    pub detail: String,
    pub text: Option<String>,
}

/// `65s` for 65 000 milliseconds.
pub(super) fn secs(ms: i64) -> String {
    format!("{}s", (ms + 500).div_euclid(1000))
}

impl Registry {
    pub(super) fn open_faults(&self) -> HashMap<Key, Fault> {
        self.faults(false)
            .into_iter()
            .map(|f| ((f.project.clone(), f.thread.clone()), f))
            .collect()
    }

    /// Open a fault, leave the Overlord a notice and wake it at once (`fault-open`), once per thread.
    /// A `board-closed` fault is the notice only: it ends by itself, and the reminder wakes the Overlord
    /// if it stays open. The Overlord's own fault wakes nobody.
    pub(super) fn raise(
        &mut self,
        env: &mut Env<'_>,
        scope: &str,
        thread: &str,
        failure: &Failure<'_>,
    ) -> Option<Fault> {
        self.pending
            .shift_remove(&(scope.to_owned(), thread.to_owned()));
        let new = NewFault {
            scope: scope.to_owned(),
            thread: thread.to_owned(),
            since: iso(self.now()),
            reason: failure.reason.to_owned(),
            wake_id: failure.wake_id.map(str::to_owned),
            detail: Some(failure.detail.clone()),
        };
        let fault = match self.fault_store.open(&new) {
            Ok(fault) => fault?,
            Err(err) => {
                warn!(scope, thread, %err, "wake fault not recorded");
                return None;
            }
        };
        self.alerted
            .insert((scope.to_owned(), thread.to_owned()), self.now());
        if !self.is_main(scope, thread) {
            let text = failure.text.clone().unwrap_or_else(|| {
                let detail = if failure.detail.is_empty() {
                    String::new()
                } else {
                    format!(" ({})", failure.detail)
                };
                format!("{scope}/{thread}: {}{detail}", failure.reason)
            });
            self.tell_main(env, scope, thread, "wake-fault", text.clone());
            if failure.reason != NOTICE_ONLY {
                self.tell_main(env, scope, thread, "fault-open", text);
            }
        }
        Some(fault)
    }

    /// A new wake came for a thread whose fault is open: tell the Overlord again, at most once per
    /// ten minutes per thread. The Overlord's own fault wakes nobody.
    pub(super) fn realert(&mut self, env: &mut Env<'_>, key: &Key, open: &Fault) {
        let now = self.now();
        let last = self.alerted.get(key).copied().unwrap_or(0);
        if self.is_main(&key.0, &key.1) || now - last <= REALERT_MS {
            return;
        }
        self.alerted.insert(key.clone(), now);
        let (scope, thread) = key;
        let text = format!(
            "{scope}/{thread}: a new wake is waiting and nobody listens (fault open since {}, {}). Reach it now with its session (the owner is waiting), then coord wake-fault clear.",
            open.since, open.reason
        );
        self.tell_main(env, scope, thread, "wake-fault", text);
    }

    /// Sends one wake of `reason` about `scope/thread` to the Overlord.
    fn tell_main(&self, env: &mut Env<'_>, scope: &str, thread: &str, reason: &str, text: String) {
        let wake = WakeInput::about(
            &self.boards.overlord(),
            &self.main_thread(),
            reason,
            &format!("{scope}/{thread}"),
            text,
        );
        if let Err(err) = env.engine.submit(&[wake]) {
            warn!(scope, thread, %err, "wake fault not sent to the Overlord");
        }
    }

    pub(super) fn raise_delivery_error(
        &mut self,
        env: &mut Env<'_>,
        scope: &str,
        thread: &str,
        id: &str,
        err: &str,
    ) {
        let detail: String = if err.is_empty() {
            "write failed".to_owned()
        } else {
            err.chars().take(ERROR_CLIP).collect()
        };
        let failure = Failure {
            reason: "delivery-error",
            wake_id: Some(id),
            detail,
            text: None,
        };
        self.raise(env, scope, thread, &failure);
    }
}
