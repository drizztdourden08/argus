//! What the run service tells the dashboard: run docs into the doc store (so the page streams
//! carry them), flushed lines to the live tail, warnings to the server log.

use std::sync::Arc;

use argus_core::log::Logger;
use argus_core::{Coll, DocChange};
use argus_launchers::{LogEvent, RunEvents, RunRow};
use argus_store::Db;
use serde_json::json;

use crate::sse::RunLogs;

/// The run service's events over the doc store and the log hub.
pub struct DocRunEvents {
    db: Arc<Db>,
    logs: Arc<RunLogs>,
    log: Arc<Logger>,
}

impl DocRunEvents {
    /// Events written to `db` (the run docs), `logs` (the lines) and `log` (the warnings).
    #[must_use]
    pub fn new(db: Arc<Db>, logs: Arc<RunLogs>, log: Arc<Logger>) -> Self {
        DocRunEvents { db, logs, log }
    }

    fn apply(&self, scope: &str, change: DocChange) {
        if let Err(e) = self.db.docs().apply(scope, &[change]) {
            self.warn("run doc not written", &format!("{scope}: {e}"));
        }
    }
}

impl RunEvents for DocRunEvents {
    fn project(&self, row: &RunRow) {
        self.apply(&row.scope, DocChange::set(Coll::Runs, row.id.clone(), row.to_doc()));
    }

    fn unproject(&self, scope: &str, rid: &str) {
        self.apply(scope, DocChange::delete(Coll::Runs, rid));
    }

    fn reproject(&self, scope: &str, rows: &[RunRow]) {
        let docs = rows
            .iter()
            .map(|r| DocChange::set(Coll::Runs, r.id.clone(), r.to_doc()))
            .collect();
        if let Err(e) = self.db.docs().replace(scope, &[Coll::Runs], docs) {
            self.warn("run docs not rebuilt", &format!("{scope}: {e}"));
        }
    }

    fn log_line(&self, event: &LogEvent) {
        self.logs.publish(event);
    }

    fn warn(&self, message: &str, detail: &str) {
        self.log.warn(message, json!({ "detail": detail }));
    }
}
