//! The portfolio: the owner's posts handed down to the project boards, the project list, the
//! cards, and the proof images of a test.
//! K1, K2, K3: the hand-down, the project list, the cards and the proofs.

// test-tag: critical

use crate::common::{alpha, home_with_alpha, portfolio_of};
use argus_board::BoardError;
use argus_board::portfolio::{import_portfolio_entries, project_keys, summarize};
use argus_board::proofs::keep_proof_bytes;
use argus_board::store::read_board;
use serde_json::{Value, json};

fn portfolio() -> Value {
    portfolio_of(json!({"alpha": {"key": "alpha", "name": "Alpha", "root": null}}))
}

fn post(id: &str, kind: &str, project: &str) -> Value {
    json!({
        "id": id, "kind": kind, "author": "human", "project": project, "text": "Please review the API",
        "createdAt": argus_core::clock::now_iso(),
    })
}

#[test]
fn a_decision_is_handed_down_to_the_lead_of_the_project() {
    let home = home_with_alpha();
    let mut p = portfolio();
    let out = import_portfolio_entries(home.path(), &mut p, vec![post("O-1", "decision", "alpha")])
        .expect("imported");
    assert_eq!(out.imported, 1);
    assert_eq!(
        out.results,
        vec![("O-1".to_owned(), "delivered".to_owned())]
    );
    assert_eq!(p["entries"]["O-1"]["deliveredTo"], json!(["alpha"]));
    assert_eq!(p["kind"], "portfolio");
    let board = read_board(home.path(), "alpha")
        .expect("read")
        .expect("a board");
    let made = board["entries"]
        .as_object()
        .and_then(|m| m.values().find(|e| e["ref"] == "O-1"))
        .cloned()
        .expect("an entry on the board");
    assert_eq!(made["kind"], "decision");
    assert_eq!(made["status"], "pending");
    assert_eq!(made["thread"], "coordinator");
    let inbox = board["inbox"]["coordinator"]
        .as_array()
        .cloned()
        .unwrap_or_default();
    assert!(inbox.iter().any(|m| {
        m["text"]
            .as_str()
            .is_some_and(|t| t.contains("Run coord entry pick"))
    }));
    let note = &board["log"]["portfolio"]["entries"][0];
    assert!(
        note["text"]
            .as_str()
            .is_some_and(|t| t.contains("decision from the Overlord"))
    );
}

#[test]
fn a_known_id_is_reported_and_a_foreign_kind_is_refused() {
    let home = home_with_alpha();
    let mut p = portfolio();
    import_portfolio_entries(home.path(), &mut p, vec![post("O-1", "followup", "alpha")])
        .expect("first");
    let again = import_portfolio_entries(
        home.path(),
        &mut p,
        vec![
            post("O-1", "followup", "alpha"),
            post("O-2", "shout", "alpha"),
        ],
    )
    .expect("second");
    assert_eq!(again.imported, 0);
    assert_eq!(
        again.results,
        vec![
            ("O-1".to_owned(), "delivered".to_owned()),
            ("O-2".to_owned(), "refused".to_owned())
        ]
    );
}

#[test]
fn a_post_to_all_waits_for_a_project_that_has_no_board_yet() {
    let home = home_with_alpha();
    let mut p = portfolio();
    p["projects"]["ghost"] = json!({"key": "ghost", "name": "Ghost", "root": null});
    let out = import_portfolio_entries(home.path(), &mut p, vec![post("O-3", "followup", "all")])
        .expect("imported");
    assert_eq!(
        out.results,
        vec![("O-3".to_owned(), "waiting for a board".to_owned())]
    );
    assert_eq!(p["entries"]["O-3"]["pendingFor"], json!(["ghost"]));
    assert_eq!(p["entries"]["O-3"]["deliveredTo"], json!(["alpha"]));
}

#[test]
fn the_project_list_has_the_registered_and_the_found_boards_without_the_retired() {
    let home = home_with_alpha();
    let mut p = portfolio();
    p["projects"]["old"] = json!({"key": "old", "name": "Old", "retired": true});
    p["projects"]["beta"] = json!({"key": "beta", "name": "Beta"});
    std::fs::create_dir_all(home.path().join("_portfolio")).expect("dir");
    std::fs::write(home.path().join("_portfolio").join("board.json"), "{}").expect("write");
    assert_eq!(project_keys(home.path(), &p), vec!["alpha", "beta"]);
}

