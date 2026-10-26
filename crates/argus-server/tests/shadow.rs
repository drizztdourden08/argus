//! The start of a shadow home: a copy of the Node database, config and process rules, only when
//! the shadow home has none, and the Node home left as it was.

use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use argus_core::Bus;
use argus_server::shadow::seed;
use argus_store::Db;
use argus_testkit::{TestOptions, TestServer, Tweak};
use tempfile::TempDir;

fn node_home(dir: &Path) -> (PathBuf, Db) {
    let home = dir.join("node");
    let db = Db::open(&home.join("dashboard.db"), Bus::new()).unwrap();
    db.kv_set("seen", "yes").unwrap();
    fs::write(home.join("config.json"), "{\"from\":\"node\"}").unwrap();
    fs::write(home.join("process-rules.json"), "{\"rules\":[]}").unwrap();
    (home, db)
}

#[test]
fn a_shadow_home_with_nothing_gets_the_database_the_config_and_the_rules() {
    let dir = TempDir::new().unwrap();
    let (node, _live) = node_home(dir.path());
    let shadow = dir.path().join("shadow");
    seed(&node, &shadow).unwrap();
    let copy = Db::open(&shadow.join("dashboard.db"), Bus::new()).unwrap();
    assert_eq!(copy.kv_get("seen").unwrap().as_deref(), Some("yes"));
    let config: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(shadow.join("config.json")).unwrap()).unwrap();
    assert_eq!(
        config,
        serde_json::json!({ "from": "node", "port": 47616 }),
        "the Node config, and the port of the shadow as its one change"
    );
    assert!(shadow.join("process-rules.json").is_file());
}

#[test]
fn what_the_shadow_home_already_has_is_kept() {
    let dir = TempDir::new().unwrap();
    let (node, _live) = node_home(dir.path());
    let shadow = dir.path().join("shadow");
    let own = Db::open(&shadow.join("dashboard.db"), Bus::new()).unwrap();
    own.kv_set("seen", "mine").unwrap();
    fs::write(shadow.join("config.json"), "{\"from\":\"shadow\"}").unwrap();
    seed(&node, &shadow).unwrap();
    assert_eq!(own.kv_get("seen").unwrap().as_deref(), Some("mine"));
    assert_eq!(
        fs::read_to_string(shadow.join("config.json")).unwrap(),
        "{\"from\":\"shadow\"}"
    );
    assert!(
        shadow.join("process-rules.json").is_file(),
        "the missing one is copied"
    );
}

#[test]
fn the_node_home_is_not_written_and_one_home_is_left_alone() {
    let dir = TempDir::new().unwrap();
    let (node, live) = node_home(dir.path());
    let names = |home: &Path| {
        let mut all: Vec<_> = fs::read_dir(home)
            .unwrap()
            .map(|e| e.unwrap().file_name())
            .collect();
        all.sort();
        all
    };
    let before = names(&node);
    seed(&node, &dir.path().join("shadow")).unwrap();
    assert_eq!(names(&node), before);
    seed(&node, &node).unwrap();
    assert_eq!(live.kv_get("seen").unwrap().as_deref(), Some("yes"));
}

#[test]
fn a_node_home_with_no_database_gives_an_empty_shadow() {
    let dir = TempDir::new().unwrap();
    let shadow = dir.path().join("shadow");
    seed(&dir.path().join("none"), &shadow).unwrap();
    assert!(!shadow.join("dashboard.db").exists());
}

#[tokio::test]
async fn serve_in_shadow_mode_starts_from_the_node_database() {
    let dir = TempDir::new().unwrap();
    let (node, _live) = node_home(dir.path());
    let tweak: Tweak = Arc::new(move |o| o.shadow_seed = Some(node.clone()));
    let t = TestServer::start(TestOptions {
        shadow: true,
        tweak: Some(tweak),
        ..TestOptions::default()
    })
    .await;
    assert_eq!(t.svc.db.kv_get("seen").unwrap().as_deref(), Some("yes"));
}
