//! The approvals of the hosted Overlord in the dashboard's SQLite file (`overlord_approvals`), and
//! the owner's toast over the notification service. The table is made here with `IF NOT EXISTS`,
//! so the six migrations Node shares stay as they are.

use std::sync::Arc;

use argus_notify::{By, Level, Notify, PingRequest};
use argus_overlord::approvals::Approval;
use argus_overlord::ports::{ApprovalFault, ApprovalStore, OwnerToast};
use argus_store::Db;

const TABLE: &str = "CREATE TABLE IF NOT EXISTS overlord_approvals (
    id TEXT PRIMARY KEY,
    at TEXT NOT NULL,
    tool TEXT NOT NULL,
    input_summary TEXT NOT NULL,
    status TEXT NOT NULL,
    decided_at TEXT,
    decided_by TEXT
);";

const TITLE: &str = "The Overlord needs an answer";
/// The board of the Overlord and its coordinator thread.
pub(crate) const BOARD: &str = "portfolio";
pub(crate) const THREAD: &str = "coordinator";

/// The rows over the shared database, as the `ApprovalStore` port.
pub struct SqlApprovals {
    db: Arc<Db>,
}

fn fault<E>(_: E) -> ApprovalFault {
    ApprovalFault("the approval table failed".to_owned())
}

impl SqlApprovals {
    /// The store, with its table made when it is missing.
    ///
    /// # Errors
    /// A fault when the table cannot be made.
    pub fn open(db: Arc<Db>) -> Result<Self, ApprovalFault> {
        db.with_conn(|c| c.execute_batch(TABLE)).map_err(fault)?;
        Ok(Self { db })
    }
}

impl ApprovalStore for SqlApprovals {
    fn save(&self, approval: &Approval) -> Result<(), ApprovalFault> {
        self.db
            .with_conn(|c| {
                c.execute(
                    "INSERT OR REPLACE INTO overlord_approvals \
                     (id, at, tool, input_summary, status, decided_at, decided_by) \
                     VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
                    (
                        &approval.id,
                        &approval.at,
                        &approval.tool,
                        &approval.input_summary,
                        approval.status.as_str(),
                        &approval.decided_at,
                        &approval.decided_by,
                    ),
                )
            })
            .map(|_| ())
            .map_err(fault)
    }

    fn abandon_pending(&self, at: &str) -> Result<(), ApprovalFault> {
        self.db
            .with_conn(|c| {
                c.execute(
                    "UPDATE overlord_approvals SET status = 'expired', decided_at = ?1, \
                     decided_by = 'restart' WHERE status = 'pending'",
                    (at,),
                )
            })
            .map(|_| ())
            .map_err(fault)
    }
}

/// The owner's toast through the notification service: urgent, from the Overlord. Without the
/// service (notifications off) there is no toast and the tab is the only place to answer.
pub struct NotifyToast(pub Option<Arc<Notify>>);

impl OwnerToast for NotifyToast {
    fn urgent(&self, text: &str, url: &str) {
        let Some(notify) = &self.0 else {
            return;
        };
        let _ = notify.send(PingRequest {
            title: Some(TITLE.to_owned()),
            text: text.to_owned(),
            level: Level::Urgent,
            project: Some(BOARD.to_owned()),
            thread: Some(THREAD.to_owned()),
            by: By::Overlord,
            url: Some(url.to_owned()),
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use argus_core::Bus;
    use argus_overlord::approvals::{Decision, Queue, Status};

    // test-tag: critical
    #[test]
    fn a_saved_row_is_replaced_and_a_restart_abandons_the_pending_ones() {
        let dir = tempfile::tempdir().unwrap();
        let db = Arc::new(Db::open(&dir.path().join("argus.db"), Bus::new()).unwrap());
        let store = SqlApprovals::open(Arc::clone(&db)).unwrap();
        let mut queue = Queue::default();
        let done = queue.submit("Bash", "ls", "2026-11-14T10:00:00.000Z", 1);
        let open = queue.submit("Bash", "rm x", "2026-11-14T10:00:01.000Z", 2);
        store.save(&done).unwrap();
        store.save(&open).unwrap();
        let settled = queue
            .decide(&done.id, Decision::Allow, None, "2026-11-14T10:00:02.000Z")
            .unwrap();
        store.save(&settled).unwrap();
        store.abandon_pending("2026-11-14T11:00:00.000Z").unwrap();
        let rows: Vec<(String, String)> = db
            .with_conn(|c| {
                let mut stmt =
                    c.prepare("SELECT id, status FROM overlord_approvals ORDER BY id")?;
                stmt.query_map((), |r| Ok((r.get(0)?, r.get(1)?)))?
                    .collect::<Result<_, _>>()
            })
            .unwrap();
        assert_eq!(rows.len(), 2);
        let status_of = |id: &str| rows.iter().find(|r| r.0 == id).unwrap().1.clone();
        assert_eq!(status_of(&done.id), Status::Allowed.as_str());
        assert_eq!(status_of(&open.id), Status::Expired.as_str());
    }
}