#[test]
fn the_card_of_a_running_project_counts_what_needs_the_owner() {
    let home = home_with_alpha();
    let board = alpha();
    let card = summarize(home.path(), "alpha", &portfolio(), Some(&board));
    assert_eq!(card["state"], "running");
    assert_eq!(card["name"], "Alpha");
    assert_eq!(card["url"], "/p/alpha/");
    assert_eq!(card["coordinator"]["session"], "Alpha");
    assert_eq!(card["counts"]["threads"], 1);
    assert_eq!(card["counts"]["needsYou"], 3);
    assert_eq!(card["counts"]["itemsOpen"], 4);
    assert_eq!(card["counts"]["testsPass"], 1);
    assert_eq!(card["counts"]["tests"], 2);
    assert_eq!(card["counts"]["agents"], 1);
    assert_eq!(card["masterMissing"], true);
    assert_eq!(card["master"], Value::Null);
    assert_eq!(card["recent"][0]["text"], "T-1 pass: Form renders");
    let from_disk = summarize(home.path(), "alpha", &portfolio(), None);
    assert_eq!(from_disk["state"], "running");
}

#[test]
fn the_card_lists_what_is_assigned_to_the_owner_and_a_to_do_for_a_lead_does_not_need_him() {
    let home = home_with_alpha();
    let mut board = alpha();
    board["items"]["I-1"]["assignee"] = json!("owner");
    board["items"]["I-1"]["type"] = json!("bug");
    board["items"]["I-1"]["stage"] = json!("verifying");
    board["items"]["I-2"]["assignee"] = json!("ui");
    board["actions"]["H-2"] = json!({
        "id": "H-2", "thread": "ui", "text": "Ask Tessera", "status": "pending", "assignee": "lead:tessera",
    });
    board["actions"]["H-3"] = json!({
        "id": "H-3", "thread": "ui", "text": "Pay the fee", "status": "pending", "assignee": "owner",
    });
    let card = summarize(home.path(), "alpha", &portfolio(), Some(&board));
    let want = json!([
        {"id": "I-1", "title": "Login screen", "type": "bug", "stage": "verifying"},
        {"id": "I-21", "title": "Login smoke run 1", "type": "test-run", "stage": "ready"},
        {"id": "H-1", "title": "Install the test certificate", "type": "todo", "stage": "pending"},
        {"id": "H-3", "title": "Pay the fee", "type": "todo", "stage": "pending"},
    ]);
    assert_eq!(card["ownerItems"], want);
    assert_eq!(card["counts"]["needsYou"], 4);
}

#[test]
fn a_card_with_nothing_assigned_to_the_owner_has_an_empty_list_and_a_done_item_leaves_it() {
    let home = home_with_alpha();
    let mut board = alpha();
    board["actions"]["H-1"]["status"] = json!("done");
    board["items"]["I-2"]["assignee"] = json!("owner");
    board["items"]["I-2"]["status"] = json!("done");
    board["items"]["I-21"]["status"] = json!("done");
    let card = summarize(home.path(), "alpha", &portfolio(), Some(&board));
    assert_eq!(card["ownerItems"].as_array().map(Vec::len), Some(0));
}

#[test]
fn counts_done_counts_done_items_of_live_threads() {
    let home = home_with_alpha();
    let mut board = alpha();
    board["threads"]["old"] = json!({"id": "old", "status": "archived"});
    let done = |id: &str, thread: &str, stage: &str| json!({"id": id, "thread": thread, "title": id, "status": "todo", "stage": stage});
    board["items"]["I-10"] = done("I-10", "ui", "done");
    board["items"]["I-11"] = done("I-11", "old", "done");
    board["items"]["I-12"] = done("I-12", "gone", "done");
    board["items"]["I-13"] = done("I-13", "ui", "doing");
    let card = summarize(home.path(), "alpha", &portfolio(), Some(&board));
    assert_eq!(card["counts"]["done"], 1);
}

#[test]
fn proofs_are_kept_next_to_the_board_and_numbered_up() {
    let home = tempfile::tempdir().expect("temp dir");
    let mut test = json!({"id": "T-1", "proofs": []});
    let first =
        keep_proof_bytes(home.path(), "alpha", &mut test, b"png", ".PNG", "green").expect("kept");
    let second =
        keep_proof_bytes(home.path(), "alpha", &mut test, b"jpg", ".jpg", "red").expect("kept");
    assert_eq!((first, second), (1, 2));
    assert!(
        home.path()
            .join("alpha")
            .join("proofs")
            .join("T-1-1.png")
            .exists()
    );
    assert_eq!(test["proofs"][1]["caption"], "red");
    let bad = keep_proof_bytes(home.path(), "alpha", &mut test, b"x", ".txt", "no");
    assert!(matches!(bad, Err(BoardError::Refused(m)) if m.starts_with("proof must be an image")));
    let empty = keep_proof_bytes(home.path(), "alpha", &mut test, b"x", ".png", "  ");
    assert!(matches!(empty, Err(BoardError::Refused(m)) if m == "caption text is required."));
}
