//! The owner's ops on a board: pause and resume, move, acceptance criteria, create, bulk, the
//! soft ops, the handed build and purge with its backup.
//! K1, K3, K4: the owner ops and the confirmed purge.

// test-tag: critical

use crate::common::{alpha, exported, post};
use argus_board::ops::purge_plan;
use argus_board::sync::import_entries_at;
use argus_core::Coll;
use serde_json::{Value, json};

fn with_api_thread() -> Value {
    let mut b = alpha();
    let mut api = b["threads"]["ui"].clone();
    api["id"] = json!("api");
    api["session"] = json!("Alpha - API");
    b["threads"]["api"] = api;
    b
}

#[test]
fn pause_stops_the_agents_and_frees_the_locks_and_resume_brings_the_thread_back() {
    let mut b = alpha();
    post(&mut b, json!({"kind": "pause", "replyTo": "ui"}));
    assert_eq!(b["threads"]["ui"]["status"], "paused");
    assert_eq!(b["threads"]["ui"]["pausedBy"], "the owner");
    assert_eq!(b["agents"]["A-1"]["status"], "stopped");
    assert_eq!(b["locks"]["dev-server"]["holder"], Value::Null);
    let inbox = b["inbox"]["ui"].as_array().cloned().unwrap_or_default();
    assert!(inbox.iter().any(|m| m["kind"] == "pause"));
    post(&mut b, json!({"kind": "resume", "replyTo": "ui"}));
    assert_eq!(b["threads"]["ui"]["status"], "active");
    assert!(b["threads"]["ui"].get("pausedBy").is_none());
}

#[test]
fn an_item_takes_a_blocker_and_a_blank_value_clears_it() {
    let mut b = alpha();
    let edit = |b: &mut Value, blocker: &str| {
        let (_, res) = post(
            b,
            json!({"kind": "edit", "target": {"c": "items", "id": "I-1"}, "set": {"blocker": blocker}}),
        );
        assert_eq!(res.ops.applied, 1);
    };
    edit(&mut b, "D-11");
    assert_eq!(b["items"]["I-1"]["blocker"], "D-11");
    edit(&mut b, "  ");
    assert_eq!(b["items"]["I-1"]["blocker"], Value::Null);
}

#[test]
fn moving_an_item_to_another_thread_takes_its_tests_and_open_entries_along() {
    let mut b = with_api_thread();
    let (id, res) = post(
        &mut b,
        json!({"kind": "move", "target": {"c": "items", "id": "I-1"}, "to": {"thread": "api"}}),
    );
    assert_eq!(
        res.ops.applied,
        1,
        "{}",
        b["entries"][id.as_str()]["outcome"]
    );
    assert_eq!(b["items"]["I-1"]["thread"], "api");
    assert_eq!(b["tests"]["T-2"]["thread"], "api");
    assert_eq!(b["entries"]["Q-1"]["thread"], "api");
    assert_eq!(b["items"]["I-1"]["rev"], 2);
    let told = b["inbox"]["ui"].as_array().cloned().unwrap_or_default();
    assert!(
        told.iter()
            .any(|m| m["kind"] == "change" && m["ref"] == "I-1")
    );
}

#[test]
fn adding_and_removing_acceptance_criteria_renumbers_what_the_tests_cover() {
    let mut b = alpha();
    let (_, added) = post(
        &mut b,
        json!({"kind": "ac", "target": {"c": "items", "id": "I-1"}, "do": "add", "text": "has a title", "to": 1}),
    );
    assert_eq!(added.ops.applied, 1);
    assert_eq!(b["items"]["I-1"]["ac"][0]["text"], "has a title");
    assert_eq!(b["items"]["I-1"]["ac"][2]["n"], 3);
    assert_eq!(b["tests"]["T-1"]["ac"], json!([2]));
    assert_eq!(b["tests"]["T-2"]["ac"], json!([3]));
    let (_, removed) = post(
        &mut b,
        json!({"kind": "ac", "target": {"c": "items", "id": "I-1"}, "do": "rm", "n": 2}),
    );
    assert_eq!(removed.ops.applied, 1);
    assert_eq!(b["items"]["I-1"]["ac"].as_array().map(Vec::len), Some(2));
    assert_eq!(b["tests"]["T-1"]["ac"], json!([]));
    assert_eq!(b["tests"]["T-2"]["ac"], json!([2]));
}

