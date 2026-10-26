//! The sync service: boards into docs, owner posts, proofs, the portfolio cards.

use std::sync::Arc;

use argus_sync::{SyncOptions, SyncService};

use crate::{ServeOptions, Services};

pub(crate) fn build(svc: &Services, opts: &ServeOptions) -> Option<Arc<SyncService>> {
    opts.sync.then(|| {
        Arc::new(SyncService::new(SyncOptions {
            paths: svc.paths.clone(),
            db: Arc::clone(&svc.db),
            config: Arc::clone(&svc.config),
            log: Arc::clone(&svc.log),
            runs: opts.runs.clone(),
            queue: svc.queue.clone(),
        }))
    })
}

pub(crate) async fn start(svc: &Arc<Services>) {
    if let Some(sync) = &svc.sync {
        sync.start().await;
    }
}

pub(crate) fn stop(svc: &Services) {
    if let Some(sync) = &svc.sync {
        sync.stop();
    }
}
