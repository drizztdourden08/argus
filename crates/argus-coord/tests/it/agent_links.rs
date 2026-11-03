//! `coord agent start` and `set` as linked records: a start needs an item and a place, an agent
//! carries `items`, `master` and `blockedBy`, and the record a hook made stays as it was until a
//! lead binds it. I-85 P2.

// test-tag: critical

use crate::agent_common::{As, Rig, agent, hook_agent};
use argus_core::clock::now_iso;
use serde_json::{Value, json};

const WHY: &str = "known cause, one module";
const NO_ITEM: &str = "an agent works on an item: add --item I-n";
const NO_PLACE: &str = "an agent runs somewhere: add --worktree <path>";

fn rig() -> Rig {
    let rig = Rig::new();
    rig.with_item();
    rig.with_item_status("I-2", "doing");
    rig
}

fn base<'a>(name: &'a str, extra: &[&'a str]) -> Vec<&'a str> {
    let mut args = vec![
        "agent",
        "start",
        name,
        "do the work",
        "--model",
        "sonnet",
        "--why",
        WHY,
    ];
    args.extend(extra);
    args
}

#[test]
fn a_start_without_an_item_or_without_a_place_is_refused_with_the_fix_and_makes_no_record() {
    let rig = rig();
    rig.refused(&base("a1", &["--master"]), As::Lead, NO_ITEM);
    rig.refused(&base("a1", &["--item", "I-1"]), As::Lead, NO_PLACE);
    rig.refused(
        &base("a1", &["--item", "I-1", "--master", "--worktree", "w"]),
        As::Lead,
        "exclude each other",
    );
    rig.refused(
        &base("a1", &["--item", "I-9", "--master"]),
        As::Lead,
        "no item \"I-9\"",
    );
    assert_eq!(
        rig.board()["agents"]
            .as_object()
            .map_or(0, serde_json::Map::len),
        0
    );
}

#[test]
fn a_start_on_two_items_in_its_own_folder_records_items_the_place_and_what_it_waits_on() {
    let rig = rig();
    rig.ok(
        &base(
            "a1",
            &[
                "--item",
                "I-1",
                "--item",
                "I-2",
                "--worktree",
                "auto",
                "--blocked-by",
                "I-2",
            ],
        ),
        As::Lead,
    );
    let a = agent(&rig, "a1");
    assert_eq!(a["items"], json!(["I-1", "I-2"]));
    assert_eq!(a["item"], "I-1");
    assert_eq!(a["worktree"], Value::Null);
    assert_eq!(a["master"], false);
    assert_eq!(a["blockedBy"], json!(["I-2"]));
}

#[test]
fn a_start_on_the_main_checkout_says_master_and_prints_the_items() {
    let rig = rig();
    let ran = rig.ok(&base("a1", &["--item", "I-1,I-2", "--master"]), As::Lead);
    assert!(
        ran.out.contains("started on coordinator for I-1, I-2"),
        "{}",
        ran.out
    );
    let a = agent(&rig, "a1");
    assert_eq!(a["master"], true);
    assert_eq!(a["items"], json!(["I-1", "I-2"]));
}

#[test]
fn a_start_with_a_named_worktree_keeps_the_path() {
    let rig = rig();
    rig.ok(
        &base(
            "a1",
            &[
                "--item",
                "I-1",
                "--worktree",
                "X:/p/.claude/worktrees/agent-a1",
            ],
        ),
        As::Lead,
    );
    let a = agent(&rig, "a1");
    assert_eq!(a["worktree"], "X:/p/.claude/worktrees/agent-a1");
    assert_eq!(a["master"], false);
}

#[test]
fn a_blocker_is_an_id_of_the_board_never_text() {
    let rig = rig();
    rig.refused(
        &base(
            "a1",
            &["--item", "I-1", "--master", "--blocked-by", "the key"],
        ),
        As::Lead,
        "is not an id",
    );
}

#[test]
fn an_item_may_be_written_with_the_letters_of_its_type_and_the_wrong_letters_are_refused() {
    let rig = rig();
    rig.edit(|b| b["items"]["I-2"]["type"] = json!("bug"));
    rig.refused(
        &base("a1", &["--item", "S-2", "--master"]),
        As::Lead,
        "S-2 is not a story: I-2 is a bug, B-2.",
    );
    rig.refused(
        &base("a1", &["--item", "B-9", "--master"]),
        As::Lead,
        "no item \"B-9\"",
    );
    assert_eq!(
        rig.board()["agents"]
            .as_object()
            .map_or(0, serde_json::Map::len),
        0
    );
    rig.ok(&base("a1", &["--item", "B-2", "--master"]), As::Lead);
    let a = agent(&rig, "a1");
    assert_eq!(a["items"], json!(["I-2"]));
    assert_eq!(a["item"], "I-2");
    let id = a["id"].as_str().unwrap_or_default().to_owned();
    rig.ok(&["agent", "set", &id, "--add-item", "K-1"], As::Lead);
    assert_eq!(agent(&rig, "a1")["items"], json!(["I-2", "I-1"]));
    rig.refused(
        &["agent", "set", &id, "--item", "S-2"],
        As::Lead,
        "S-2 is not a story: I-2 is a bug, B-2.",
    );
    assert_eq!(agent(&rig, "a1")["items"], json!(["I-2", "I-1"]));
}

#[test]
fn the_record_a_hook_made_has_no_item_and_a_lead_start_binds_it_and_fills_the_links() {
    let rig = rig();
    rig.edit(|b| {
        let mut rec = hook_agent("pad-overlay", "sonnet");
        rec["claudeId"] = json!("hk-90");
        b["agents"]["A-90"] = rec;
    });
    assert_eq!(rig.board()["agents"]["A-90"].get("items"), None);
    rig.ok(
        &[
            "agent",
            "start",
            "pad-overlay",
            "Rebuild the pad overlay screen",
            "--model",
            "sonnet",
            "--why",
            WHY,
            "--item",
            "I-2",
            "--worktree",
            "auto",
        ],
        As::Lead,
    );
    let a = &rig.board()["agents"]["A-90"];
    assert_eq!(a["items"], json!(["I-2"]));
    assert_eq!(a["item"], "I-2");
    assert_eq!(a["leadBound"], true);
}

#[test]
fn set_adds_an_item_clears_the_list_moves_the_place_and_replaces_the_blockers() {
    let rig = rig();
    rig.ok(&base("a1", &["--item", "I-1", "--master"]), As::Lead);
    let id = agent(&rig, "a1")["id"].as_str().unwrap().to_owned();
    rig.ok(&["agent", "set", &id, "--add-item", "I-2"], As::Lead);
    assert_eq!(agent(&rig, "a1")["items"], json!(["I-1", "I-2"]));
    rig.ok(&["agent", "set", &id, "--worktree", "agent-a1"], As::Lead);
    let moved = agent(&rig, "a1");
    assert_eq!(
        (moved["worktree"].as_str(), moved["master"].as_bool()),
        (Some("agent-a1"), Some(false))
    );
    rig.ok(&["agent", "set", &id, "--master"], As::Lead);
    assert_eq!(agent(&rig, "a1")["worktree"], Value::Null);
    rig.ok(&["agent", "set", &id, "--blocked-by", "I-2"], As::Lead);
    assert_eq!(agent(&rig, "a1")["blockedBy"], json!(["I-2"]));
    rig.ok(&["agent", "set", &id, "--blocked-by", ""], As::Lead);
    assert_eq!(agent(&rig, "a1")["blockedBy"], json!([]));
    rig.ok(&["agent", "set", &id, "--item", ""], As::Lead);
    let cleared = agent(&rig, "a1");
    assert_eq!(
        (cleared["items"].clone(), cleared["item"].clone()),
        (json!([]), Value::Null)
    );
    rig.refused(
        &[
            "agent",
            "set",
            &id,
            "--item",
            "I-1",
            "--master",
            "--worktree",
            "w",
        ],
        As::Lead,
        "exclude each other",
    );
}

#[test]
fn an_old_record_with_only_item_lists_it_and_set_item_keeps_both_fields_one_fact() {
    let rig = rig();
    rig.edit(|b| {
        let at = now_iso();
        b["agents"]["A-5"] = json!({
            "id": "A-5", "createdAt": at, "createdBy": "worker", "updatedAt": at,
            "updatedBy": "worker", "rank": 5000, "name": "old", "thread": "worker",
            "task": "an older record", "model": "sonnet", "modelWhy": null, "kind": "build",
            "item": "I-1", "branch": null, "worktree": null, "status": "working",
            "note": null, "result": null, "startedAt": at, "finishedAt": null, "rev": 1,
        });
    });
    rig.ok(&["agent", "set", "A-5", "--add-item", "I-2"], As::Lead);
    let a = &rig.board()["agents"]["A-5"];
    assert_eq!(a["items"], json!(["I-1", "I-2"]));
    assert_eq!(a["item"], "I-1");
}