#[test]
fn the_owner_creates_an_item_with_criteria_for_a_thread() {
    let mut b = alpha();
    let (_, made) = post(
        &mut b,
        json!({"kind": "create", "target": {"c": "items"}, "set": {"title": "Home", "thread": "ui", "type": "epic"}}),
    );
    assert_eq!(made.ops.applied, 1);
    let (id, res) = post(
        &mut b,
        json!({"kind": "create", "target": {"c": "items"}, "set": {"title": "New thing", "thread": "ui", "parent": "I-3", "ac": ["one", "two"]}}),
    );
    assert_eq!(
        res.ops.applied,
        1,
        "{}",
        b["entries"][id.as_str()]["outcome"]
    );
    assert_eq!(b["entries"][id.as_str()]["outcome"], "I-4 created");
    let item = &b["items"]["I-4"];
    assert_eq!(item["origin"], "coordinator");
    assert_eq!(item["subject"], Value::Null);
    assert_eq!(item["ac"].as_array().map(Vec::len), Some(2));
    assert_eq!(item["createdBy"], "human");
    assert_eq!(b["counters"]["I"], 4);
}

#[test]
fn a_bulk_judges_each_op_on_its_own() {
    let mut b = alpha();
    let (id, res) = post(
        &mut b,
        json!({"kind": "bulk", "ops": [
            {"kind": "edit", "target": {"c": "items", "id": "I-2"}, "set": {"title": "Bulk title"}},
            {"kind": "edit", "target": {"c": "items", "id": "I-2"}, "set": {"colour": "red"}},
            {"kind": "bulk", "ops": []},
        ]}),
    );
    assert_eq!(res.ops.applied, 1);
    let entry = &b["entries"][id.as_str()];
    assert_eq!(entry["outcome"], "1 of 3 applied, 2 refused");
    assert_eq!(
        entry["results"][1]["outcome"],
        "colour cannot be edited on items."
    );
    assert_eq!(
        entry["results"][2]["outcome"],
        "a bulk cannot be inside a bulk."
    );
    assert_eq!(b["items"]["I-2"]["title"], "Bulk title");
}

#[test]
fn delete_restore_and_archive_are_soft_and_stop_a_running_agent() {
    let mut b = alpha();
    post(
        &mut b,
        json!({"kind": "delete", "target": {"c": "items", "id": "I-2"}, "reason": "dup"}),
    );
    assert_eq!(b["items"]["I-2"]["deletedBy"], "human");
    assert_eq!(b["items"]["I-2"]["deleteReason"], "dup");
    let (_, again) = post(
        &mut b,
        json!({"kind": "edit", "target": {"c": "items", "id": "I-2"}, "set": {"title": "x"}}),
    );
    assert_eq!(again.ops.refused, 1);
    post(
        &mut b,
        json!({"kind": "restore", "target": {"c": "items", "id": "I-2"}}),
    );
    assert!(b["items"]["I-2"].get("deletedAt").is_none());
    post(
        &mut b,
        json!({"kind": "archive", "target": {"c": "agents", "id": "A-1"}}),
    );
    assert_eq!(b["agents"]["A-1"]["status"], "stopped");
    assert!(b["agents"]["A-1"]["archivedAt"].is_string());
    let (_, coord) = post(
        &mut b,
        json!({"kind": "archive", "target": {"c": "threads", "id": "coordinator"}}),
    );
    assert_eq!(coord.ops.refused, 1);
}

