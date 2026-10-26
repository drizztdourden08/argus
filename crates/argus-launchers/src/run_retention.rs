//! Retention: at every run end and once an hour, per (scope, name) keep the newest
//! `keep_per_def` finished runs and nothing older than `keep_days`, with their logs and
//! attachments.

use crate::attach::remove_file;
use crate::host::ProcessHost;
use crate::record::RunRow;
use crate::run_service::RunService;
use crate::time::{DAY_MS, parse_iso};

impl<H: ProcessHost> RunService<H> {
    fn drop_runs(&self, runs: &[RunRow]) {
        for run in runs {
            let files: Vec<String> = self
                .inner
                .deps
                .store
                .attachments_of(&run.scope, &run.id)
                .into_iter()
                .map(|a| a.path)
                .collect();
            self.inner.deps.store.remove(&run.scope, &run.id);
            self.inner.deps.events.unproject(&run.scope, &run.id);
            for path in files {
                remove_file(&path);
            }
        }
    }

    pub(crate) fn prune(&self, scope: &str, name: &str) {
        let limits = self.limits();
        let cutoff = self.inner.deps.clock.now_ms() - limits.keep_days * DAY_MS;
        let stale: Vec<RunRow> = self
            .inner
            .deps
            .store
            .finished(scope, name)
            .into_iter()
            .enumerate()
            .filter(|(index, run)| {
                let ended = run.ended_at.as_deref().unwrap_or(&run.queued_at);
                *index >= limits.keep_per_def || parse_iso(ended).is_some_and(|at| at < cutoff)
            })
            .map(|(_, run)| run)
            .collect();
        if !stale.is_empty() {
            self.drop_runs(&stale);
        }
    }

    /// Applies the retention rules to every (scope, name) that has runs.
    pub fn sweep(&self) {
        for (scope, name) in self.inner.deps.store.groups() {
            self.prune(&scope, &name);
        }
    }
}
