//! The sub-agent services' activity port: a line carries its own kind and a level name.

use argus_agents::{ActivityLine, ActivitySink};
use argus_store::{ActivityLevel, NewActivity};

use super::ActivityAdapter;

impl ActivitySink for ActivityAdapter {
    fn add(&self, line: ActivityLine) {
        let level = ActivityLevel::or_info(&line.level);
        let new = NewActivity::new(line.kind, level, line.text)
            .project(Some(line.project))
            .thread(line.thread);
        self.log.add(new);
    }
}
