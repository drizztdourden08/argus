//! Where the worktree service says what it did (the activity log) and whom it wants to wake
//! (the wake engine, through the bridge).

use std::sync::Arc;

use argus_worktrees::{ActivityLine, Notifier, WakeEvent};

use crate::adapters::activity::ActivityAdapter;
use crate::adapters::link::{WakeBridge, WakeLink};

/// The worktree service's notifier.
pub struct TreeNotifier {
    activity: ActivityAdapter,
    link: Arc<WakeBridge>,
}

impl TreeNotifier {
    /// A notifier writing to the activity log and waking through `link`.
    #[must_use]
    pub fn new(activity: ActivityAdapter, link: Arc<WakeBridge>) -> Self {
        TreeNotifier { activity, link }
    }
}

impl Notifier for TreeNotifier {
    fn activity(&self, line: ActivityLine) {
        self.activity.worktree_line(line);
    }

    fn wake(&self, events: &[WakeEvent]) -> Result<(), String> {
        self.link.worktree_wakes(events)
    }
}
