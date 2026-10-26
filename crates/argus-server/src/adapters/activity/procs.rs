//! The process monitor's activity lines: part of its host trait, so the host adapter calls this.

use argus_procs::{Activity, Level};
use argus_store::NewActivity;

use super::ActivityAdapter;

level_of!(Level);

impl ActivityAdapter {
    /// Records a line of the process monitor.
    pub fn procs_line(&self, entry: &Activity) {
        let level = level_of(entry.level);
        self.log
            .add(NewActivity::new(entry.kind, level, entry.text.clone()));
    }
}
