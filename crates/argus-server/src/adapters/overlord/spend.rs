//! The spend ledger of the hosted Overlord in the dashboard's SQLite file: `overlord_spend` has one
//! row per turn, `overlord_spend_warned` one row per month that had its warning. The tables are
//! made here with `IF NOT EXISTS`, so the six migrations Node shares stay as they are.

use std::sync::Arc;

use argus_overlord::ports::{SpendFault, SpendLedger};
use argus_overlord::spend::{Last, SpendRow};
use argus_store::Db;

const TABLES: &str = "CREATE TABLE IF NOT EXISTS overlord_spend (
    at TEXT NOT NULL,
    session_id TEXT NOT NULL,
    process_seq INTEGER NOT NULL,
    cumulative_usd REAL NOT NULL,
    delta_usd REAL NOT NULL,
    month TEXT NOT NULL
);
CREATE INDEX IF NOT EXISTS overlord_spend_month ON overlord_spend (month);
CREATE TABLE IF NOT EXISTS overlord_spend_warned (month TEXT PRIMARY KEY);";

/// The ledger over the shared database, as the `SpendLedger` port.
pub struct SqlLedger {
    db: Arc<Db>,
}

fn fault<E>(_: E) -> SpendFault {
    SpendFault("the spend ledger failed".to_owned())
}

impl SqlLedger {
    /// The ledger, with its tables made when they are missing.
    ///
    /// # Errors
    /// A fault when the tables cannot be made.
    pub fn open(db: Arc<Db>) -> Result<Self, SpendFault> {
        db.with_conn(|c| c.execute_batch(TABLES)).map_err(fault)?;
        Ok(Self { db })
    }
}

impl SpendLedger for SqlLedger {
    fn record(&self, row: &SpendRow) -> Result<(), SpendFault> {
        let seq = i64::try_from(row.process_seq).map_err(fault)?;
        self.db
            .with_conn(|c| {
                c.execute(
                    "INSERT INTO overlord_spend (at, session_id, process_seq, cumulative_usd, delta_usd, month) \
                     VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
                    (
                        &row.at,
                        &row.session_id,
                        seq,
                        row.cumulative_usd,
                        row.delta_usd,
                        &row.month,
                    ),
                )
            })
            .map(|_| ())
            .map_err(fault)
    }

    fn month_total(&self, month: &str) -> Result<f64, SpendFault> {
        self.db
            .with_conn(|c| {
                c.query_row(
                    "SELECT COALESCE(SUM(delta_usd), 0.0) FROM overlord_spend WHERE month = ?1",
                    (month,),
                    |r| r.get::<_, f64>(0),
                )
            })
            .map_err(fault)
    }

    fn last(&self, session: &str) -> Result<Option<Last>, SpendFault> {
        let rows = self
            .db
            .with_conn(|c| {
                let mut stmt = c.prepare(
                    "SELECT process_seq, cumulative_usd FROM overlord_spend \
                     WHERE session_id = ?1 ORDER BY rowid DESC LIMIT 1",
                )?;
                stmt.query_map((session,), |r| {
                    Ok((r.get::<_, i64>(0)?, r.get::<_, f64>(1)?))
                })?
                .collect::<Result<Vec<_>, _>>()
            })
            .map_err(fault)?;
        Ok(rows.first().map(|(seq, usd)| Last {
            process_seq: u64::try_from(*seq).unwrap_or(0),
            cumulative_usd: *usd,
        }))
    }

    fn warned(&self, month: &str) -> Result<bool, SpendFault> {
        let count = self
            .db
            .with_conn(|c| {
                c.query_row(
                    "SELECT COUNT(*) FROM overlord_spend_warned WHERE month = ?1",
                    (month,),
                    |r| r.get::<_, i64>(0),
                )
            })
            .map_err(fault)?;
        Ok(count > 0)
    }

    fn set_warned(&self, month: &str) -> Result<(), SpendFault> {
        self.db
            .with_conn(|c| {
                c.execute(
                    "INSERT OR IGNORE INTO overlord_spend_warned (month) VALUES (?1)",
                    (month,),
                )
            })
            .map(|_| ())
            .map_err(fault)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use argus_core::Bus;

    fn row(session: &str, seq: u64, cumulative: f64, delta: f64, month: &str) -> SpendRow {
        SpendRow {
            at: "2026-11-14T10:00:00.000Z".to_owned(),
            session_id: session.to_owned(),
            process_seq: seq,
            cumulative_usd: cumulative,
            delta_usd: delta,
            month: month.to_owned(),
        }
    }

    fn open(file: &std::path::Path) -> SqlLedger {
        let db = Db::open(file, Bus::new()).unwrap();
        SqlLedger::open(Arc::new(db)).unwrap()
    }

    // test-tag: critical
    #[test]
    fn the_month_total_and_the_warning_survive_a_reopen() {
        let dir = tempfile::tempdir().unwrap();
        let file = dir.path().join("argus.db");
        let ledger = open(&file);
        ledger.record(&row("s1", 1, 0.25, 0.25, "2026-11")).unwrap();
        ledger.record(&row("s1", 1, 0.75, 0.5, "2026-11")).unwrap();
        ledger.record(&row("s1", 2, 0.5, 0.5, "2026-12")).unwrap();
        ledger.set_warned("2026-11").unwrap();
        drop(ledger);
        let again = open(&file);
        assert!((again.month_total("2026-11").unwrap() - 0.75).abs() < 1e-9);
        assert!((again.month_total("2027-01").unwrap()).abs() < 1e-9);
        assert!(again.warned("2026-11").unwrap());
        assert!(!again.warned("2026-12").unwrap());
        let last = again.last("s1").unwrap().unwrap();
        assert_eq!(last.process_seq, 2);
        assert!(again.last("other").unwrap().is_none());
    }
}
