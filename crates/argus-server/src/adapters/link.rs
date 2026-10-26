//! What the launcher, run and worktree services ask of the wake engine. The wake wiring plugs
//! the engine in with [`WakeBridge::set`]; until then every call does nothing and succeeds.

use std::sync::{Arc, PoisonError, RwLock};

use argus_launchers::RunRow;
use argus_launchers::launcher::WakeEvent as LauncherWake;
use argus_worktrees::WakeEvent as TreeWake;

/// The wake engine as the services of this slice see it.
pub trait WakeLink: Send + Sync {
    /// A run ended (any final status): a failed one wakes the lead.
    fn run_ended(&self, _run: &RunRow) {}

    /// Wakes of the launcher service, `launcher-master-missing` and `launcher-stale`.
    ///
    /// # Errors
    /// The engine refused them.
    fn launcher_wakes(&self, _events: Vec<LauncherWake>) -> Result<(), String> {
        Ok(())
    }

    /// When the wake store last sent this wake (ms); 0 when never.
    fn launcher_last_sent_ms(
        &self,
        _scope: &str,
        _thread: &str,
        _reason: &str,
        _subject: &str,
    ) -> i64 {
        0
    }

    /// Wakes of the worktree service, `worktree-stale`.
    ///
    /// # Errors
    /// The engine refused them, so they are not remembered as told.
    fn worktree_wakes(&self, _events: &[TreeWake]) -> Result<(), String> {
        Ok(())
    }
}

/// The link the services hold from the start; the engine is plugged in later.
#[derive(Default)]
pub struct WakeBridge {
    link: RwLock<Option<Arc<dyn WakeLink>>>,
}

impl WakeBridge {
    /// Plugs the engine in.
    pub fn set(&self, link: Arc<dyn WakeLink>) {
        *self.link.write().unwrap_or_else(PoisonError::into_inner) = Some(link);
    }

    fn current(&self) -> Option<Arc<dyn WakeLink>> {
        self.link
            .read()
            .unwrap_or_else(PoisonError::into_inner)
            .clone()
    }
}

impl WakeLink for WakeBridge {
    fn run_ended(&self, run: &RunRow) {
        if let Some(link) = self.current() {
            link.run_ended(run);
        }
    }

    fn launcher_wakes(&self, events: Vec<LauncherWake>) -> Result<(), String> {
        self.current()
            .map_or(Ok(()), |link| link.launcher_wakes(events))
    }

    fn launcher_last_sent_ms(&self, scope: &str, thread: &str, reason: &str, subject: &str) -> i64 {
        self.current().map_or(0, |link| {
            link.launcher_last_sent_ms(scope, thread, reason, subject)
        })
    }

    fn worktree_wakes(&self, events: &[TreeWake]) -> Result<(), String> {
        self.current()
            .map_or(Ok(()), |link| link.worktree_wakes(events))
    }
}
