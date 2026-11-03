//! The owner ops that link and keep worktrees and link and retire builds: `worktree-link`,
//! `worktree-keep`, `launcher-link`, `launcher-retire`. I-85.

// test-tag: critical

use crate::common::{alpha, applied, post, refused};
use argus_board::ops::{OpResult, Who, apply_op};
use serde_json::{Value, json};

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
    let res = apply_op(b, &op, &Who::human(), None);
    applied(&res);
    res.extra["created"]["id"]
        .as_str()
        .unwrap_or_default()
        .to_owned()
}

fn on(b: &mut Value, kind: &str, (coll, id): (&str, &str), set: &Value) -> OpResult {
    let op = json!({"kind": kind, "target": {"c": coll, "id": id}, "set": set});
    apply_op(b, &op, &Who::human(), None)
}

fn worktree(b: &mut Value, kind: &str, id: &str, set: &Value) -> OpResult {
    on(b, kind, ("worktrees", id), set)
}

fn build(b: &mut Value, kind: &str, name: &str, set: &Value) -> OpResult {
    on(b, kind, ("launchers", name), set)
}

#[test]
fn a_worktree_is_linked_to_an_open_item_and_unlinked_with_a_null_item() {
    let mut b = alpha();
    let item = story(&mut b);
    applied(&worktree(
        &mut b,
        "worktree-link",
        "ab12cd34ef",
        &json!({"item": item}),
    ));
    let link = &b["worktreeLinks"]["ab12cd34ef"];
    assert_eq!(link["item"], json!(item));
    assert_eq!(link["by"], "human");
    assert!(link["at"].is_string());
    let unlink = json!({"item": null});
    applied(&worktree(&mut b, "worktree-link", "ab12cd34ef", &unlink));
    assert_eq!(b["worktreeLinks"].get("ab12cd34ef"), None);
    let again = worktree(&mut b, "worktree-link", "ab12cd34ef", &unlink);
    assert!(refused(&again).contains("no link"));
}

#[test]
fn a_worktree_link_refuses_a_missing_item_a_closed_item_and_a_bad_id() {
    let mut b = alpha();
    let item = story(&mut b);
    let gone = worktree(&mut b, "worktree-link", "ab12", &json!({"item": "I-999"}));
    assert!(refused(&gone).contains("no item"));
    let done =
        json!({"kind": "move", "target": {"c": "items", "id": item}, "to": {"status": "done"}});
    applied(&apply_op(&mut b, &done, &Who::human(), None));
    let closed = worktree(&mut b, "worktree-link", "ab12", &json!({"item": item}));
    assert!(refused(&closed).contains("open item"));
    let bad = worktree(&mut b, "worktree-link", "../x", &json!({"item": item}));
    assert!(refused(&bad).contains("id of a worktree"));
    assert_eq!(b["worktreeLinks"].get("ab12"), None);
}

#[test]
fn a_worktree_is_kept_with_a_note_for_seven_days_or_the_days_asked() {
    let mut b = alpha();
    let first = json!({"note": "review pending"});
    applied(&worktree(&mut b, "worktree-keep", "ab12", &first));
    let keep = &b["worktreeKeeps"]["ab12"];
    assert_eq!(keep["note"], "review pending");
    assert_eq!(keep["by"], "human");
    assert!(keep["until"].is_string());
    let longer = json!({"note": "again", "days": 30});
    applied(&worktree(&mut b, "worktree-keep", "ab12", &longer));
    assert_eq!(b["worktreeKeeps"]["ab12"]["note"], "again");
    applied(&worktree(
        &mut b,
        "worktree-keep",
        "ab12",
        &json!({"clear": true}),
    ));
    assert_eq!(b["worktreeKeeps"].get("ab12"), None);
}

#[test]
fn a_worktree_keep_refuses_no_note_a_long_note_and_days_out_of_range() {
    let mut b = alpha();
    refused(&worktree(&mut b, "worktree-keep", "ab12", &json!({})));
    let long = json!({"note": "x".repeat(301)});
    refused(&worktree(&mut b, "worktree-keep", "ab12", &long));
    for days in [json!(0), json!(366), json!("7"), json!(1.5)] {
        let set = json!({"note": "n", "days": days});
        let res = worktree(&mut b, "worktree-keep", "ab12", &set);
        assert!(refused(&res).contains("days"));
    }
    let clear = json!({"clear": true});
    let nothing = worktree(&mut b, "worktree-keep", "ab12", &clear);
    assert!(refused(&nothing).contains("not kept"));
    assert_eq!(b["worktreeKeeps"].get("ab12"), None);
}

#[test]
fn a_build_is_linked_to_the_open_item_it_serves() {
    let mut b = alpha();
    let item = story(&mut b);
    b["launchers"]["web"] = launcher("web", None);
    let set = json!({"for": item});
    applied(&build(&mut b, "launcher-link", "web", &set));
    assert_eq!(b["launchers"]["web"]["for"], json!(item));
    assert_eq!(b["launchers"]["web"]["by"], "human");
    let same = build(&mut b, "launcher-link", "web", &set);
    applied(&same);
    assert!(same.outcome.contains("already serves"));
}

#[test]
fn a_build_link_refuses_no_build_no_item_and_a_closed_item() {
    let mut b = alpha();
    let item = story(&mut b);
    refused(&build(
        &mut b,
        "launcher-link",
        "nope",
        &json!({"for": item}),
    ));
    b["launchers"]["web"] = launcher("web", None);
    refused(&build(&mut b, "launcher-link", "web", &json!({})));
    refused(&build(
        &mut b,
        "launcher-link",
        "web",
        &json!({"for": "I-999"}),
    ));
    refused(&build(
        &mut b,
        "launcher-link",
        "web",
        &json!({"for": "T-1"}),
    ));
    assert_eq!(b["launchers"]["web"]["for"], Value::Null);
}

#[test]
fn a_build_is_retired_with_its_note_and_play_but_never_the_last_master() {
    let mut b = alpha();
    b["launchers"]["main"] = launcher("main", None);
    b["launchers"]["main"]["master"] = json!(true);
    b["launchers"]["web"] = launcher("web", None);
    b["runDefs"] = json!({"launcher-web": {"name": "launcher-web", "launcher": "web"}});
    let master = build(&mut b, "launcher-retire", "main", &json!({}));
    assert!(refused(&master).contains("master build"));
    assert_eq!(b["launchers"]["main"]["retiredAt"], Value::Null);
    let note = json!({"note": "obsolete"});
    applied(&build(&mut b, "launcher-retire", "web", &note));
    let web = &b["launchers"]["web"];
    assert!(web["retiredAt"].is_string());
    assert_eq!(web["retiredBy"], "human");
    assert_eq!(web["retireNote"], "obsolete");
    assert_eq!(web["current"], false);
    assert_eq!(b["runDefs"].get("launcher-web"), None);
    let again = build(&mut b, "launcher-retire", "web", &json!({}));
    assert!(again.outcome.contains("already retired"));
}

#[test]
fn the_new_op_kinds_travel_as_owner_entries() {
    let mut b = alpha();
    let item = story(&mut b);
    let (_, res) = post(
        &mut b,
        json!({"kind": "worktree-link", "target": {"c": "worktrees", "id": "ab12"}, "set": {"item": item}}),
    );
    assert_eq!(res.ops.applied, 1);
    assert_eq!(b["worktreeLinks"]["ab12"]["item"], json!(item));
    let (_, bad) = post(
        &mut b,
        json!({"kind": "launcher-retire", "target": {"c": "launchers", "id": "nope"}, "set": {}}),
    );
    assert_eq!(bad.ops.refused, 1);
}
