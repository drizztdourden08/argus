//! T2: after a migration the counts are the same and no old key is left as a key.

use argus_core::boards;
use argus_migrate::{Mode, run};
use rusqlite::Connection;

use crate::fixture::{Fixture, KEYS, snapshot};

fn count(conn: &Connection, sql: &str) -> i64 {
    conn.query_row(sql, [], |r| r.get(0)).unwrap()
}

fn assert_no_key_in_db(home: &std::path::Path) {
    let conn = Connection::open(home.join("dashboard").join("dashboard.db")).unwrap();
    for key in KEYS {
        let q = |sql: &str| count(&conn, &sql.replace("{k}", key));
        assert_eq!(q("SELECT COUNT(*) FROM docs WHERE scope='{k}'"), 0, "docs {key}");
        assert_eq!(q("SELECT COUNT(*) FROM activity WHERE project='{k}'"), 0, "activity {key}");
        assert_eq!(q("SELECT COUNT(*) FROM notifications WHERE project='{k}'"), 0, "notes {key}");
        assert_eq!(
            q("SELECT COUNT(*) FROM kv WHERE key LIKE '%:{k}' OR key LIKE '%:{k}:%'"),
            0,
            "kv {key}"
        );
    }
}

// test-tag: critical
#[test]
fn counts_are_kept_and_no_old_key_is_left() {
    let fx = Fixture::new();
    let before = snapshot::take(&fx.home);
    run(&fx.options(Mode::Migrate)).unwrap();
    let after = snapshot::take(&fx.home);
    for (table, rows) in &before.db {
        assert_eq!(rows.len(), after.db[table].len(), "rows of {table}");
    }
    let table = boards::load(&fx.home).unwrap();
    assert_eq!(table.boards.len(), KEYS.len());
    for key in KEYS {
        let id = boards::resolve(&fx.home, key).unwrap();
        assert_ne!(id, key);
        assert!(table.boards[&id].aliases.iter().any(|a| a == key), "alias {key}");
        assert!(!fx.home.join(key).exists(), "folder {key} renamed");
        assert!(fx.home.join(&id).join("board.json").is_file());
    }
    assert_no_key_in_db(&fx.home);
    let portfolio = &after.json["_portfolio/board.json"];
    for key in portfolio["projects"].as_object().unwrap().keys() {
        assert!(!KEYS.contains(&key.as_str()), "project key {key}");
    }
}
