//! The Desktop sessions as a list, read only: one row per `local_<uuid>` record, newest first. A session
//! kept under two accounts has two records; they fold into one row, archived when any record says so.

use std::collections::HashMap;
use std::fs;
use std::path::Path;

use argus_core::clock::iso_from_ms;
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::session_map::record_files;

/// One Desktop session.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SessionRow {
    /// `local_<uuid>`.
    pub local_id: String,
    pub title: Option<String>,
    pub cwd: Option<String>,
    pub archived: bool,
    /// The CLI session id the Desktop app resumes now.
    pub cli_session_id: Option<String>,
    /// The last activity (ISO), empty when the record has none.
    pub updated_at: String,
    #[serde(skip)]
    updated_ms: i64,
}

/// The fields of a record the list reads.
#[derive(Deserialize, Default)]
#[serde(default, rename_all = "camelCase")]
struct Fields {
    title: Option<String>,
    cwd: Option<String>,
    is_archived: bool,
    cli_session_id: Option<String>,
    last_activity_at: Value,
}

fn millis(value: &Value) -> i64 {
    // The record keeps epoch milliseconds, whole or with a fraction; the fraction is dropped.
    let text = value
        .as_number()
        .map(ToString::to_string)
        .unwrap_or_default();
    text.split('.')
        .next()
        .and_then(|whole| whole.parse().ok())
        .unwrap_or(0)
}

fn row_of(local_id: &str, fields: Fields) -> SessionRow {
    let updated_ms = millis(&fields.last_activity_at);
    SessionRow {
        local_id: local_id.to_owned(),
        title: fields.title.filter(|t| !t.trim().is_empty()),
        cwd: fields.cwd,
        archived: fields.is_archived,
        cli_session_id: fields.cli_session_id.filter(|id| !id.is_empty()),
        updated_at: if updated_ms > 0 {
            iso_from_ms(updated_ms)
        } else {
            String::new()
        },
        updated_ms,
    }
}

/// Folds the second record of a session into the first: the newer one gives the fields, and an archived one
/// archives the session.
fn fold(kept: SessionRow, other: SessionRow) -> SessionRow {
    let archived = kept.archived || other.archived;
    let newer = if other.updated_ms > kept.updated_ms {
        other
    } else {
        kept
    };
    SessionRow { archived, ..newer }
}

/// The sessions of the Desktop app in `app_data`, newest first, at most `limit`.
#[must_use]
pub fn sessions(app_data: &Path, limit: usize) -> Vec<SessionRow> {
    let mut by_id: HashMap<String, SessionRow> = HashMap::new();
    for path in record_files(app_data) {
        let Some(local) = path.file_stem().map(|s| s.to_string_lossy().into_owned()) else {
            continue;
        };
        let Some(fields) = fs::read_to_string(&path)
            .ok()
            .and_then(|text| serde_json::from_str::<Fields>(&text).ok())
        else {
            continue;
        };
        let row = row_of(&local, fields);
        let merged = match by_id.remove(&local) {
            Some(kept) => fold(kept, row),
            None => row,
        };
        by_id.insert(local, merged);
    }
    let mut rows: Vec<SessionRow> = by_id.into_values().collect();
    rows.sort_by(|a, b| {
        b.updated_ms
            .cmp(&a.updated_ms)
            .then_with(|| a.local_id.cmp(&b.local_id))
    });
    rows.truncate(limit);
    rows
}

// test-tag: critical
#[cfg(test)]
mod tests {
    use super::*;
    use crate::fixtures::write_record as write;

    #[test]
    fn the_list_is_newest_first_and_reads_the_fields() {
        let dir = tempfile::tempdir().expect("dir");
        let old = r#"{"title":"Old","cwd":"X:/a","cliSessionId":"c1","lastActivityAt":1000}"#;
        let new = r#"{"title":"New","isArchived":true,"lastActivityAt":2000.0}"#;
        write(dir.path(), ("a", "o"), "local_old.json", old);
        write(dir.path(), ("a", "o"), "local_new.json", new);
        let rows = sessions(dir.path(), 10);
        let ids: Vec<&str> = rows.iter().map(|r| r.local_id.as_str()).collect();
        assert_eq!(ids, ["local_new", "local_old"]);
        assert!(rows[0].archived);
        assert_eq!(rows[1].title.as_deref(), Some("Old"));
        assert_eq!(rows[1].cwd.as_deref(), Some("X:/a"));
        assert_eq!(rows[1].cli_session_id.as_deref(), Some("c1"));
        assert_eq!(rows[1].updated_at, "1970-01-01T00:00:01.000Z");
    }

    #[test]
    fn two_records_of_one_session_fold_into_one_archived_by_either() {
        let dir = tempfile::tempdir().expect("dir");
        let live = r#"{"title":"Newer","lastActivityAt":5000}"#;
        let archived = r#"{"title":"Older","isArchived":true,"lastActivityAt":4000}"#;
        write(dir.path(), ("a1", "o1"), "local_x.json", archived);
        write(dir.path(), ("a2", "o2"), "local_x.json", live);
        let rows = sessions(dir.path(), 10);
        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0].title.as_deref(), Some("Newer"));
        assert!(rows[0].archived);
    }

    #[test]
    fn the_limit_cuts_the_list_and_a_broken_record_is_skipped() {
        let dir = tempfile::tempdir().expect("dir");
        write(
            dir.path(),
            ("a", "o"),
            "local_a.json",
            r#"{"lastActivityAt":1}"#,
        );
        write(
            dir.path(),
            ("a", "o"),
            "local_b.json",
            r#"{"lastActivityAt":2}"#,
        );
        write(dir.path(), ("a", "o"), "local_c.json", "not json");
        let rows = sessions(dir.path(), 1);
        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0].local_id, "local_b");
    }
}
