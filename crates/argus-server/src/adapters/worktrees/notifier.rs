//! Where the worktree service says what it did (the activity log) and whom it wants to wake
//! (the wake engine, through the bridge).

use argus_worktrees::{ActivityLine, Notifier, WakeEvent};

use crate::adapters::activity::ActivityAdapter;
use crate::{WakeHook, WakeRequest};

/// The worktree service's notifier.
pub struct TreeNotifier {
    activity: ActivityAdapter,
    hook: WakeHook,
}

impl TreeNotifier {
    /// A notifier writing to the activity log and waking through `hook`.
    #[must_use]
    pub fn new(activity: ActivityAdapter, hook: WakeHook) -> Self {
        TreeNotifier { activity, hook }
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
}
