//! Graceful shutdown: queued runs fail, hidden runs are tree-killed (`stopped`, by
//! `server shutdown`), visible runs (a handed build the owner is using) are left running
//! and marked as orphans that Stop can still kill.

use std::sync::Arc;
use std::time::Duration;

use tokio::task::JoinSet;

use crate::host::ProcessHost;
use crate::live_run::LiveRun;
use crate::record::RunStatus;
use crate::run_service::RunService;

const HIDDEN_WAIT: Duration = Duration::from_secs(4);
const FORCE_WAIT: Duration = Duration::from_secs(1);

impl<H: ProcessHost> RunService<H> {
    async fn shut_hidden(&self, live: Arc<LiveRun>) {
        live.control().stop_by = Some("server shutdown".to_owned());
        live.sys("the dashboard is stopping this run");
        if let Some(pid) = live.pid {
            let grace = Duration::from_millis(self.limits().stop_grace_ms);
            self.inner.deps.host.kill_tree(pid, grace).await;
        }
        live.wait_done(HIDDEN_WAIT).await;
        live.force.notify_one();
        live.wait_done(FORCE_WAIT).await;
    }

    async fn leave_visible(&self, live: &Arc<LiveRun>) {
        live.release.notify_one();
        live.wait_done(FORCE_WAIT).await;
    }

    /// Stops accepting starts and ends or releases every run.
    pub async fn shutdown(&self) {
        let sweeper = {
            let mut state = self.state();
            state.closing = true;
            state.sweeper.take()
        };
        if let Some(sweeper) = sweeper {
            sweeper.abort();
        }
        for row in self.inner.deps.store.queued_rows() {
            self.fail_now(
                &row,
                "the dashboard stopped before this run started",
                RunStatus::Failed,
                None,
            );
        }
        let live: Vec<Arc<LiveRun>> = self.state().live.values().cloned().collect();
        let mut hidden = JoinSet::new();
        for run in live {
            if run.visible {
                self.leave_visible(&run).await;
            } else {
                let service = self.clone();
                hidden.spawn(async move { service.shut_hidden(run).await });
            }
        }
        while hidden.join_next().await.is_some() {}
    }
}
