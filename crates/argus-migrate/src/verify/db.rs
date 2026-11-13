//! The database side of the verification: no old key left as a scope, a project or in a kv key,
//! the integrity check, and (full pass) the row counts equal to the backup's.

use std::path::Path;

use rusqlite::{Connection, OpenFlags};

use super::Check;
use crate::backup::{DB_REL, is_intact};
use crate::error::Result;
use crate::sql::{PROJECT_TABLES, SCOPE_TABLES, rewrite_key};

fn open(file: &Path) -> Result<Connection> {
    Ok(Connection::open_with_flags(
        file,
        OpenFlags::SQLITE_OPEN_READ_ONLY,
    )?)
}

fn count(conn: &Connection, table: &str) -> Result<i64> {
    Ok(conn.query_row(&format!("SELECT COUNT(*) FROM {table}"), [], |r| r.get(0))?)
}

fn leftovers(conn: &Connection, check: &Check, problems: &mut Vec<String>) -> Result<()> {
    let columns = SCOPE_TABLES
        .iter()
        .map(|t| (t, "scope"))
        .chain(PROJECT_TABLES.iter().map(|t| (t, "project")));
    for (table, column) in columns {
        let sql = format!("SELECT COUNT(*) FROM {table} WHERE {column} = ?1");
        for key in check.map.keys() {
            let n: i64 = conn.query_row(&sql, [key], |r| r.get(0))?;
            if n > 0 {
                problems.push(format!("{table}.{column} still holds {key} in {n} rows"));
            }
        }
    }
    let mut stmt = conn.prepare("SELECT key FROM kv WHERE key LIKE '%:%'")?;
    for row in stmt.query_map([], |r| r.get::<_, String>(0))? {
        let key = row?;
        if rewrite_key(&key, check.map).is_some() {
            problems.push(format!("kv key {key} still holds an old board key"));
        }
    }
    Ok(())
}

fn table_names(conn: &Connection) -> Result<Vec<String>> {
    let mut stmt = conn.prepare(
        "SELECT name FROM sqlite_master WHERE type = 'table' AND name NOT LIKE 'sqlite_%'",
    )?;
    let names = stmt.query_map([], |r| r.get(0))?;
    Ok(names.collect::<std::result::Result<Vec<String>, _>>()?)
}

fn counts_equal(conn: &Connection, backup: &Path, problems: &mut Vec<String>) -> Result<()> {
    let before = open(&backup.join(DB_REL))?;
    for table in table_names(&before)? {
        if count(&before, &table)? != count(conn, &table)? {
            problems.push(format!("the row count of {table} changed"));
        }
    }
    Ok(())
}

/// The database checks (nothing when the home has no database).
pub fn check(check: &Check, problems: &mut Vec<String>) -> Result<()> {
    let file = check.home.join(DB_REL);
    if !file.is_file() {
        return Ok(());
    }
    if !is_intact(&file)? {
        problems.push("the database fails its integrity check".to_owned());
    }
    let conn = open(&file)?;
    leftovers(&conn, check, problems)?;
    match check.backup {
        Some(backup) if backup.join(DB_REL).is_file() => counts_equal(&conn, backup, problems),
        _ => Ok(()),
    }
}
