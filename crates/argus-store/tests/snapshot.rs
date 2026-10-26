//! `snapshot`: a copy of a live (WAL) database, the source untouched, no half copy left.

use std::fs;

use argus_core::Bus;
use argus_store::{Db, snapshot};
use tempfile::TempDir;

#[test]
fn a_live_database_is_copied_with_what_is_still_in_its_wal() {
    let dir = TempDir::new().unwrap();
    let node = dir.path().join("node").join("dashboard.db");
    let live = Db::open(&node, Bus::new()).unwrap();
    live.kv_set("seen", "yes").unwrap();
    let copy = dir.path().join("shadow").join("dashboard.db");
    snapshot(&node, &copy).unwrap();
    let opened = Db::open(&copy, Bus::new()).unwrap();
    assert_eq!(opened.kv_get("seen").unwrap().as_deref(), Some("yes"));
    live.kv_set("later", "1").unwrap();
    assert_eq!(opened.kv_get("later").unwrap(), None, "a copy, not a link");
}

#[test]
fn a_missing_source_fails_and_leaves_no_file() {
    let dir = TempDir::new().unwrap();
    let copy = dir.path().join("shadow.db");
    assert!(snapshot(&dir.path().join("none.db"), &copy).is_err());
    let left: Vec<_> = fs::read_dir(dir.path()).unwrap().collect();
    assert!(left.is_empty(), "{left:?}");
}