#[test]
fn handing_a_build_records_it_and_unhanding_clears_it() {
    let mut b = alpha();
    let (_, same) = post(&mut b, json!({"kind": "build", "handed": "b1"}));
    assert_eq!(same.ops.applied, 1);
    post(&mut b, json!({"kind": "build", "handed": "b2"}));
    assert_eq!(b["handedBuild"], "b2");
    assert_eq!(b["builds"]["b2"]["status"], "ready");
    post(&mut b, json!({"kind": "build", "handed": null}));
    assert_eq!(b["handedBuild"], Value::Null);
    assert_eq!(b["handedAt"], Value::Null);
}

#[test]
fn the_lead_avatar_is_a_brand_or_a_small_image_and_the_export_carries_it() {
    let mut b = alpha();
    let (_, brand) = post(&mut b, json!({"kind": "avatar", "avatar": "brand:tessera"}));
    assert_eq!(brand.ops.applied, 1);
    assert_eq!(b["avatar"], "brand:tessera");
    assert_eq!(exported(&b, Coll::Meta, "board")["avatar"], "brand:tessera");
    let image = "data:image/png;base64,iVBORw0KGgo=";
    post(&mut b, json!({"kind": "avatar", "avatar": image}));
    assert_eq!(b["avatar"], image);
    for bad in [
        json!("brand:nope"),
        json!("data:image/svg+xml;base64,PHN2Zz4="),
        json!("data:image/png;base64,not base64!"),
        json!(format!("data:image/png;base64,{}", "A".repeat(250_000))),
        json!(7),
    ] {
        let (_, res) = post(&mut b, json!({"kind": "avatar", "avatar": bad}));
        assert_eq!(res.ops.refused, 1);
    }
    assert_eq!(b["avatar"], image);
    post(&mut b, json!({"kind": "avatar", "avatar": null}));
    assert_eq!(b["avatar"], Value::Null);
}

#[test]
fn a_confirmed_purge_writes_a_backup_then_removes_the_thread_and_what_it_owns() {
    let home = tempfile::tempdir().expect("temp dir");
    let mut b = with_api_thread();
    let plan = purge_plan(&b, &json!({"threads": ["ui"]})).expect("plan");
    assert_eq!(plan.counts["items"], 4);
    let entry = json!({
        "id": "O-purge", "kind": "purge", "author": "human", "status": "new", "createdAt": argus_core::clock::now_iso(),
        "criteria": {"threads": ["ui"]}, "confirm": "alpha", "expect": plan.counts,
    });
    let res = import_entries_at(home.path(), &mut b, vec![entry], &|_| false);
    assert_eq!(res.ops.applied, 1, "{}", b["entries"]["O-purge"]["outcome"]);
    assert_eq!(res.notes.len(), 1);
    assert!(b["threads"].get("ui").is_none());
    assert_eq!(b["items"], json!({}));
    assert!(b["threads"].get("api").is_some());
    let purged = home.path().join("alpha").join("purged");
    let backups: Vec<_> = std::fs::read_dir(&purged)
        .expect("backup folder")
        .filter_map(Result::ok)
        .collect();
    assert_eq!(backups.len(), 1);
    let text = std::fs::read_to_string(backups[0].path()).expect("backup");
    let backup: Value = serde_json::from_str(&text).expect("json");
    assert_eq!(backup["project"], "alpha");
    assert_eq!(backup["records"]["items"]["I-1"]["title"], "Login screen");
}

#[test]
fn a_purge_with_the_wrong_confirm_text_is_refused_and_removes_nothing() {
    let home = tempfile::tempdir().expect("temp dir");
    let mut b = alpha();
    let entry = json!({
        "id": "O-bad", "kind": "purge", "author": "human", "status": "new", "createdAt": argus_core::clock::now_iso(),
        "criteria": {"threads": ["ui"]}, "confirm": "beta", "expect": {"threads": 1},
    });
    let res = import_entries_at(home.path(), &mut b, vec![entry], &|_| false);
    assert_eq!(res.ops.refused, 1);
    assert_eq!(
        b["entries"]["O-bad"]["outcome"],
        "the confirm text must equal the project key"
    );
    assert!(b["threads"].get("ui").is_some());
}
