//! The sync service: boards into docs, owner posts, proofs, the portfolio cards.

use std::sync::Arc;

use argus_sync::{RunsView, SyncOptions, SyncService};

use crate::adapters::runs::RunsCards;
use crate::{ServeOptions, Services};

/// The runs the cards show: the test's own view, else the run service.
fn runs_view(svc: &Services, opts: &ServeOptions) -> Option<Arc<dyn RunsView>> {
    opts.runs.clone().or_else(|| {
        let cards = RunsCards::new(Arc::clone(svc.runs.as_ref()?));
        Some(Arc::new(cards) as Arc<dyn RunsView>)
    })
}

pub(crate) fn build(svc: &Services, opts: &ServeOptions) -> Option<Arc<SyncService>> {
    opts.sync.then(|| {
        Arc::new(SyncService::new(SyncOptions {
            paths: svc.paths.clone(),
            db: Arc::clone(&svc.db),
            config: Arc::clone(&svc.config),
            log: Arc::clone(&svc.log),
            runs: runs_view(svc, opts),
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
