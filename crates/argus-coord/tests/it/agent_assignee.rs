//! I-206 P3b: `coord agent start --item` makes the agent the assignee of its items, the end of the
//! agent hands them back to its thread, a second agent on a held item is refused unless `--take`,
//! and `coord item migrate-assignees` brings old boards in line.

// test-tag: critical

use crate::agent_common::{As, Rig, hook_agent};
use serde_json::{Value, json};

const WHY: &str = "known cause, one module";

fn rig() -> Rig {
    let rig = Rig::new();
    rig.with_item();
    rig.with_item_status("I-2", "doing");
    rig
}

fn start(name: &str, extra: &[&str]) -> Vec<String> {
    let mut args = vec!["agent", "start", name, "do the work", "--model", "sonnet"];
    args.extend(["--why", WHY]);
    args.extend(extra);
    args.into_iter().map(str::to_owned).collect()
}

fn run(rig: &Rig, args: &[String], who: As) -> crate::common::Ran {
    let line: Vec<&str> = args.iter().map(String::as_str).collect();
    rig.call(&line, who)
}

fn assignee(rig: &Rig, id: &str) -> Value {
    rig.board()["items"][id]["assignee"].clone()
}

#[test]
fn a_start_writes_the_agent_as_assignee_of_each_item() {
    let rig = rig();
    let ran = run(
        &rig,
        &start("a1", &["--item", "I-1,I-2", "--master"]),
        As::Lead,
    );
    assert_eq!(ran.code, 0, "{}", ran.err);
    assert_eq!(assignee(&rig, "I-1"), "agent:A-1");
    assert_eq!(assignee(&rig, "I-2"), "agent:A-1");
}

#[test]
fn the_end_of_the_agent_returns_the_items_to_its_thread() {
    let rig = rig();
    run(&rig, &start("a1", &["--item", "I-1", "--master"]), As::Lead);
    rig.ok(&["agent", "done", "A-1"], As::Lead);
    assert_eq!(assignee(&rig, "I-1"), "coordinator");
    run(&rig, &start("a2", &["--item", "I-2", "--master"]), As::Lead);
    rig.ok(&["agent", "stop", "A-2"], As::Lead);
    assert_eq!(assignee(&rig, "I-2"), "coordinator");
}

#[test]
fn a_failed_or_deleted_agent_returns_its_items_too() {
    let rig = rig();
    run(&rig, &start("a1", &["--item", "I-1", "--master"]), As::Lead);
    rig.ok(&["agent", "fail", "A-1", "it broke"], As::Lead);
    assert_eq!(assignee(&rig, "I-1"), "coordinator");
    run(&rig, &start("a2", &["--item", "I-2", "--master"]), As::Lead);
    rig.ok(&["agent", "rm", "A-2"], As::Lead);
    assert_eq!(assignee(&rig, "I-2"), "coordinator");
}

#[test]
fn an_item_taken_over_by_someone_else_stays_with_them_when_the_agent_ends() {
    let rig = rig();
    run(&rig, &start("a1", &["--item", "I-1", "--master"]), As::Lead);
    rig.ok(&["item", "set", "I-1", "--assignee", "worker"], As::Lead);
    rig.ok(&["agent", "done", "A-1"], As::Lead);
    assert_eq!(assignee(&rig, "I-1"), "worker");
}

#[test]
fn a_second_agent_on_a_held_item_is_refused_unless_it_takes_the_item() {
    let rig = rig();
    run(&rig, &start("a1", &["--item", "I-1", "--master"]), As::Lead);
    let refused = run(&rig, &start("a2", &["--item", "I-1", "--master"]), As::Lead);
    assert_ne!(refused.code, 0);
    assert!(
        refused.err.contains("stop A-1 or pass --take"),
        "{}",
        refused.err
    );
    assert_eq!(assignee(&rig, "I-1"), "agent:A-1");
    let taken = run(
        &rig,
        &start("a2", &["--item", "I-1", "--master", "--take"]),
        As::Lead,
    );
    assert_eq!(taken.code, 0, "{}", taken.err);
    assert_eq!(assignee(&rig, "I-1"), "agent:A-2");
}

#[test]
fn migrate_assignees_sets_the_live_agent_and_a_dry_run_writes_nothing() {
    let rig = rig();
    rig.edit(|b| {
        let mut agent = hook_agent("old", "sonnet");
        agent["id"] = json!("A-1");
        agent["items"] = json!(["I-1"]);
        b["agents"]["A-1"] = agent;
    });
    let before = rig.board();
    let dry = rig.ok(&["item", "migrate-assignees", "--dry-run"], As::Lead);
    assert!(
        dry.out.contains("dry run: 1 assignee(s) would migrate"),
        "{}",
        dry.out
    );
    assert_eq!(rig.board(), before);
    let ran = rig.ok(&["item", "migrate-assignees"], As::Lead);
    assert!(
        ran.out.contains("migrated 1 assignee(s), kept 0."),
        "{}",
        ran.out
    );
    assert_eq!(assignee(&rig, "I-1"), "agent:A-1");
    assert_eq!(assignee(&rig, "I-2"), Value::Null);
    let again = rig.ok(&["item", "migrate-assignees"], As::Lead);
    assert_eq!(again.out, "no item needs an agent assignee.\n");
}

#[test]
fn a_takeover_gives_the_hold_of_the_twin_to_the_absorbing_agent() {
    let rig = rig();
    rig.edit(|b| {
        let mut target = hook_agent("lead-started", "sonnet");
        target["id"] = json!("A-1");
        target["source"] = json!("manual");
        target["hookId"] = Value::Null;
        target["items"] = json!(["I-1"]);
        let mut twin = hook_agent("hook-made", "sonnet");
        twin["id"] = json!("A-2");
        twin["claudeId"] = json!("dupe-1");
        twin["hookId"] = json!("dupe-1");
        twin["items"] = json!(["I-1"]);
        b["agents"]["A-1"] = target;
        b["agents"]["A-2"] = twin;
        b["items"]["I-1"]["assignee"] = json!("agent:A-2");
    });
    rig.ok(&["agent", "set", "A-1", "--claude-id", "dupe-1"], As::Lead);
    assert_eq!(assignee(&rig, "I-1"), "agent:A-1");
    assert_eq!(rig.board()["agents"]["A-2"]["status"], "stopped");
}
