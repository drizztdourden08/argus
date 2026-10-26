//! The worktree service's activity lines: part of its notifier trait, so the notifier adapter
//! calls this.

use argus_store::{ActivityLevel, NewActivity};
use argus_worktrees::ActivityLine;

use super::ActivityAdapter;

impl ActivityAdapter {
    /// Records a line of the worktree service.
    pub fn worktree_line(&self, line: ActivityLine) {
        let new = NewActivity::new(line.kind, ActivityLevel::or_info(line.level), line.text)
            .project(Some(line.project))
            .thread(line.thread);
        self.log.add(new);
    }
}
