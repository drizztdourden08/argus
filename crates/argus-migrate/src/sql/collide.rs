//! A rollback precheck: the rows the id-to-key rewrite would land on top of. Every rewrite that
//! would hit a unique key (the kv key, docs by scope, collection and id, launch definitions by
//! scope and name, runs by scope and run id) is listed, so the run stops before it writes.

use std::path::Path;

use rusqlite::{Connection, OpenFlags};

use super::kv::rewrite_key;
use crate::error::Result;
use crate::map::IdMap;

/// The tables with a unique key made of the scope and these columns.
const UNIQUE: [(&str, &[&str]); 3] = [
    ("docs", &["coll", "id"]),
    ("launch_defs", &["name"]),
    ("runs", &["rid"]),
];

fn scoped(conn: &Connection, table: &str, cols: &[&str], from: &str, to: &str) -> Result<usize> {
    let on: Vec<String> = cols.iter().map(|c| format!("a.{c} = b.{c}")).collect();
    let sql = format!(
        "SELECT COUNT(*) FROM {table} a JOIN {table} b ON {} WHERE a.scope = ?1 AND b.scope = ?2",
        on.join(" AND ")
    );
    let n: i64 = conn.query_row(&sql, [from, to], |r| r.get(0))?;
    Ok(usize::try_from(n).unwrap_or(0))
}

fn kv_collisions(conn: &Connection, map: &IdMap, out: &mut Vec<String>) -> Result<()> {
    let mut keys: Vec<String> = Vec::new();
    let mut stmt = conn.prepare("SELECT key FROM kv WHERE key LIKE '%:%'")?;
    for row in stmt.query_map([], |r| r.get(0))? {
        keys.push(row?);
    }
    drop(stmt);
    let mut find = conn.prepare("SELECT COUNT(*) FROM kv WHERE key = ?1")?;
    for old in keys {
        let Some(new) = rewrite_key(&old, map) else {
            continue;
        };
        let n: i64 = find.query_row([&new], |r| r.get(0))?;
        if n > 0 {
            out.push(format!("kv key {old} would overwrite {new}"));
        }
    }
    Ok(())
}

/// The collisions a rewrite by `map` would hit in the database, as readable lines (none when the
/// file is absent).
pub fn collisions(db: &Path, map: &IdMap) -> Result<Vec<String>> {
    let mut out = Vec::new();
    if !db.is_file() {
        return Ok(out);
    }
    let conn = Connection::open_with_flags(db, OpenFlags::SQLITE_OPEN_READ_ONLY)?;
    for (from, to) in map {
        for (table, cols) in UNIQUE {
            let n = scoped(&conn, table, cols, from, to)?;
            if n > 0 {
                out.push(format!(
                    "{table}: {n} rows of scope {from} collide with scope {to}"
                ));
            }
        }
    }
    kv_collisions(&conn, map, &mut out)?;
    Ok(out)
}
