//! The `wake_faults` table over SQLite: one open row per project and thread.

use std::sync::Arc;

use argus_core::StorageError;
use argus_store::rusqlite::{OptionalExtension, Row, params};
use argus_store::{Db, StoreError, Tx};
use argus_wake::faults::{ClearFault, Fault, FaultStore, NewFault};
use argus_wake::store::StoreResult;

const COLUMNS: &str =
    "id, scope, thread, since, reason, wake_id, detail, cleared_at, cleared_by, note";

/// The faults table.
pub struct SqlFaultStore {
    db: Arc<Db>,
}

impl SqlFaultStore {
    /// The table of a database.
    #[must_use]
    pub fn new(db: Arc<Db>) -> Self {
        SqlFaultStore { db }
    }
}

fn failed(what: &str, err: &StoreError) -> StorageError {
    StorageError(format!("wake faults: {what}: {err}"))
}

fn fault_of(r: &Row<'_>) -> argus_store::rusqlite::Result<Fault> {
    Ok(Fault {
        id: r.get(0)?,
        project: r.get(1)?,
        thread: r.get(2)?,
        since: r.get(3)?,
        reason: r.get(4)?,
        wake_id: r.get(5)?,
        detail: r.get(6)?,
        cleared_at: r.get(7)?,
        cleared_by: r.get(8)?,
        note: r.get(9)?,
    })
}

fn by_id(tx: &Tx<'_>, id: i64) -> Result<Option<Fault>, StoreError> {
    Ok(tx
        .conn()
        .query_row(
            &format!("SELECT {COLUMNS} FROM wake_faults WHERE id = ?"),
            [id],
            fault_of,
        )
        .optional()?)
}

fn open_row(tx: &Tx<'_>, scope: &str, thread: &str) -> Result<Option<i64>, StoreError> {
    Ok(tx
        .conn()
        .query_row(
            "SELECT id FROM wake_faults WHERE scope = ? AND thread = ? AND cleared_at IS NULL",
            [scope, thread],
            |r| r.get(0),
        )
        .optional()?)
}

fn insert(tx: &Tx<'_>, fault: &NewFault) -> Result<Option<Fault>, StoreError> {
    if open_row(tx, &fault.scope, &fault.thread)?.is_some() {
        return Ok(None);
    }
    tx.conn().execute(
        "INSERT INTO wake_faults(scope, thread, since, reason, wake_id, detail) VALUES(?, ?, ?, ?, ?, ?)",
        params![
            fault.scope,
            fault.thread,
            fault.since,
            fault.reason,
            fault.wake_id,
            fault.detail
        ],
    )?;
    by_id(tx, tx.conn().last_insert_rowid())
}

fn close(tx: &Tx<'_>, request: &ClearFault) -> Result<Option<Fault>, StoreError> {
    let Some(id) = open_row(tx, &request.scope, &request.thread)? else {
        return Ok(None);
    };
    tx.conn().execute(
        "UPDATE wake_faults SET cleared_at = ?, cleared_by = ?, note = ? WHERE id = ?",
        params![request.at, request.by, request.note, id],
    )?;
    by_id(tx, id)
}

impl FaultStore for SqlFaultStore {
    fn open(&self, fault: &NewFault) -> StoreResult<Option<Fault>> {
        self.db
            .tx(|tx| insert(tx, fault))
            .map_err(|e| failed("open", &e))
    }

    fn clear(&self, request: &ClearFault) -> StoreResult<Option<Fault>> {
        self.db
            .tx(|tx| close(tx, request))
            .map_err(|e| failed("clear", &e))
    }

    fn list(&self, all: bool) -> StoreResult<Vec<Fault>> {
        let filter = if all { "" } else { "WHERE cleared_at IS NULL" };
        let sql = format!("SELECT {COLUMNS} FROM wake_faults {filter} ORDER BY id");
        self.db.with_conn(|c| {
            let mut st = c
                .prepare_cached(&sql)
                .map_err(|e| StorageError(format!("wake faults: {e}")))?;
            let rows = st
                .query_map([], fault_of)
                .map_err(|e| StorageError(format!("wake faults: {e}")))?;
            rows.collect::<Result<_, _>>()
                .map_err(|e| StorageError(format!("wake faults: {e}")))
        })
    }
}
