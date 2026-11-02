//! A fault open for an hour also reaches the owner: a toast and a line in the notifications.

use std::collections::HashSet;

use argus_core::clock::age_text;

use crate::activity::fault_line;
use crate::faults::Fault;
use crate::time::ms_or_zero;

/// What the owner is told.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OwnerNotice {
    /// The headline.
    pub title: String,
    /// The words.
    pub text: String,
    /// The project of the thread.
    pub project: String,
    /// The thread.
    pub thread: String,
    /// When the fault opened (ISO), which with the project and the thread names it for good.
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
            if age >= self.after_ms && self.told.insert(key_of(fault)) {
                out.push(OwnerNotice {
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
