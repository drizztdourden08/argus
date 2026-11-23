//! The per-board Kanban config: the default is today's four columns, the `kanban-config` op
//! checks and stores a config, the export carries it, and a lane change stamps `laneAt`.

// test-tag: critical

use crate::common::{alpha, applied, exported, lead, post, refused};
use argus_board::kanban_config::{default_config, effective, parse};
use argus_board::ops::{Role, Who, apply_op};
use argus_core::Coll;
use serde_json::{Value, json};

const HOUR_MS: i64 = 3_600_000;

fn good() -> Value {
    json!({
        "columns": [
            {"id": "now", "name": "Now", "match": {"lanes": ["doing", "review"]},
             "limit": {"max": 3, "mode": "warn"}, "rules": ["epic"]},
            {"id": "later", "name": "Later", "match": {"lanes": ["todo"]}}
        ],
        "rules": [
            {"id": "epic", "kind": "require-parent", "params": {"types": ["epic"]}, "mode": "refuse"}
        ]
    })
}

fn with(edit: impl FnOnce(&mut Value)) -> Value {
    let mut config = good();
    edit(&mut config);
    config
}

fn set_config(config: &Value) -> String {
    let op = json!({"kind": "kanban-config", "kanban": config});
    refused(&apply_op(&mut alpha(), &op, &lead(), None))
}

#[test]
fn the_default_is_the_four_columns_and_the_quiet_hours_of_before() {
    let config = default_config();
    let columns: Vec<(&str, Vec<&str>)> = config
        .columns
        .iter()
        .map(|c| {
            let lanes = c.matches.lanes.iter().map(String::as_str).collect();
            (c.id.as_str(), lanes)
        })
        .collect();
    assert_eq!(
        columns,
        vec![
            ("todo", vec!["todo"]),
            ("doing", vec!["doing"]),
            ("review", vec!["review"]),
            ("done", vec!["done", "dropped"]),
        ]
    );
    let dwell = |id: &str| {
        let rule = config.rules.iter().find(|r| r.id == id);
        rule.map(|r| r.params["ms"].as_i64().unwrap_or(0))
    };
    assert_eq!(dwell("stale-todo"), Some(24 * HOUR_MS));
    assert_eq!(dwell("stale-doing"), Some(4 * HOUR_MS));
    assert_eq!(dwell("stale-review"), Some(6 * HOUR_MS));
    assert_eq!(
        parse(&serde_json::to_value(&config).unwrap()).unwrap(),
        config
    );
    assert_eq!(effective(&alpha()), config);
}

#[test]
fn a_good_config_is_stored_and_an_equal_one_changes_nothing() {
    let mut b = alpha();
    let (_, res) = post(&mut b, json!({"kind": "kanban-config", "kanban": good()}));
    assert_eq!(res.ops.applied, 1);
    assert_eq!(b["kanban"]["columns"][0]["id"], "now");
    assert_eq!(effective(&b).columns.len(), 2);
    let (_, again) = post(&mut b, json!({"kind": "kanban-config", "kanban": good()}));
    assert_eq!(again.ops.applied, 0);
    assert_eq!(b["kanban"]["columns"][0]["id"], "now");
    let (_, cleared) = post(&mut b, json!({"kind": "kanban-config", "kanban": null}));
    assert_eq!(cleared.ops.applied, 1);
    assert_eq!(b.get("kanban"), None);
}

#[test]
fn a_bad_config_is_refused_with_a_stable_key() {
    let cases = [
        (
            "unknown-lane",
            with(|c| c["columns"][0]["match"]["lanes"] = json!(["nope"])),
        ),
        (
            "duplicate-column",
            with(|c| c["columns"][1]["id"] = json!("now")),
        ),
        (
            "unknown-rule",
            with(|c| c["columns"][0]["rules"] = json!(["ghost"])),
        ),
        (
            "bad-limit",
            with(|c| c["columns"][0]["limit"]["max"] = json!(0)),
        ),
        (
            "bad-limit",
            with(|c| c["columns"][0]["limit"]["mode"] = json!("loud")),
        ),
        (
            "unknown-rule-kind",
            with(|c| c["rules"][0]["kind"] = json!("magic")),
        ),
        ("bad-params", with(|c| c["rules"][0]["params"] = json!({}))),
        (
            "bad-rule-mode",
            with(|c| c["rules"][0]["mode"] = json!("shout")),
        ),
        ("no-columns", with(|c| c["columns"] = json!([]))),
        ("bad-shape", json!("columns")),
    ];
    for (key, config) in cases {
        let said = set_config(&config);
        assert!(said.starts_with(&format!("{key}:")), "{key}: {said}");
    }
}

#[test]
fn only_the_owner_and_the_lead_may_set_it() {
    let mut b = alpha();
    let op = json!({"kind": "kanban-config", "kanban": good()});
    let thread = Who {
        thread: Some("ui".to_owned()),
        role: Role::Thread,
        actor: None,
    };
    refused(&apply_op(&mut b, &op, &thread, None));
    assert_eq!(b.get("kanban"), None);
}

#[test]
fn the_export_carries_the_default_and_a_set_config() {
    let mut b = alpha();
    let meta = exported(&b, Coll::Meta, "kanban");
    assert_eq!(meta["columns"].as_array().map(Vec::len), Some(4));
    post(&mut b, json!({"kind": "kanban-config", "kanban": good()}));
    let meta = exported(&b, Coll::Meta, "kanban");
    assert_eq!(meta["columns"][0]["id"], "now");
}

#[test]
fn a_lane_change_stamps_lane_at() {
    let mut b = alpha();
    let to = if b["items"]["I-1"]["status"] == "review" {
        "doing"
    } else {
        "review"
    };
    b["items"]["I-1"]["laneAt"] = json!("old");
    let target = json!({"c": "items", "id": "I-1"});
    let op = json!({"kind": "move", "target": target, "to": {"status": to}});
    applied(&apply_op(&mut b, &op, &lead(), None));
    assert_eq!(b["items"]["I-1"]["status"], to);
    assert_ne!(b["items"]["I-1"]["laneAt"], "old");
}
