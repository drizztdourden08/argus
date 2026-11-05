//! I-104: `coord demo seed` fills a preview home with an example of every record kind, refuses
//! the live home and an unset one, and refuses to seed twice.

// test-tag: critical

use std::collections::BTreeSet;
use std::fs;

use argus_coord::Env;
use serde_json::Value;

use crate::common::{Home, coord};

fn board(home: &Home, key: &str) -> Value {
    let file = home.coord_home().join(key).join("board.json");
    let text = fs::read_to_string(file).expect("board file");
    serde_json::from_str(&text).expect("board json")
}

fn records<'a>(board: &'a Value, collection: &str) -> Vec<&'a Value> {
    match &board[collection] {
        Value::Object(map) => map.values().collect(),
        Value::Array(list) => list.iter().collect(),
        _ => Vec::new(),
    }
}

fn seen(board: &Value, collection: &str, field: &str) -> BTreeSet<String> {
    records(board, collection)
        .iter()
        .filter_map(|r| r[field].as_str().map(str::to_owned))
        .collect()
}

fn has_all(found: &BTreeSet<String>, wanted: &[&str], what: &str) {
    for word in wanted {
        assert!(found.contains(*word), "no {what} {word}: found {found:?}");
    }
}

fn seeded() -> (Home, String) {
    let home = Home::new();
    let cwd = home.folder("cwd");
    let ran = coord(&["demo", "seed"], home.env(), &cwd);
    (home, format!("{}\n{}", ran.out, ran.err))
}

#[test]
fn the_seed_has_a_record_of_every_kind() {
    let (home, said) = seeded();
    let demo = board(&home, "demo");
    let types = seen(&demo, "items", "type");
    let kinds = [
        "epic",
        "story",
        "task",
        "bug",
        "debt",
        "test-plan",
        "test-run",
    ];
    has_all(&types, &kinds, "item type");
    let stages: BTreeSet<String> = records(&demo, "items")
        .iter()
        .filter(|i| i["type"] == "test-run")
        .filter_map(|i| i["stage"].as_str().map(str::to_owned))
        .collect();
    has_all(
        &stages,
        &["draft", "ready", "running", "reported", "passed"],
        "run stage",
    );
    has_all(
        &seen(&demo, "tests", "mode"),
        &["auto", "manual"],
        "test mode",
    );
    assert!(said.contains("seeded"), "{said}");
}

#[test]
fn the_seed_has_agents_pull_requests_requests_and_a_release() {
    let (home, said) = seeded();
    let demo = board(&home, "demo");
    let agents = seen(&demo, "agents", "status");
    assert!(agents.len() >= 3, "agent statuses: {agents:?}");
    has_all(&agents, &["done", "stopped"], "agent status");
    has_all(
        &seen(&demo, "prs", "status"),
        &["open", "merged", "closed"],
        "pull request",
    );
    for collection in [
        "entries",
        "launchers",
        "runDefs",
        "worktreeLinks",
        "log",
        "actions",
    ] {
        assert!(
            !records(&demo, collection).is_empty(),
            "{collection} is empty:\n{said}"
        );
    }
    let ledger = board(&home, "_portfolio");
    let to: BTreeSet<String> = records(&ledger, "deps")
        .iter()
        .filter_map(|d| d["to"]["project"].as_str().map(str::to_owned))
        .collect();
    has_all(
        &to,
        &["tessera", "brock", "archipelia", "demo"],
        "request to",
    );
    let releases = home.coord_home().join("releases").join("demo");
    assert!(
        releases
            .join("latest")
            .join("0.9.0")
            .join("release.json")
            .exists()
    );
    assert!(said.contains("seeded"), "{said}");
}

#[test]
fn a_second_seed_is_refused_and_the_first_had_no_failed_step() {
    let (home, said) = seeded();
    let again = coord(&["demo", "seed"], home.env(), &home.folder("cwd"));
    assert_eq!(again.code, 1, "{}", again.err);
    assert!(again.err.contains("already exists"), "{}", again.err);
    assert!(!said.contains("did not apply"), "{said}");
}

#[test]
fn the_seed_refuses_an_unset_home_and_the_live_home() {
    let home = Home::new();
    let cwd = home.folder("cwd");
    let unset = coord(&["demo", "seed"], Env::of(&[]), &cwd);
    assert_eq!(unset.code, 1);
    assert!(unset.err.contains("COORD_HOME"), "{}", unset.err);
    let user = home.folder("user");
    let user_text = user.to_string_lossy().into_owned();
    for coord_home in [user.join(".coord"), user.join(".coord").join("nested")] {
        let text = coord_home.to_string_lossy().into_owned();
        let env = Env::of(&[("USERPROFILE", &user_text), ("COORD_HOME", &text)]);
        let ran = coord(&["demo", "seed"], env, &cwd);
        assert_eq!(ran.code, 1, "{}", ran.err);
        assert!(ran.err.contains("live home"), "{}", ran.err);
        assert!(!coord_home.exists(), "{} was written", coord_home.display());
    }
}
