//! The wakes table over SQLite: one row per wake, the first row of a window carries the event.

use std::sync::Arc;

use argus_core::StorageError;
use argus_store::rusqlite::{OptionalExtension, Row, params};
use argus_store::{Db, StoreError};
use argus_wake::store::{RowsQuery, StoreResult, WakeStore, prune_cutoff};
use argus_wake::time::ms_or_zero;
use argus_wake::wake::{FinishedEvent, WakeInput, WakeRow, WakeStatus};

const COLUMNS: &str = "id, n, scope, thread, reason, subject, text, at, status, listeners, delivered_at, lines, count, merged_into";
const COUNTER: &str = "wake";

/// The wakes table.
pub struct SqlWakeStore {
    db: Arc<Db>,
}

impl SqlWakeStore {
    /// The table of a database.
    #[must_use]
    pub fn new(db: Arc<Db>) -> Self {
        SqlWakeStore { db }
    }
}

fn failed(what: &str, err: &StoreError) -> StorageError {
    StorageError(format!("wakes: {what}: {err}"))
}

/// The status of a row. A passive notice is stored as `coalesced` with no event to fold into,
/// so the table keeps the three words the Node schema allows and Node still opens it.
fn status_of(word: &str, merged_into: Option<&str>) -> WakeStatus {
    match (word, merged_into) {
        ("delivered", _) => WakeStatus::Delivered,
        ("coalesced", None) => WakeStatus::Passive,
        ("coalesced", Some(_)) => WakeStatus::Coalesced,
        _ => WakeStatus::Queued,
    }
}

fn status_word(status: WakeStatus) -> &'static str {
    match status {
        WakeStatus::Queued => "queued",
        WakeStatus::Delivered => "delivered",
        WakeStatus::Coalesced | WakeStatus::Passive => "coalesced",
    }
}

fn row_of(r: &Row<'_>) -> argus_store::rusqlite::Result<WakeRow> {
    let lines: Option<String> = r.get(11)?;
    let merged_into: Option<String> = r.get(13)?;
    let status = status_of(&r.get::<_, String>(8)?, merged_into.as_deref());
    Ok(WakeRow {
        id: r.get(0)?,
        n: u64::try_from(r.get::<_, i64>(1)?).unwrap_or(0),
        scope: r.get(2)?,
        thread: r.get(3)?,
        reason: r.get(4)?,
        subject: r.get(5)?,
        text: r.get(6)?,
        at: r.get(7)?,
        status,
        listeners: u32::try_from(r.get::<_, i64>(9)?).unwrap_or(0),
        delivered_at: r.get(10)?,
        lines: lines.and_then(|text| serde_json::from_str(&text).ok()),
        count: u32::try_from(r.get::<_, i64>(12)?).unwrap_or(1),
        merged_into,
    })
}

impl SqlWakeStore {
    fn read(
        &self,
        sql: &str,
        args: &[&dyn argus_store::rusqlite::ToSql],
    ) -> StoreResult<Vec<WakeRow>> {
        self.db.with_conn(|c| {
            let mut st = c
                .prepare_cached(sql)
                .map_err(|e| StorageError(format!("wakes: {e}")))?;
            let rows = st
                .query_map(args, row_of)
                .map_err(|e| StorageError(format!("wakes: {e}")))?;
            rows.collect::<Result<_, _>>()
                .map_err(|e| StorageError(format!("wakes: {e}")))
        })
    }
}

impl WakeStore for SqlWakeStore {
    fn insert(&self, wake: &WakeInput, at: &str) -> StoreResult<WakeRow> {
        self.db
            .tx(|tx| {
                let n = tx.next_counter(COUNTER)?;
                tx.conn().execute(
                    "INSERT INTO wakes(id, n, scope, thread, reason, subject, text, at, status, listeners, count) VALUES(?, ?, ?, ?, ?, ?, ?, ?, 'queued', 0, 1)",
                    params![format!("W-{n}"), n, wake.scope, wake.thread, wake.reason, wake.subject, wake.text, at],
                )?;
                Ok(WakeRow::queued(u64::try_from(n).unwrap_or(0), wake, at))
            })
            .map_err(|e| failed("insert", &e))
    }

