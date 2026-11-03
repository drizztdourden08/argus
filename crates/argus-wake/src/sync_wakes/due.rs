//! The projects whose remote sync is still due, as open notices: they stay open until a record ends them.

use std::collections::HashMap;
use std::sync::Arc;

use argus_core::remote::SyncRecord;

use super::{REASON, lead_of, tracked};
use crate::reminders::{NoticeSource, OpenNotice};
use crate::source::{BoardSource, SyncSource};

/// The open `remote-sync-due` notices.
pub struct SyncDue {
    boards: Arc<dyn BoardSource>,
    syncs: Arc<dyn SyncSource>,
    due_ms: i64,
    started_at: i64,
}

impl SyncDue {
    /// Syncs due `due_ms` after the last record, counted from `started_at` for a lead with neither a record nor a
    /// registration date.
    #[must_use]
    pub fn new(
        boards: Arc<dyn BoardSource>,
        syncs: Arc<dyn SyncSource>,
        due_ms: i64,
        started_at: i64,
    ) -> Self {
        Self {
            boards,
            syncs,
            due_ms,
            started_at,
        }
    }
}

impl NoticeSource for SyncDue {
    fn open(&self, now_ms: i64) -> Vec<OpenNotice> {
        let records: HashMap<String, SyncRecord> = self.syncs.syncs().into_iter().collect();
        let mut scopes = self.boards.scopes();
        scopes.sort();
        let mut out = Vec::new();
        for scope in scopes.iter().filter(|s| tracked(s)) {
            let Some(lead) = self.boards.board(scope).as_deref().and_then(lead_of) else {
                continue;
            };
            let synced = records.get(scope).and_then(SyncRecord::at_ms);
            let since = synced.or(lead.registered_ms).unwrap_or(self.started_at);
            if now_ms - since >= self.due_ms {
                out.push(OpenNotice {
                    reason: REASON.to_owned(),
                    project: scope.clone(),
                    subject: lead.id,
                });
            }
        }
        out
    }
}
