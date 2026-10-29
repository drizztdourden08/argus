//! Where the worktree service says what it did (the activity log), whom it wants to wake (the
//! wake engine, through the bridge) and what failed (the server's log).

use std::sync::Arc;

use argus_core::log::Logger;
use argus_worktrees::{ActivityLine, Notifier, WakeEvent};
use serde_json::json;

use crate::adapters::activity::ActivityAdapter;
use crate::{WakeHook, WakeRequest};

/// The worktree service's notifier.
pub struct TreeNotifier {
    activity: ActivityAdapter,
    hook: WakeHook,
    log: Arc<Logger>,
}

impl TreeNotifier {
    /// A notifier writing to the activity log, waking through `hook` and logging to `log`.
    #[must_use]
    pub fn new(activity: ActivityAdapter, hook: WakeHook, log: Arc<Logger>) -> Self {
        TreeNotifier {
            activity,
            hook,
            log,
        }
    }
}

impl Notifier for TreeNotifier {
    fn activity(&self, line: ActivityLine) {
        self.activity.worktree_line(line);
    }

    fn wake(&self, events: &[WakeEvent]) -> Result<(), String> {
        let wakes: Vec<WakeRequest> = events
            .iter()
            .map(|e| WakeRequest::about(&e.scope, &e.thread, e.reason, &e.subject, e.text.clone()))
            .collect();
        self.hook.submit(&wakes)
    }

    fn problem(&self, text: &str) {
        self.log.warn("worktrees", json!({ "err": text }));
    }
}