    fn finish(&self, lead: &WakeRow, others: &[WakeRow], event: &FinishedEvent) -> StoreResult<()> {
        let lines = serde_json::to_string(&event.lines).unwrap_or_default();
        self.db
            .tx(|tx| {
                tx.conn().execute(
                    "UPDATE wakes SET lines = ?, count = ?, status = ?, listeners = ?, delivered_at = ? WHERE id = ?",
                    params![lines, event.count, status_word(event.status), event.listeners, event.delivered_at, lead.id],
                )?;
                for other in others {
                    tx.conn().execute(
                        "UPDATE wakes SET status = 'coalesced', merged_into = ? WHERE id = ?",
                        params![lead.id, other.id],
                    )?;
                }
                Ok(())
            })
            .map_err(|e| failed("finish", &e))
    }

    fn mark_delivered(&self, id: &str, listeners: u32, at: &str) -> StoreResult<()> {
        self.db
            .tx(|tx| {
                tx.conn().execute(
                    "UPDATE wakes SET status = 'delivered', listeners = ?, delivered_at = ? WHERE id = ?",
                    params![listeners, at, id],
                )?;
                Ok(())
            })
            .map_err(|e| failed("mark delivered", &e))
    }

    fn queued_events(
        &self,
        scope: &str,
        thread: &str,
        since_iso: &str,
    ) -> StoreResult<Vec<WakeRow>> {
        let base = format!(
            "SELECT {COLUMNS} FROM wakes WHERE status = 'queued' AND lines IS NOT NULL AND scope = ?1 AND at >= ?2"
        );
        if thread == "*" {
            self.read(&format!("{base} ORDER BY n"), &[&scope, &since_iso])
        } else {
            self.read(
                &format!("{base} AND thread = ?3 ORDER BY n"),
                &[&scope, &since_iso, &thread],
            )
        }
    }

    fn unflushed(&self) -> StoreResult<Vec<WakeRow>> {
        self.read(
            &format!(
                "SELECT {COLUMNS} FROM wakes WHERE status = 'queued' AND lines IS NULL ORDER BY n"
            ),
            &[],
        )
    }

    fn get(&self, id: &str) -> StoreResult<Option<WakeRow>> {
        let rows = self.read(
            &format!("SELECT {COLUMNS} FROM wakes WHERE id = ?1"),
            &[&id],
        )?;
        Ok(rows.into_iter().next())
    }

    fn rows(&self, scope: &str, query: &RowsQuery) -> StoreResult<Vec<WakeRow>> {
        let limit = i64::try_from(query.limit).unwrap_or(i64::MAX);
        let base = format!("SELECT {COLUMNS} FROM wakes WHERE scope = ?1");
        match query.thread.as_deref().filter(|t| *t != "*") {
            Some(thread) => self.read(
                &format!("{base} AND thread = ?2 ORDER BY n DESC LIMIT ?3"),
                &[&scope, &thread, &limit],
            ),
            None => self.read(
                &format!("{base} ORDER BY n DESC LIMIT ?2"),
                &[&scope, &limit],
            ),
        }
    }

    fn last_at(
        &self,
        scope: &str,
        thread: &str,
        reason: &str,
        subject: Option<&str>,
    ) -> StoreResult<i64> {
        let newest: Option<String> = self.db.with_conn(|c| {
            c.query_row(
                "SELECT MAX(at) FROM wakes WHERE scope = ?1 AND thread = ?2 AND reason = ?3 AND (?4 IS NULL OR subject = ?4)",
                params![scope, thread, reason, subject],
                |r| r.get(0),
            )
            .optional()
            .map(Option::flatten)
            .map_err(|e| StorageError(format!("wakes: {e}")))
        })?;
        Ok(ms_or_zero(newest.as_deref()))
    }

    fn scopes(&self) -> StoreResult<Vec<String>> {
        self.db.with_conn(|c| {
            let mut st = c
                .prepare_cached("SELECT DISTINCT scope FROM wakes")
                .map_err(|e| StorageError(format!("wakes: {e}")))?;
            let rows = st
                .query_map([], |r| r.get(0))
                .map_err(|e| StorageError(format!("wakes: {e}")))?;
            rows.collect::<Result<_, _>>()
                .map_err(|e| StorageError(format!("wakes: {e}")))
        })
    }

    fn prune(&self, now_ms: i64) -> StoreResult<Vec<String>> {
        let cutoff = prune_cutoff(now_ms);
        self.db
            .tx(|tx| {
                let mut st = tx
                    .conn()
                    .prepare_cached("SELECT DISTINCT scope FROM wakes WHERE at < ?")?;
                let hit: Vec<String> = st
                    .query_map([&cutoff], |r| r.get(0))?
                    .collect::<Result<_, _>>()?;
                tx.conn()
                    .execute("DELETE FROM wakes WHERE at < ?", [&cutoff])?;
                Ok(hit)
            })
            .map_err(|e| failed("prune", &e))
    }
}
