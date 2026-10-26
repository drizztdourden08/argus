//! Opening a fault: the record, and the wake that tells the Overlord.

use std::collections::HashMap;

use tracing::warn;

use super::{Env, Key, Registry};
use crate::faults::{Fault, NewFault};
use crate::reasons::MAIN_SCOPE;
use crate::time::iso;
use crate::wake::WakeInput;

const ERROR_CLIP: usize = 200;

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

    /// Open a fault and wake the Overlord, once per thread. The Overlord's own fault wakes nobody.
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
        if !self.is_main(scope, thread) {
            let text = failure.text.clone().unwrap_or_else(|| {
                let detail = if failure.detail.is_empty() {
                    String::new()
                } else {
                    format!(" ({})", failure.detail)
                };
                format!("{scope}/{thread}: {}{detail}", failure.reason)
            });
            let wake = WakeInput::about(
                MAIN_SCOPE,
                &self.main_thread(),
                "wake-fault",
                &format!("{scope}/{thread}"),
                text,
            );
            if let Err(err) = env.engine.submit(&[wake]) {
                warn!(scope, thread, %err, "wake fault not sent to the Overlord");
            }
        }
        Some(fault)
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
