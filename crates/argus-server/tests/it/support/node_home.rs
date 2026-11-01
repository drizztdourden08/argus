//! Shared by the shadow specs: a Node home with a database, a config and process rules.

use std::fs;
use std::path::{Path, PathBuf};

use argus_core::Bus;
use argus_store::Db;

pub fn node_home(dir: &Path) -> (PathBuf, Db) {
    let home = dir.join("node");
    let db = Db::open(&home.join("dashboard.db"), Bus::new()).unwrap();
    db.kv_set("seen", "yes").unwrap();
    fs::write(home.join("config.json"), "{\"from\":\"node\"}").unwrap();
    fs::write(home.join("process-rules.json"), "{\"rules\":[]}").unwrap();
    (home, db)
}
