//! Step 6: the database, in one `BEGIN IMMEDIATE` transaction: the scope and project columns,
//! the attachment paths and the kv keys that embed a board key.

mod kv;
mod tables;

use std::path::Path;
use std::time::Duration;

use rusqlite::{Connection, TransactionBehavior};

use crate::error::Result;
use crate::map::IdMap;

/// Rewrites `dashboard.db` (nothing when the file is absent).
pub fn rewrite(db: &Path, map: &IdMap) -> Result<()> {
    if !db.is_file() {
        return Ok(());
    }
    let mut conn = Connection::open(db)?;
    conn.busy_timeout(Duration::from_secs(10))?;
    let tx = conn.transaction_with_behavior(TransactionBehavior::Immediate)?;
    tables::rewrite(&tx, map)?;
    kv::rewrite(&tx, map)?;
    tx.commit()?;
    Ok(())
}

pub use kv::rewrite_key;
pub use tables::{PROJECT_TABLES, SCOPE_TABLES};
