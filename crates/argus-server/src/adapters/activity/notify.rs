//! The notifications' activity port: every line is kind `note`.

use argus_notify::{ActivityLevel as NotifyLevel, ActivityLine, ActivitySink};
use argus_store::{ActivityLevel, NewActivity};

use super::ActivityAdapter;

const KIND: &str = "note";

fn level_of(level: NotifyLevel) -> ActivityLevel {
    match level {
        NotifyLevel::Info => ActivityLevel::Info,
        NotifyLevel::Warn => ActivityLevel::Warn,
    }
}

impl ActivitySink for ActivityAdapter {
    fn note(&self, line: ActivityLine) {
        let new = NewActivity::new(KIND, level_of(line.level), line.text)
            .project(line.project)
            .thread(line.thread);
        self.log.add(new);
    }
}
