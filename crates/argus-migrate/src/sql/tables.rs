//! The columns that hold a board key: `scope` in eight tables, `project` in two, and the path of
//! an attachment. Free text (`url`, bodies) stays: old addresses still resolve.

use rusqlite::Transaction;

use crate::error::Result;
use crate::map::IdMap;
use crate::rewrite::strings::rewrite_text;

/// The tables with a `scope` column.
pub const SCOPE_TABLES: [&str; 8] = [
    "docs",
    "changes",
    "posts",
    "launch_defs",
    "runs",
    "attachments",
    "wakes",
    "wake_faults",
];
/// The tables with a `project` column.
pub const PROJECT_TABLES: [&str; 2] = ["activity", "notifications"];

fn rename_column(tx: &Transaction, table: &str, column: &str, map: &IdMap) -> Result<()> {
    let sql = format!("UPDATE {table} SET {column} = ?1 WHERE {column} = ?2");
    for (from, to) in map {
        tx.execute(&sql, [to, from])?;
    }
    Ok(())
}

fn attachment_paths(tx: &Transaction, map: &IdMap) -> Result<()> {
    let mut found: Vec<(String, String)> = Vec::new();
    let mut stmt = tx.prepare("SELECT id, path FROM attachments WHERE path LIKE '%.coord%'")?;
    for row in stmt.query_map([], |r| Ok((r.get(0)?, r.get(1)?)))? {
        found.push(row?);
    }
    drop(stmt);
    for (id, path) in found {
        if let Some(new) = rewrite_text(&path, map) {
            tx.execute("UPDATE attachments SET path = ?1 WHERE id = ?2", [new, id])?;
        }
    }
    Ok(())
}

/// Every table column of step 6 except the kv keys.
pub fn rewrite(tx: &Transaction, map: &IdMap) -> Result<()> {
    for table in SCOPE_TABLES {
        rename_column(tx, table, "scope", map)?;
    }
    for table in PROJECT_TABLES {
        rename_column(tx, table, "project", map)?;
    }
    attachment_paths(tx, map)
}
