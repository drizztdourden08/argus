//! The `runs` table as the run service sees it: the columns a row is read from, and the text
//! and number conversions both ways.

use argus_launchers::{Origin, RunKind, RunRow, RunStatus};
use argus_store::rusqlite::{Result, Row};
use serde::de::DeserializeOwned;
use serde_json::Value;

/// The columns of a [`RunRow`], in the order [`run_of`] reads them.
pub(super) const COLUMNS: &str = "scope, rid, name, kind, status, exit_code, signal, pid, cmd, cwd, visible, origin, \"by\", agent, reason, queued_at, started_at, ended_at, log_bytes, log_dropped, log_seq, orphan, error, last_line, stdin";

/// A word the pages and the table spell the same way (`running`, `page`, `out`).
pub(super) fn word<T: DeserializeOwned>(text: String) -> Option<T> {
    serde_json::from_value(Value::String(text)).ok()
}

/// A counter as the table keeps it.
pub(super) fn int(n: u64) -> i64 {
    i64::try_from(n).unwrap_or(i64::MAX)
}

fn count(row: &Row<'_>, at: usize) -> Result<u64> {
    Ok(u64::try_from(row.get::<_, i64>(at)?).unwrap_or(0))
}

/// One row of `SELECT {COLUMNS} FROM runs`.
pub(super) fn run_of(row: &Row<'_>) -> Result<RunRow> {
    Ok(RunRow {
        scope: row.get(0)?,
        id: row.get(1)?,
        name: row.get(2)?,
        kind: RunKind::parse(&row.get::<_, String>(3)?).unwrap_or_default(),
        status: word(row.get(4)?).unwrap_or(RunStatus::Failed),
        exit_code: row.get(5)?,
        signal: row.get(6)?,
        pid: row.get::<_, Option<i64>>(7)?.and_then(|p| u32::try_from(p).ok()),
        cmd: row.get(8)?,
        cwd: row.get(9)?,
        visible: row.get(10)?,
        origin: word(row.get(11)?).unwrap_or(Origin::Cli),
        by: row.get(12)?,
        agent: row.get(13)?,
        reason: row.get(14)?,
        queued_at: row.get(15)?,
        started_at: row.get(16)?,
        ended_at: row.get(17)?,
        log_bytes: count(row, 18)?,
        log_dropped: count(row, 19)?,
        log_seq: count(row, 20)?,
        orphan: row.get(21)?,
        error: row.get(22)?,
        last_line: row.get(23)?,
        stdin: row.get(24)?,
    })
}
