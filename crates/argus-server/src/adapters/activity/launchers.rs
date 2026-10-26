//! The launcher wakes' activity lines, kind `launcher-stale`: part of the wakes' environment
//! trait, so its adapter calls this. The run service's warnings go to the server log only, as
//! in the Node server.

use argus_launchers::launcher::StaleNote;
use argus_store::{ActivityLevel, NewActivity};

use super::ActivityAdapter;

const KIND: &str = "launcher-stale";

impl ActivityAdapter {
    /// Records a line of the launcher wakes.
    pub fn stale_note(&self, note: StaleNote) {
        let new = NewActivity::new(KIND, ActivityLevel::or_info(note.level), note.text)
            .project(Some(note.project))
            .thread(note.thread);
        self.log.add(new);
    }
}
