//! The numbers `C-n` of the check runs, kept in the dashboard database: one key per run holds
//! its number and one counter per project hands out the next, so a number never moves and is
//! never given twice, across restarts.

use std::sync::Arc;

use argus_records::checks::IdBook;
use argus_store::Db;

/// The check run numbers over the database's key value table.
pub struct DbIds(Arc<Db>);

impl DbIds {
    /// The numbers kept in `db`.
    #[must_use]
    pub fn new(db: Arc<Db>) -> Self {
        DbIds(db)
    }
}

impl IdBook for DbIds {
    fn number(&self, project: &str, run_id: &str) -> Option<u32> {
        let key = format!("check-run:{project}:{run_id}");
        let counter = format!("check-seq:{project}");
        let n = self
            .0
            .tx(|tx| {
                if let Some(kept) = tx.kv_get(&key)? {
                    return Ok(kept.parse::<i64>().unwrap_or(0));
                }
                let n = tx.next_counter(&counter)?;
                tx.kv_set(&key, &n.to_string())?;
                Ok(n)
            })
            .ok()?;
        u32::try_from(n).ok().filter(|n| *n > 0)
    }
}
