//! When the cache is pruned: once after the start delay, as soon as the worktrees are known, then every day.

use std::sync::Arc;
use std::time::Duration;

use super::{config, places, prune};
use crate::Services;

const TASK: &str = "cache";
const DAY: Duration = Duration::from_secs(24 * 3600);
const RETRY: Duration = Duration::from_secs(30);

async fn run_once(svc: &Arc<Services>) {
    let svc = Arc::clone(svc);
    let _ = tokio::task::spawn_blocking(move || prune(&svc, false)).await;
}

async fn wait_for_worktrees(svc: &Arc<Services>) {
    loop {
        let ready = {
            let svc = Arc::clone(svc);
            tokio::task::spawn_blocking(move || places::worktrees_known(&svc))
                .await
                .unwrap_or(false)
        };
        if ready {
            return;
        }
        tokio::time::sleep(RETRY).await;
    }
}

async fn prune_loop(svc: Arc<Services>) {
    tokio::time::sleep(Duration::from_millis(config::start_delay_ms(&svc))).await;
    wait_for_worktrees(&svc).await;
    loop {
        run_once(&svc).await;
        tokio::time::sleep(DAY).await;
    }
}

/// Starts the timer; ended by [`stop`].
pub(super) fn start(svc: &Arc<Services>) {
    svc.tasks
        .keep(TASK, tokio::spawn(prune_loop(Arc::clone(svc))));
}

/// Ends the timer.
pub(super) fn stop(svc: &Services) {
    svc.tasks.abort(TASK);
}
