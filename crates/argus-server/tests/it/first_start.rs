//! The first start of Argus on a home last opened by Node: one backup of the database before
//! anything writes, then a marker, and never again.
//! K3: the one backup of the Node database before the first write.
// test-tag: critical

use std::fs;
use std::path::Path;

use argus_core::{Bus, Paths};
use argus_server::first_start::{MARKER_FILE, back_up_node_database};
use argus_store::Db;
use tempfile::TempDir;

fn node_home(dir: &Path) -> Paths {
    let home = dir.join("dashboard");
    let db = Db::open(&home.join("dashboard.db"), Bus::new()).unwrap();
    db.kv_set("seen", "by node").unwrap();
    Paths::new(dir.to_path_buf(), Some(home))
}

fn backups(home: &Path) -> Vec<String> {
    let mut names: Vec<String> = fs::read_dir(home)
        .unwrap()
        .map(|e| e.unwrap().file_name().to_string_lossy().into_owned())
        .filter(|n| n.starts_with("dashboard.db.node-"))
        .collect();
    names.sort();
    names
}

#[test]
fn the_first_start_takes_a_backup_of_the_node_database_and_sets_the_marker() {
    let dir = TempDir::new().unwrap();
    let paths = node_home(dir.path());
    let made = back_up_node_database(&paths).unwrap().expect("a backup");
    assert!(paths.home.join(MARKER_FILE).is_file());
    assert_eq!(backups(&paths.home).len(), 1);
    let name = made.file_name().unwrap().to_string_lossy().into_owned();
    let stamp = name.strip_prefix("dashboard.db.node-").unwrap();
    assert!(
        stamp.len() == 16 && stamp.ends_with('Z') && stamp.as_bytes()[8] == b'T',
        "{stamp}"
    );
    let copy = Db::open(&made, Bus::new()).unwrap();
    assert_eq!(copy.kv_get("seen").unwrap().as_deref(), Some("by node"));
}

#[test]
fn a_home_with_the_marker_is_not_backed_up_again() {
    let dir = TempDir::new().unwrap();
    let paths = node_home(dir.path());
    back_up_node_database(&paths).unwrap();
    assert!(back_up_node_database(&paths).unwrap().is_none());
    assert_eq!(backups(&paths.home).len(), 1);
}

#[test]
fn a_home_with_no_database_only_gets_the_marker() {
    let dir = TempDir::new().unwrap();
    let paths = Paths::new(dir.path().to_path_buf(), Some(dir.path().join("fresh")));
    assert!(back_up_node_database(&paths).unwrap().is_none());
    assert!(paths.home.join(MARKER_FILE).is_file());
    assert_eq!(backups(&paths.home).len(), 0);
}
