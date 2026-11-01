//! One target with a time limit. A removal that never comes back (a git that hangs past its own timeout, a wait that
//! never ends) fails with a sentence once the limit has passed, and the request goes on with the next target. While a
//! target runs, the record says which one and since when.

use super::record::{Current, Outcome, RequestRec, Target};
use crate::clock::iso_from_ms;
use crate::service::Service;
use std::sync::mpsc::{self, RecvTimeoutError};
use std::thread;
use std::time::Duration;

/// `90 s`, `30 min`.
fn limit_text(ms: u64) -> String {
    if ms >= 60_000 {
        format!("{} min", ms / 60_000)
    } else {
        format!("{} s", ms.div_ceil(1000))
    }
}

fn failed(target: &Target, text: String, code: &str) -> Outcome {
    Outcome {
        id: target.id.clone(),
        label: target.label.clone(),
        ok: false,
        text,
        code: Some(code.to_string()),
    }
}

fn overdue(target: &Target, ms: u64) -> Outcome {
    let text = format!(
        "{}: the removal did not finish within {}, so the request went on without it. It may still finish on its own: look at the listing before asking again",
        target.label,
        limit_text(ms)
    );
    failed(target, text, "timed_out")
}

fn broke(target: &Target) -> Outcome {
    let text = format!(
        "{}: the removal stopped on an error inside Argus; look at the listing before asking again",
        target.label
    );
    failed(target, text, "failed")
}

impl Service {
    fn set_current(&self, project: &str, id: &str, target: Option<&Target>) {
        let now = self.now();
        let current = target.map(|t| Current {
            id: t.id.clone(),
            label: t.label.clone(),
            since: iso_from_ms(now),
            since_ms: now,
            for_s: 0,
        });
        self.update_request(project, id, |rec| rec.current = current);
        self.save_requests(project);
    }

    /// The target through the very same removal, on its own thread, waited for at most `target_limit_ms`.
    pub(super) fn remove_watched(
        &self,
        project: &str,
        rec: &RequestRec,
        target: &Target,
    ) -> Outcome {
        self.set_current(project, &rec.id, Some(target));
        let (tx, rx) = mpsc::channel();
        let (service, held_rec, held_target) = (self.clone(), rec.clone(), target.clone());
        thread::spawn(move || {
            let _ = tx.send(service.remove_one(&held_rec, &held_target));
        });
        let ms = u64::try_from(self.opts.target_limit_ms.max(0)).unwrap_or(0);
        let outcome = match rx.recv_timeout(Duration::from_millis(ms)) {
            Ok(outcome) => outcome,
            Err(RecvTimeoutError::Timeout) => overdue(target, ms),
            Err(RecvTimeoutError::Disconnected) => broke(target),
        };
        self.set_current(project, &rec.id, None);
        outcome
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_limit_reads_in_seconds_below_a_minute_and_in_minutes_above() {
        assert_eq!(limit_text(300), "1 s");
        assert_eq!(limit_text(45_000), "45 s");
        assert_eq!(limit_text(30 * 60_000), "30 min");
    }
}
