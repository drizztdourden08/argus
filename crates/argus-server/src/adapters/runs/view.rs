//! The runs as the portfolio cards read them.

use std::sync::Arc;

use argus_launchers::{ListFilter, RunService};
use argus_sync::RunsView;
use serde_json::{Value, json};

use super::host::RunHost;

/// The counts and last runs the sync service puts on the cards.
pub struct RunsCards {
    runs: Arc<RunService<RunHost>>,
}

impl RunsCards {
    /// The cards' view of the run service.
    #[must_use]
    pub fn new(runs: Arc<RunService<RunHost>>) -> Self {
        RunsCards { runs }
    }
}

impl RunsView for RunsCards {
    fn running(&self, key: &str) -> usize {
        let filter = ListFilter::default().with_statuses("running");
        self.runs.list(key, &filter).len()
    }

    fn last_run(&self, key: &str, name: &str) -> Option<Value> {
        let filter = ListFilter {
            name: Some(name.to_owned()),
            limit: Some(1),
            ..ListFilter::default()
        };
        let run = self.runs.list(key, &filter).into_iter().next()?;
        Some(json!({
            "id": run.id,
            "name": run.name,
            "status": run.status.as_str(),
            "exitCode": run.exit_code,
            "signal": run.signal,
            "error": run.error,
            "stdin": run.stdin,
            "queuedAt": run.queued_at,
            "startedAt": run.started_at,
            "endedAt": run.ended_at,
            "lastLine": run.last_line,
        }))
    }
}
