//! A fault open for an hour also reaches the owner: a toast and a line in the notifications.

use std::collections::HashSet;

use argus_core::clock::age_text;

use crate::activity::fault_line;
use crate::faults::Fault;
use crate::time::ms_or_zero;

/// The topic of a notice about a wake fault.
pub const TOPIC_FAULT: &str = "wake-fault";

/// The fault of a hosted Overlord whose child is gone: it reaches the owner at once, since the
/// Overlord cannot resolve its own outage.
pub const HOSTED_DOWN: &str = "hosted-down";

/// The topic of a notice about an unread wake.
pub const TOPIC_STALL: &str = "stall";

/// The topic of a notice that an item or a to-do was assigned to the owner (`notify.auto.assigned`).
pub const TOPIC_ASSIGNED: &str = "assigned";

/// The topic of a notice that a bug he reported is ready to verify (`notify.auto.verify`).
pub const TOPIC_VERIFY: &str = "verify";

/// The topic of a notice that a test plan is ready to run (`notify.auto.run`).
pub const TOPIC_RUN: &str = "run";

/// What the owner is told.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OwnerNotice {
    /// What the notice is about (`wake-fault`, `stall`, `assigned`, `verify` or `run`); the server keeps one told-mark per topic.
    pub topic: String,
    /// The headline.
    pub title: String,
    /// The words.
    pub text: String,
    /// The project of the thread.
    pub project: String,
    /// The thread: the one that is faulted, or the one that made the change.
    pub thread: String,
    /// When the fault opened (ISO), or the event of an item (`<id>:<rev>:<kind>`); with the topic, the
    /// project and the thread it names the notice for good.
    pub since: String,
}

/// Where the owner's notices go. The server plugs the notification service in.
pub trait OwnerSink: Send + Sync {
    /// Tell the owner.
    fn notify(&self, notice: OwnerNotice);
}

type Key = (String, String, String);

fn key_of(fault: &Fault) -> Key {
    (
        fault.project.clone(),
        fault.thread.clone(),
        fault.since.clone(),
    )
}

/// The faults the owner was told about.
pub struct OwnerAlerts {
    after_ms: i64,
    told: HashSet<Key>,
}

impl OwnerAlerts {
    /// Tell the owner of a fault once it has been open `after_ms`.
    #[must_use]
    pub fn new(after_ms: i64) -> Self {
        Self {
            after_ms,
            told: HashSet::new(),
        }
    }

    /// The notices due now: one for each fault open long enough that the owner has not heard of yet.
    pub fn due(&mut self, now: i64, faults: &[Fault]) -> Vec<OwnerNotice> {
        self.told
            .retain(|key| faults.iter().any(|f| key_of(f) == *key));
        let mut out = Vec::new();
        for fault in faults {
            let age = now - ms_or_zero(Some(fault.since.as_str()));
            let after = if fault.reason == HOSTED_DOWN {
                0
            } else {
                self.after_ms
            };
            if age >= after && self.told.insert(key_of(fault)) {
                out.push(OwnerNotice {
                    topic: TOPIC_FAULT.to_owned(),
                    title: format!("Wake fault open for {}", age_text(age)),
                    text: fault_line(fault),
                    project: fault.project.clone(),
                    thread: fault.thread.clone(),
                    since: fault.since.clone(),
                });
            }
        }
        out
    }
}
