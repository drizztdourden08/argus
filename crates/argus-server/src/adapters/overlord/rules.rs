//! The timed rules of the hosted Overlord in the dashboard's SQLite file: `overlord_rules` has one
//! row per rule, the rule as JSON beside its number. The table is made here with `IF NOT EXISTS`,
//! so the six migrations Node shares stay as they are. The first creation seeds the three owner
//! rules; a table that exists is never seeded again, even when it is empty.

use std::sync::Arc;

use argus_overlord::rules::{Rule, seed};
use argus_store::rusqlite::{OptionalExtension, params};
use argus_store::{Db, StoreError, Tx};

const COUNTER: &str = "overlord.rule-seq";
const TABLE: &str = "overlord_rules";
const CREATE: &str = "CREATE TABLE IF NOT EXISTS overlord_rules (
    n INTEGER PRIMARY KEY,
    id TEXT NOT NULL UNIQUE,
    json TEXT NOT NULL
)";

/// The rules over the shared database.
pub struct SqlRules {
    db: Arc<Db>,
}

fn table_exists(tx: &Tx) -> Result<bool, StoreError> {
    let found: Option<i64> = tx
        .conn()
        .query_row(
            "SELECT 1 FROM sqlite_master WHERE type = 'table' AND name = ?1",
            [TABLE],
            |r| r.get(0),
        )
        .optional()?;
    Ok(found.is_some())
}

fn put(tx: &Tx, rule: &Rule) -> Result<(), StoreError> {
    let n: i64 = number_of(&rule.id);
    let json = serde_json::to_string(rule)?;
    tx.conn().execute(
        "INSERT INTO overlord_rules (n, id, json) VALUES (?1, ?2, ?3) \
         ON CONFLICT(id) DO UPDATE SET json = excluded.json",
        params![n, rule.id, json],
    )?;
    Ok(())
}

/// The number in `OR-n`; 0 for an id of another shape.
fn number_of(id: &str) -> i64 {
    id.strip_prefix("OR-")
        .and_then(|n| n.parse().ok())
        .unwrap_or(0)
}

fn setup(tx: &Tx, now_ms: i64) -> Result<(), StoreError> {
    if table_exists(tx)? {
        return Ok(());
    }
    tx.conn().execute_batch(CREATE)?;
    let seeded = seed(now_ms);
    for rule in &seeded {
        put(tx, rule)?;
    }
    tx.kv_set(COUNTER, &seeded.len().to_string())
}

impl SqlRules {
    /// The store, with its table made and seeded when it is missing.
    ///
    /// # Errors
    /// A store failure.
    pub fn open(db: Arc<Db>, now_ms: i64) -> Result<Self, StoreError> {
        db.tx(|tx| setup(tx, now_ms))?;
        Ok(Self { db })
    }

    /// Every rule, oldest number first.
    ///
    /// # Errors
    /// A store failure or a row that is not a rule.
    pub fn list(&self) -> Result<Vec<Rule>, StoreError> {
        let rows: Vec<String> = self.db.with_conn(|c| {
            let mut stmt = c.prepare("SELECT json FROM overlord_rules ORDER BY n")?;
            let rows = stmt.query_map([], |r| r.get::<_, String>(0))?;
            rows.collect::<Result<Vec<_>, _>>()
        })?;
        rows.iter()
            .map(|json| serde_json::from_str(json).map_err(StoreError::from))
            .collect()
    }

    /// One rule, `None` for an unknown id.
    ///
    /// # Errors
    /// A store failure or a row that is not a rule.
    pub fn get(&self, id: &str) -> Result<Option<Rule>, StoreError> {
        let row: Option<String> = self.db.with_conn(|c| {
            c.query_row("SELECT json FROM overlord_rules WHERE id = ?1", [id], |r| {
                r.get(0)
            })
            .optional()
        })?;
        row.map(|json| serde_json::from_str(&json).map_err(StoreError::from))
            .transpose()
    }

    /// The next free id, `OR-n`. A number is never given twice, even after a delete.
    ///
    /// # Errors
    /// A store failure.
    pub fn next_id(&self) -> Result<String, StoreError> {
        Ok(format!("OR-{}", self.db.next_counter(COUNTER)?))
    }

    /// Saves a rule, new or changed.
    ///
    /// # Errors
    /// A store failure.
    pub fn save(&self, rule: &Rule) -> Result<(), StoreError> {
        self.db.tx(|tx| put(tx, rule))
    }

    /// Removes a rule; false for an unknown id.
    ///
    /// # Errors
    /// A store failure.
    pub fn delete(&self, id: &str) -> Result<bool, StoreError> {
        let gone = self
            .db
            .with_conn(|c| c.execute("DELETE FROM overlord_rules WHERE id = ?1", [id]))?;
        Ok(gone > 0)
    }
}

#[cfg(test)]
mod specs {
    use argus_core::Bus;

    use super::*;

    fn store() -> (tempfile::TempDir, SqlRules) {
        let dir = tempfile::tempdir().unwrap();
        let db = Db::open(&dir.path().join("a.db"), Bus::new()).unwrap();
        (dir, SqlRules::open(Arc::new(db), 1_800_000_000_000).unwrap())
    }

    // test-tag: critical
    #[test]
    fn the_first_open_seeds_three_rules_and_a_second_does_not() {
        let (_dir, rules) = store();
        assert_eq!(rules.list().unwrap().len(), 3);
        for rule in rules.list().unwrap() {
            assert!(rules.delete(&rule.id).unwrap());
        }
        let again = SqlRules::open(Arc::clone(&rules.db), 1_800_000_000_000).unwrap();
        assert_eq!(again.list().unwrap().len(), 0);
    }

    #[test]
    fn ids_continue_after_the_seed_and_are_never_reused() {
        let (_dir, rules) = store();
        assert_eq!(rules.next_id().unwrap(), "OR-4");
        rules.delete("OR-3").unwrap();
        assert_eq!(rules.next_id().unwrap(), "OR-5");
    }

    #[test]
    fn a_saved_change_is_read_back_and_an_unknown_id_is_none() {
        let (_dir, rules) = store();
        let mut rule = rules.get("OR-1").unwrap().unwrap();
        rule.enabled = false;
        rules.save(&rule).unwrap();
        assert!(!rules.get("OR-1").unwrap().unwrap().enabled);
        assert_eq!(rules.get("OR-99").unwrap(), None);
        assert!(!rules.delete("OR-99").unwrap());
    }
}
