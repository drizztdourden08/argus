//! The log lines of the runs over the `run_logs` table: a batch applied in one transaction,
//! and the reads by cursor and by tail.

use argus_launchers::{LogCommit, LogLine, LogRead, LogStore, ReadMode, Stream};
use argus_store::rusqlite::params;

use super::rows::{int, word};
use super::store::SqliteRuns;

impl LogStore for SqliteRuns {
    fn commit(&self, scope: &str, rid: &str, commit: &LogCommit) -> Result<(), String> {
        let id = format!("{scope}:{rid}");
        let found = self
            .db
            .tx(|tx| {
                let c = tx.conn();
                let updated = c.execute(
                    "UPDATE runs SET log_bytes = ?, log_dropped = ?, log_seq = ?, last_line = ? WHERE id = ?",
                    params![
                        int(commit.log_bytes),
                        int(commit.log_dropped),
                        int(commit.log_seq),
                        commit.last_line,
                        id
                    ],
                )?;
                if updated == 0 {
                    return Ok(false);
                }
                let mut insert = c.prepare_cached(
                    "INSERT INTO run_logs (run_id, seq, stream, at, text, head) VALUES (?, ?, ?, ?, ?, ?)",
                )?;
                for stored in &commit.lines {
                    let line = &stored.line;
                    insert.execute(params![
                        id,
                        int(line.seq),
                        line.stream.as_str(),
                        line.at,
                        line.text,
                        stored.head
                    ])?;
                }
                if let Some(cut) = commit.drop_up_to {
                    c.execute(
                        "DELETE FROM run_logs WHERE run_id = ? AND seq <= ? AND head = 0",
                        params![id, int(cut)],
                    )?;
                }
                Ok(true)
            })
            .map_err(|e| e.to_string())?;
        found
            .then_some(())
            .ok_or_else(|| format!("no run {rid} in {scope}"))
    }

    fn read(&self, scope: &str, rid: &str, read: &LogRead) -> Vec<LogLine> {
        let (order, after, limit) = match read.mode {
            ReadMode::Tail(n) => ("DESC", 0, n),
            ReadMode::After { after, limit } => ("ASC", after, limit),
        };
        let sql = format!(
            "SELECT seq, stream, at, text FROM run_logs WHERE run_id = ?1 AND seq > ?2 AND (?3 IS NULL OR stream = ?3) ORDER BY seq {order} LIMIT {limit}"
        );
        let stream = read.stream.map(Stream::as_str);
        let mut lines: Vec<LogLine> = self.db.with_conn(|c| {
            let Ok(mut statement) = c.prepare_cached(&sql) else {
                return Vec::new();
            };
            statement
                .query_map(params![format!("{scope}:{rid}"), int(after), stream], line_of)
                .map(|rows| rows.flatten().collect())
                .unwrap_or_default()
        });
        if order == "DESC" {
            lines.reverse();
        }
        lines
    }
}

fn line_of(row: &argus_store::rusqlite::Row<'_>) -> argus_store::rusqlite::Result<LogLine> {
    Ok(LogLine {
        seq: u64::try_from(row.get::<_, i64>(0)?).unwrap_or(0),
        stream: word(row.get(1)?).unwrap_or(Stream::Sys),
        at: row.get(2)?,
        text: row.get(3)?,
    })
}
