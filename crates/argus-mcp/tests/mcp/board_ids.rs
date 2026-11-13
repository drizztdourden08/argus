//! A board id or an old key names a board in every tool; the table is `boards.json`.
// test-tag: critical

use argus_mcp::MemoryBackend;
use serde_json::{Value, json};

use crate::support::{Rig, fixture, run_lines, run_rows};

fn table() -> Value {
    json!({
        "version": 1,
        "state": "active",
        "boards": {
            "alpha": { "name": "Alpha Game", "aliases": ["old-alpha"], "createdAt": "2026-10-22T10:00:00.000Z" },
            "beta": { "name": "Beta", "aliases": [], "createdAt": "2026-10-22T10:00:00.000Z" }
        }
    })
}

fn rig(home: &tempfile::TempDir) -> Rig {
    std::fs::write(home.path().join("boards.json"), table().to_string()).expect("boards.json");
    Rig::over(
        MemoryBackend::new(fixture())
            .with_runs(run_rows(), run_lines())
            .with_home(home.path().to_path_buf()),
    )
}

#[tokio::test]
async fn an_old_key_reads_the_same_board_as_its_id() {
    let home = tempfile::tempdir().expect("home");
    let rig = rig(&home);
    let by_id = rig.ok("list_items", json!({ "project": "alpha" })).await;
    let by_alias = rig
        .ok("list_items", json!({ "project": "old-alpha" }))
        .await;
    assert_eq!(by_alias, by_id);
    assert_eq!(by_alias["items"][0]["id"], "I-1");
}

#[tokio::test]
async fn an_unknown_board_lists_the_known_ones_as_name_and_id() {
    let home = tempfile::tempdir().expect("home");
    let rig = rig(&home);
    let e = rig
        .err("list_items", json!({ "project": "nope" }), "not_found")
        .await;
    assert_eq!(
        e["details"],
        json!({ "known": ["Alpha Game (alpha)", "Beta (beta)"] })
    );
}

#[tokio::test]
async fn portfolio_overview_carries_id_name_and_aliases() {
    let home = tempfile::tempdir().expect("home");
    let rig = rig(&home);
    let out = rig.ok("portfolio_overview", json!({})).await;
    let cards = out["cards"].as_array().expect("cards");
    let alpha = cards
        .iter()
        .find(|c| c["id"] == "alpha")
        .expect("the alpha card");
    assert_eq!(alpha["name"], "Alpha Game");
    assert_eq!(alpha["aliases"], json!(["old-alpha"]));
    assert_eq!(alpha["threads"], 2);
}
