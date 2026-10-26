//! The wake engine's activity port.

use argus_store::NewActivity;
use argus_wake::activity::{ActivityLine, ActivitySink, Level};

use super::ActivityAdapter;

level_of!(Level);

impl ActivitySink for ActivityAdapter {
    fn add(&self, line: ActivityLine) {
        let new = NewActivity::new(line.kind, level_of(line.level), line.text)
            .project(line.project)
            .thread(line.thread);
        self.log.add(new);
    }
}
