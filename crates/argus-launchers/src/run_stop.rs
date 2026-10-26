//! Stopping a run: a live one is ended as a tree, a queued one never starts, and an orphan
//! row (a run a restart interrupted) has its recorded pid tree killed when that pid still
//! is a shell of ours.

use std::sync::Arc;
use std::time::Duration;

use crate::error::RunError;
use crate::host::ProcessHost;
use crate::live_run::LiveRun;
use crate::names::{check_key, check_rid};
use crate::record::{RunPatch, RunRow, RunStatus};
use crate::run_service::{RunService, run_key};

/// How long past the grace period a stop waits for the run to report its end.
const STOP_EXTRA: Duration = Duration::from_secs(4);

impl<H: ProcessHost> RunService<H> {
    fn grace(&self) -> Duration {
        Duration::from_millis(self.limits().stop_grace_ms)
    }

    pub(crate) fn live_run(&self, key: &str, rid: &str) -> Option<Arc<LiveRun>> {
        self.state().live.get(&run_key(key, rid)).cloned()
    }

    /// Stops run `rid` of project `key` on behalf of `by`; returns the run as it is then.
    pub async fn stop(&self, key: &str, rid: &str, by: &str) -> Result<RunRow, RunError> {
        check_key(key)?;
        check_rid(rid)?;
        let row = self.known_run(key, rid)?;
        if let Some(live) = self.live_run(key, rid) {
            self.stop_live(&live, by).await;
            return Ok(self.inner.deps.store.get(key, rid).unwrap_or(row));
        }
        if row.status == RunStatus::Queued {
            let message = format!("stopped by {by} before it started");
            return Ok(self.fail_now(&row, &message, RunStatus::Stopped, Some(by)));
        }
        Ok(self.stop_orphan(row, by).await)
    }

    async fn stop_live(&self, live: &Arc<LiveRun>, by: &str) {
        let first = {
            let mut control = live.control();
            let first = control.stop_by.is_none();
            if first {
                control.stop_by = Some(by.to_owned());
            }
            first
        };
        if first {
            live.sys(&format!("stop requested by {by}"));
            if let Some(pid) = live.pid {
                self.inner.deps.host.kill_tree(pid, self.grace()).await;
            }
        }
        live.wait_done(self.grace() + STOP_EXTRA).await;
    }

    async fn stop_orphan(&self, row: RunRow, by: &str) -> RunRow {
        let Some(pid) = row.pid.filter(|_| row.orphan) else {
            return row;
        };
        if !self.inner.deps.host.looks_like_ours(pid).await {
            return row;
        }
        self.inner.deps.host.kill_tree(pid, self.grace()).await;
        let was = row.error.clone().unwrap_or_else(|| "orphaned".to_owned());
        let patch = RunPatch {
            orphan: Some(false),
            error: Some(Some(format!("{was}; killed by {by}"))),
            ..RunPatch::default()
        };
        self.set_state(&row.scope, &row.id, &patch).unwrap_or(row)
    }
}
