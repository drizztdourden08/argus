//! The start of a shadow home: a copy of the Node database, config and process rules, only when
//! the shadow home has none, and the Node home left as it was.
//! K3, K1: seeding a shadow home from the Node home in temp folders: nothing the shadow has is overwritten and the Node home is never written.
// test-tag: critical

use std::fs;
use std::path::Path;

use crate::node_home::node_home;
use argus_core::Bus;
use argus_server::shadow::{board_findings, seed};
use argus_store::Db;
use tempfile::TempDir;

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
        serde_json::json!({ "from": "node", "port": 47616, "shadow": true }),
        "the Node config, the port of the shadow and the shadow marker as its changes"
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
fn a_board_of_nul_bytes_is_a_finding_of_its_own_and_a_good_board_is_not() {
    let coord = TempDir::new().unwrap();
    for (key, bytes) in [("tessera", vec![0u8; 64]), ("argus", b"{}".to_vec())] {
        fs::create_dir_all(coord.path().join(key)).unwrap();
        fs::write(coord.path().join(key).join("board.json"), bytes).unwrap();
    }
    let found = board_findings(coord.path());
    assert_eq!(found.len(), 1, "{found:?}");
    assert!(
        found[0].contains("board \"tessera\" is damaged"),
        "{found:?}"
    );
    assert!(found[0].contains("64 NUL bytes"), "{found:?}");
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

#[test]
fn preparing_a_shadow_seeds_and_marks_it_and_never_marks_the_node_home() {
    let dir = TempDir::new().unwrap();
    let (node, _live) = node_home(dir.path());
    let coord = dir.path().join("coord");
    let shadow = argus_core::Paths::new(&coord, Some(dir.path().join("shadow")));
    argus_server::shadow::prepare(&shadow, Some(&node)).unwrap();
    assert!(argus_server::shadow::marked(&shadow.config));
    assert!(shadow.db.is_file());
    assert!(!argus_server::shadow::marked(&node.join("config.json")));
    let live = argus_core::Paths::new(&coord, None);
    argus_server::shadow::prepare(&live, None).unwrap();
    assert!(
        !live.config.exists(),
        "the live default home is never marked"
    );
}
