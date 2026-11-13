//! What a home holds, as values: every JSON file parsed, every file name, every database row.
//! The files a run adds on purpose (table, plan, journal, marker, locks) are left out.

use std::collections::{BTreeMap, BTreeSet};
use std::fmt::Write as _;
use std::fs;
use std::path::Path;

use rusqlite::types::Value as Cell;
use rusqlite::{Connection, OpenFlags};
use serde_json::Value;
use sha2::{Digest, Sha256};

/// A home as values.
#[derive(Debug, PartialEq)]
pub struct Snapshot {
    /// Every JSON file by relative path.
    pub json: BTreeMap<String, Value>,
    /// Every file, by relative path.
    pub files: BTreeSet<String>,
    /// Every row of every table, sorted, by table.
    pub db: BTreeMap<String, Vec<String>>,
}

fn all_files(root: &Path, rel: &str, out: &mut Vec<String>) {
    for item in fs::read_dir(root.join(rel)).unwrap() {
        let item = item.unwrap();
        let name = item.file_name().to_string_lossy().into_owned();
        let path = if rel.is_empty() {
            name
        } else {
            format!("{rel}/{name}")
        };
        if item.file_type().unwrap().is_dir() {
            all_files(root, &path, out);
        } else {
            out.push(path);
        }
    }
}

fn has_ext(name: &str, ext: &str) -> bool {
    Path::new(name)
        .extension()
        .is_some_and(|e| e.eq_ignore_ascii_case(ext))
}

fn is_left_out(rel: &str) -> bool {
    let name = rel.rsplit('/').next().unwrap();
    matches!(
        rel,
        "boards.json" | "board-ids.plan.json" | "board-ids.journal.json" | "MIGRATING"
    ) || has_ext(name, "lock")
        || has_ext(name, "tmp")
        || rel.starts_with("dashboard/dashboard.db")
}

fn rows_of(file: &Path) -> BTreeMap<String, Vec<String>> {
    let conn = Connection::open_with_flags(file, OpenFlags::SQLITE_OPEN_READ_ONLY).unwrap();
    let mut stmt = conn
        .prepare("SELECT name FROM sqlite_master WHERE type='table' AND name NOT LIKE 'sqlite_%'")
        .unwrap();
    let tables: Vec<String> = stmt
        .query_map([], |r| r.get(0))
        .unwrap()
        .map(Result::unwrap)
        .collect();
    let mut out = BTreeMap::new();
    for table in tables {
        let mut query = conn.prepare(&format!("SELECT * FROM {table}")).unwrap();
        let width = query.column_count();
        let mut rows: Vec<String> = query
            .query_map([], |r| {
                let cells: Vec<Cell> = (0..width).map(|i| r.get(i).unwrap()).collect();
                Ok(format!("{cells:?}"))
            })
            .unwrap()
            .map(Result::unwrap)
            .collect();
        rows.sort();
        out.insert(table, rows);
    }
    out
}

/// Reads the home.
pub fn take(home: &Path) -> Snapshot {
    let mut names = Vec::new();
    all_files(home, "", &mut names);
    names.retain(|n| !is_left_out(n));
    let mut json = BTreeMap::new();
    for name in &names {
        if has_ext(name, "json") {
            let text = fs::read_to_string(home.join(name)).unwrap();
            json.insert(name.clone(), serde_json::from_str(&text).unwrap());
        }
    }
    let db_file = home.join("dashboard").join("dashboard.db");
    let db = if db_file.is_file() {
        rows_of(&db_file)
    } else {
        BTreeMap::new()
    };
    Snapshot {
        json,
        files: names.into_iter().collect(),
        db,
    }
}

/// The sha256 of every file of the home, the marker and the database included.
pub fn hashes(home: &Path) -> BTreeMap<String, String> {
    let mut names = Vec::new();
    all_files(home, "", &mut names);
    names
        .into_iter()
        .map(|n| {
            let digest = Sha256::digest(fs::read(home.join(&n)).unwrap());
            let hex = digest.iter().fold(String::new(), |mut acc, b| {
                let _ = write!(acc, "{b:02x}");
                acc
            });
            (n, hex)
        })
        .collect()
}
