//! The memory of rule 4: since when each lead has been idle. It lives in memory only; a restart starts the
//! clock again.

use std::collections::HashMap;
use std::sync::{Mutex, MutexGuard, PoisonError};

/// When each project's lead was first seen idle (epoch ms).
#[derive(Debug, Default)]
pub struct IdleWatch {
    since: Mutex<HashMap<String, i64>>,
}

impl IdleWatch {
    /// A watch that has seen nothing.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// The lead of `project` is idle now. The first sight keeps `first`; later sights keep that first time.
    /// Returns the time the idleness began.
    #[must_use]
    pub fn seen_idle(&self, project: &str, first: i64) -> i64 {
        *self.lock().entry(project.to_owned()).or_insert(first)
    }

    /// The lead of `project` is not idle (or has no lead to watch): the clock starts over.
    pub fn forget(&self, project: &str) {
        self.lock().remove(project);
    }

    fn lock(&self) -> MutexGuard<'_, HashMap<String, i64>> {
        self.since.lock().unwrap_or_else(PoisonError::into_inner)
    }
}

// test-tag: critical
#[cfg(test)]
mod tests {
    use super::IdleWatch;

    #[test]
    fn the_first_sight_sets_the_start_and_later_sights_keep_it() {
        let watch = IdleWatch::new();
        assert_eq!(watch.seen_idle("rotp", 100), 100);
        assert_eq!(watch.seen_idle("rotp", 900), 100);
        assert_eq!(watch.seen_idle("argus", 900), 900);
    }

    #[test]
    fn forgetting_a_project_starts_its_clock_again() {
        let watch = IdleWatch::new();
        watch.seen_idle("rotp", 100);
        watch.forget("rotp");
        assert_eq!(watch.seen_idle("rotp", 900), 900);
    }
}
