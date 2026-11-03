//! Builds served by an item on a whole board: the owner's post that closes an item closes its
//! builds, and the page export carries the link, the computed stage and the counts.
//! I-78 P10.

// test-tag: critical

use crate::common::{alpha, applied, lead, post};
use argus_board::ops::{Who, apply_op};
use argus_board::sync::export_docs;
use argus_core::Coll;
use serde_json::{Value, json};

fn exported(board: &Value, coll: Coll, id: &str) -> Value {
    export_docs(board, &|_| String::new())
        .into_iter()
        .find(|d| d.coll == coll && d.id == id)
        .and_then(|d| d.data)
        .unwrap_or(Value::Null)
}

fn launcher(name: &str, serves: Option<&str>) -> Value {
    json!({
        "name": name, "title": name, "cmd": "node run.js", "cwd": ".", "kind": "app",
        "current": true, "retiredAt": null, "retiredBy": null, "retireNote": null,
        "createdAt": "2026-01-01T00:00:00.000Z", "updatedAt": "2026-01-01T00:00:00.000Z",
        "for": serves,
    })
}

fn story(b: &mut Value) -> String {
    let op = json!({"kind": "create", "target": {"c": "items"},
        "set": {"title": "Lobby", "thread": "ui", "type": "story"}});
    let res = apply_op(b, &op, &lead(), None);
    applied(&res);
    res.extra["created"]["id"]
        .as_str()
        .unwrap_or_default()
        .to_owned()
}

#[test]
fn an_owner_post_that_moves_a_story_to_done_closes_its_builds_in_the_same_write() {
    let mut b = alpha();
    let id = story(&mut b);
    b["launchers"]["web"] = launcher("web", Some(&id));
    let (_, res) = post(
        &mut b,
        json!({"kind": "move", "target": {"c": "items", "id": id}, "to": {"status": "done"}}),
    );
    assert_eq!(res.ops.applied, 1);
    assert_eq!(b["launchers"]["web"]["retiredByItem"], json!(id));
    assert!(b["launchers"]["web"]["retiredAt"].is_string());
    let (_, back) = post(
        &mut b,
        json!({"kind": "move", "target": {"c": "items", "id": id}, "to": {"status": "doing"}}),
    );
    assert_eq!(back.ops.applied, 1);
    assert_eq!(b["launchers"]["web"]["retiredAt"], Value::Null);
}

#[test]
fn the_export_carries_the_link_the_computed_stage_and_the_counts() {
    let mut b = alpha();
    let id = story(&mut b);
    b["launchers"]["web"] = launcher("web", Some(&id));
    b["launchers"]["tool"] = launcher("tool", None);
    b["launchers"]["tool"]["cmd"] = json!("line one\nline two");
    b["worktreeLinks"] =
        json!({"wt-1": {"item": id, "by": "ui", "at": "2026-01-01T00:00:00.000Z"}});
    let web = exported(&b, Coll::Launchers, "web");
    assert_eq!(web["for"], json!(id));
    assert_eq!(web["stage"], "ready");
    assert_eq!(web["standalone"], false);
    assert_eq!(web["master"], false);
    assert_eq!(exported(&b, Coll::Launchers, "tool")["stage"], "draft");
    let item = exported(&b, Coll::Items, &id);
    assert_eq!(
        item["servedBy"],
        json!({"builds": 1, "closed": 0, "worktrees": 1})
    );
    assert_eq!(exported(&b, Coll::Items, "I-1").get("servedBy"), None);
}

#[test]
fn a_closed_build_exports_as_closed_with_the_item_that_closed_it() {
    let mut b = alpha();
    let id = story(&mut b);
    b["launchers"]["web"] = launcher("web", Some(&id));
    let done =
        json!({"kind": "move", "target": {"c": "items", "id": id}, "to": {"status": "done"}});
    applied(&apply_op(&mut b, &done, &Who::human(), None));
    let web = exported(&b, Coll::Launchers, "web");
    assert_eq!(web["stage"], "closed");
    assert_eq!(web["retiredByItem"], json!(id));
    let item = exported(&b, Coll::Items, &id);
    assert_eq!(
        item["servedBy"],
        json!({"builds": 0, "closed": 1, "worktrees": 0})
    );
}

#[test]
fn the_master_build_is_exported_as_the_master_and_never_standalone() {
    let mut b = alpha();
    b["launchers"]["main"] = launcher("main", None);
    b["launchers"]["main"]["master"] = json!(true);
    let main = exported(&b, Coll::Launchers, "main");
    assert_eq!(main["master"], true);
    assert_eq!(main["standalone"], false);
}
