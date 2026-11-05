//! The board items of the portfolio: the reconcile that gives every project one, and the owner's
//! edit, archive and unarchive of them.

// test-tag: critical

use crate::common::{home_with_alpha, portfolio_of};
use argus_board::portfolio::{import_portfolio_entries, needs_reconcile, reconcile};
use serde_json::{Value, json};
use std::path::Path;

fn two_projects() -> Value {
    portfolio_of(json!({
        "alpha": {"key": "alpha", "name": "Alpha", "goal": "Ship alpha"},
        "beta": {"key": "beta", "name": "Beta"},
    }))
}

fn reconciled() -> (tempfile::TempDir, Value) {
    let home = home_with_alpha();
    let mut p = two_projects();
    reconcile(home.path(), &mut p);
    (home, p)
}

fn owner_op(id: &str, kind: &str, author: &str, rest: &Value) -> Value {
    let mut op = json!({
        "id": id, "kind": kind, "author": author, "target": {"c": "boards", "id": "BD-1"},
        "createdAt": argus_core::clock::now_iso(),
    });
    for (field, value) in rest.as_object().cloned().unwrap_or_default() {
        op[field.as_str()] = value;
    }
    op
}

fn send(home: &Path, p: &mut Value, op: Value) -> String {
    let out = import_portfolio_entries(home, p, vec![op]).expect("imported");
    out.results[0].1.clone()
}

#[test]
fn reconcile_gives_each_project_one_board_item() {
    let (_home, p) = reconciled();
    let boards = p["boards"].as_object().expect("boards");
    assert_eq!(boards.len(), 2);
    assert_eq!(p["boards"]["BD-1"]["key"], "alpha");
    assert_eq!(p["boards"]["BD-1"]["goal"], "Ship alpha");
    assert_eq!(p["boards"]["BD-2"]["key"], "beta");
    assert_eq!(p["boards"]["BD-2"]["note"], "");
    assert_eq!(p["counters"]["BD"], 2);
    assert!(p["boards"]["BD-1"]["createdAt"].is_string());
    assert!(p["boards"]["BD-1"]["archivedAt"].is_null());
}

#[test]
fn reconcile_twice_changes_nothing() {
    let (home, mut p) = reconciled();
    assert!(!needs_reconcile(home.path(), &p));
    let before = p.clone();
    assert!(!reconcile(home.path(), &mut p));
    assert_eq!(p, before);
}

#[test]
fn a_board_folder_without_a_registry_row_gets_an_item() {
    let home = home_with_alpha();
    let mut p = portfolio_of(json!({}));
    assert!(needs_reconcile(home.path(), &p));
    assert!(reconcile(home.path(), &mut p));
    assert_eq!(p["boards"]["BD-1"]["key"], "alpha");
}

#[test]
fn a_retired_project_archives_its_item_and_keeps_its_id() {
    let (home, mut p) = reconciled();
    p["projects"]["alpha"]["retired"] = json!(true);
    assert!(reconcile(home.path(), &mut p));
    assert!(p["boards"]["BD-1"]["archivedAt"].is_string());
    assert_eq!(p["boards"]["BD-1"]["archivedBy"], "portfolio");
    assert_eq!(p["boards"]["BD-1"]["key"], "alpha");
    assert!(p["boards"]["BD-2"]["archivedAt"].is_null());
    assert_eq!(p["boards"].as_object().expect("boards").len(), 2);
}

#[test]
fn a_project_added_back_unarchives_the_same_item() {
    let (home, mut p) = reconciled();
    p["projects"]["alpha"]["retired"] = json!(true);
    reconcile(home.path(), &mut p);
    p["projects"]["alpha"]["retired"] = json!(false);
    assert!(reconcile(home.path(), &mut p));
    assert!(p["boards"]["BD-1"]["archivedAt"].is_null());
    assert!(p["boards"]["BD-1"]["archivedBy"].is_null());
    assert_eq!(p["boards"]["BD-1"]["key"], "alpha");
    assert_eq!(p["counters"]["BD"], 2);
}

#[test]
fn an_owner_edit_changes_goal_and_note_only() {
    let (home, mut p) = reconciled();
    let created = p["boards"]["BD-1"]["createdAt"].clone();
    let set = json!({"goal": "New goal", "note": "A note"});
    let status = send(
        home.path(),
        &mut p,
        owner_op("O-1", "edit", "human", &json!({"set": set})),
    );
    assert_eq!(status, "applied");
    let item = &p["boards"]["BD-1"];
    assert_eq!(item["goal"], "New goal");
    assert_eq!(item["note"], "A note");
    assert_eq!(item["key"], "alpha");
    assert_eq!(item["createdAt"], created);
    assert_eq!(item["updatedBy"], "human");
    assert_eq!(item["rev"], 2);
}

#[test]
fn an_edit_of_key_or_created_is_refused() {
    let (home, mut p) = reconciled();
    let before = p["boards"]["BD-1"].clone();
    for (n, set) in [json!({"key": "gamma"}), json!({"createdAt": "2020"})]
        .into_iter()
        .enumerate()
    {
        let id = format!("O-{n}");
        let status = send(
            home.path(),
            &mut p,
            owner_op(&id, "edit", "human", &json!({"set": set})),
        );
        assert_eq!(status, "refused");
    }
    assert_eq!(p["boards"]["BD-1"], before);
}

#[test]
fn a_thread_author_is_refused() {
    let (home, mut p) = reconciled();
    let before = p["boards"]["BD-1"].clone();
    let set = json!({"goal": "Mine now"});
    let status = send(
        home.path(),
        &mut p,
        owner_op("O-1", "edit", "ui", &json!({"set": set})),
    );
    assert_eq!(status, "refused");
    assert_eq!(p["boards"]["BD-1"], before);
}

#[test]
fn archive_retires_the_project_in_the_same_write() {
    let (home, mut p) = reconciled();
    let status = send(
        home.path(),
        &mut p,
        owner_op("O-1", "archive", "human", &json!({})),
    );
    assert_eq!(status, "applied");
    assert_eq!(p["projects"]["alpha"]["retired"], true);
    assert!(p["boards"]["BD-1"]["archivedAt"].is_string());
    assert_eq!(p["boards"]["BD-1"]["archivedBy"], "human");
    let status = send(
        home.path(),
        &mut p,
        owner_op("O-2", "unarchive", "human", &json!({})),
    );
    assert_eq!(status, "applied");
    assert!(p["projects"]["alpha"]["retired"].is_null());
    assert!(p["boards"]["BD-1"]["archivedAt"].is_null());
    assert!(!needs_reconcile(home.path(), &p));
}

#[test]
fn delete_and_purge_of_a_board_item_are_refused() {
    let (home, mut p) = reconciled();
    for (n, kind) in ["delete", "purge"].into_iter().enumerate() {
        let id = format!("O-{n}");
        let status = send(
            home.path(),
            &mut p,
            owner_op(&id, kind, "human", &json!({})),
        );
        assert_eq!(status, "refused");
    }
    assert_eq!(p["boards"].as_object().expect("boards").len(), 2);
    assert!(p["projects"]["alpha"]["retired"].is_null());
}
