//! Ports `agent.model.test.mjs`: `coord agent start|set|list|show` against a temp `COORD_HOME`:
//! `--model` and `--why` are both required, the 12 char minimum and 200 char limit, opus needs a
//! category (or "owner asked: ..."), an opus chore or test needs an owner request, `set --model`
//! needs a new `--why`, the check-in time, and the list column and show output.

#![allow(clippy::too_many_lines, clippy::similar_names)]

mod agent_common;
mod common;

use agent_common::{As, Rig};
use serde_json::{Value, json};

fn start(rig: &Rig, name: &str, extra: &[&str]) -> common::Ran {
    let mut args = vec!["agent", "start", name, "do the work"];
    args.extend(extra);
    rig.call(&args, As::Lead)
}

fn start_ok(rig: &Rig, name: &str, extra: &[&str]) {
    let ran = start(rig, name, extra);
    assert_eq!(ran.code, 0, "{name} {extra:?}\n{}\n{}", ran.out, ran.err);
}

fn start_refused(rig: &Rig, name: &str, extra: &[&str], needle: &str) -> common::Ran {
    let ran = start(rig, name, extra);
    assert_ne!(ran.code, 0, "{name} {extra:?} should be refused");
    assert!(ran.err.contains(needle), "{:?} lacks {needle:?}", ran.err);
    ran
}

fn agent(rig: &Rig, name: &str) -> Value {
    rig.board()["agents"]
        .as_object()
        .and_then(|m| m.values().find(|a| a["name"] == name).cloned())
        .unwrap_or(Value::Null)
}

#[test]
fn start_refuses_a_missing_model_a_missing_why_a_short_or_long_why_and_a_two_word_model() {
    let rig = Rig::new();
    start_refused(
        &rig,
        "a1",
        &["--why", "known cause, one module"],
        "--model is required",
    );
    start_refused(&rig, "a1", &["--model", "sonnet"], "--why is required");
    start_refused(
        &rig,
        "a1",
        &["--model", "sonnet", "--why", "too short"],
        "minimum is 12",
    );
    let long = "x".repeat(201);
    start_refused(
        &rig,
        "a1",
        &["--model", "sonnet", "--why", &long],
        "limit is 200",
    );
    start_refused(
        &rig,
        "a1",
        &["--model", "two words", "--why", "known cause, one module"],
        "one word",
    );
    assert!(rig.board()["agents"].as_object().unwrap().is_empty());
}

#[test]
fn haiku_and_sonnet_take_any_justification_of_12_chars_trimmed_and_an_unknown_model_is_accepted() {
    let rig = Rig::new();
    start_ok(
        &rig,
        "h1",
        &[
            "--model",
            "haiku",
            "--why",
            "  runs the tests and lint  ",
            "--kind",
            "test",
        ],
    );
    start_ok(
        &rig,
        "s1",
        &["--model", "sonnet", "--why", "known cause, one-module fix"],
    );
    start_ok(
        &rig,
        "n1",
        &[
            "--model",
            "newmodel-9",
            "--why",
            "a newer model for this",
            "--checkin",
            "20",
        ],
    );
    assert_eq!(agent(&rig, "h1")["modelWhy"], "runs the tests and lint");
    assert_eq!(agent(&rig, "s1")["model"], "sonnet");
    assert_eq!(agent(&rig, "n1")["model"], "newmodel-9");
}

#[test]
fn opus_without_a_category_is_refused_with_the_standard_and_an_example() {
    let rig = Rig::new();
    let ran = start_refused(
        &rig,
        "o1",
        &["--model", "opus", "--why", "this looks hard to me"],
        "design, architecture, unknown-cause, risky-review, plan",
    );
    assert!(
        ran.err
            .contains("haiku for chores and tests, sonnet for most coding")
    );
    assert!(
        ran.err
            .contains("--why \"design: new document model touches every module\"")
    );
    start_refused(
        &rig,
        "o1",
        &["--model", "opus", "--why", "explain the thing at length"],
        "one of these categories",
    );
    assert_eq!(agent(&rig, "o1"), Value::Null);
}

#[test]
fn opus_with_a_category_word_is_accepted_for_build_and_research() {
    let rig = Rig::new();
    let cases = [
        (
            "o2",
            "design: new document model touches every module",
            "build",
        ),
        ("o3", "Unknown-Cause crash only on exit", "research"),
        ("o4", "risky-review of the migration", "build"),
        ("o5", "plan the rollout across projects", "build"),
        ("o6", "architecture: split the store", "build"),
    ];
    for (name, why, kind) in cases {
        start_ok(
            &rig,
            name,
            &[
                "--model",
                "opus",
                "--why",
                why,
                "--kind",
                kind,
                "--checkin",
                "25",
            ],
        );
    }
    assert_eq!(
        agent(&rig, "o3")["modelWhy"],
        "Unknown-Cause crash only on exit"
    );
}

#[test]
fn opus_for_a_chore_or_test_is_refused_unless_the_why_starts_with_owner() {
    let rig = Rig::new();
    let ran = start_refused(
        &rig,
        "c1",
        &[
            "--model",
            "opus",
            "--why",
            "design: tidy the folder layout",
            "--kind",
            "chore",
        ],
        "not for chore agents",
    );
    assert!(ran.err.contains("owner asked"));
    start_refused(
        &rig,
        "c1",
        &[
            "--model",
            "opus",
            "--why",
            "plan: run the whole suite",
            "--kind",
            "test",
        ],
        "not for test agents",
    );
    start_ok(
        &rig,
        "c2",
        &[
            "--model",
            "opus",
            "--why",
            "owner asked: use opus for the cleanup",
            "--kind",
            "chore",
            "--checkin",
            "15",
        ],
    );
    start_ok(
        &rig,
        "c3",
        &[
            "--model",
            "opus",
            "--why",
            "owner asked: opus",
            "--kind",
            "test",
            "--checkin",
            "15",
        ],
    );
    start_ok(
        &rig,
        "c4",
        &[
            "--model",
            "opus",
            "--why",
            "owner asked: no reason given",
            "--kind",
            "build",
            "--checkin",
            "15",
        ],
    );
    assert_eq!(agent(&rig, "c2")["kind"], "chore");
    assert_eq!(
        agent(&rig, "c2")["modelWhy"],
        "owner asked: use opus for the cleanup"
    );
}

#[test]
fn set_model_needs_a_new_why_with_the_same_rules_and_other_flags_are_unchanged() {
    let rig = Rig::new();
    start_ok(
        &rig,
        "w1",
        &[
            "--model",
            "haiku",
            "--why",
            "a chore for the cheap model",
            "--kind",
            "chore",
        ],
    );
    rig.refused(
        &["agent", "set", "w1", "--model", "sonnet"],
        As::Lead,
        "--why is required",
    );
    rig.refused(
        &["agent", "set", "w1", "--model", "sonnet", "--why", "short"],
        As::Lead,
        "minimum is 12",
    );
    rig.refused(
        &[
            "agent",
            "set",
            "w1",
            "--model",
            "opus",
            "--why",
            "design: nope nope",
        ],
        As::Lead,
        "not for chore agents",
    );
    assert_eq!(agent(&rig, "w1")["model"], "haiku");
    rig.ok(
        &[
            "agent",
            "set",
            "w1",
            "--model",
            "sonnet",
            "--why",
            "it turned out to need code changes",
        ],
        As::Lead,
    );
    assert_eq!(agent(&rig, "w1")["model"], "sonnet");
    assert_eq!(
        agent(&rig, "w1")["modelWhy"],
        "it turned out to need code changes"
    );
    rig.ok(&["agent", "set", "w1", "--branch", "feat/x"], As::Lead);
    assert_eq!(agent(&rig, "w1")["branch"], "feat/x");
    assert_eq!(
        agent(&rig, "w1")["modelWhy"],
        "it turned out to need code changes"
    );
    rig.ok(
        &[
            "agent",
            "set",
            "w1",
            "--kind",
            "build",
            "--model",
            "opus",
            "--why",
            "design: rework the data flow",
            "--checkin",
            "30",
        ],
        As::Lead,
    );
    let a = agent(&rig, "w1");
    assert_eq!(
        (
            a["model"].clone(),
            a["kind"].clone(),
            a["checkinMin"].clone()
        ),
        (json!("opus"), json!("build"), json!(30))
    );
}

#[test]
fn list_has_a_why_column_clipped_to_40_and_show_prints_the_why_in_full() {
    let rig = Rig::new();
    let long = "sonnet: the cause is known and the fix is one module, nothing more";
    start_ok(&rig, "l1", &["--model", "sonnet", "--why", long]);
    let list = rig.ok(&["agent", "list"], As::Lead);
    let first = list.out.lines().next().unwrap();
    assert!(
        first.contains("model") && first.contains("why") && first.contains("item"),
        "{first}"
    );
    let row = list
        .out
        .lines()
        .find(|l| l.contains(" l1 "))
        .expect("a row for l1");
    assert!(row.contains(&long[..36]), "the clipped why is listed");
    assert!(!row.contains(long), "but not in full");
    let id = agent(&rig, "l1")["id"].as_str().unwrap().to_owned();
    let show = rig.ok(&["agent", "show", &id], As::Lead);
    let shown: Value = serde_json::from_str(&show.out).unwrap();
    assert_eq!(shown["modelWhy"], long);
}

#[test]
fn a_record_without_modelwhy_lists_shows_and_takes_a_note_as_before() {
    let rig = Rig::new();
    rig.edit(|b| {
        let at = "2026-10-26T12:00:00.000Z";
        b["agents"]["A-90"] = json!({
            "id": "A-90", "name": "legacy", "thread": "coordinator", "task": "old one",
            "model": "sonnet", "kind": "build", "item": null, "branch": null, "worktree": null,
            "status": "working", "note": null, "result": null, "startedAt": at,
            "finishedAt": null, "createdAt": at, "createdBy": "coordinator", "updatedAt": at,
            "updatedBy": "coordinator", "rev": 1, "rank": 9999,
        });
    });
    assert!(rig.ok(&["agent", "list"], As::Lead).out.contains("legacy"));
    let show = rig.ok(&["agent", "show", "A-90"], As::Lead);
    let shown: Value = serde_json::from_str(&show.out).unwrap();
    assert_eq!(shown["modelWhy"], Value::Null);
    assert!(shown.get("modelWhy").is_none());
    rig.ok(&["agent", "note", "A-90", "still fine"], As::Lead);
}

const HELP: &str = "--checkin 15..30: minutes between check-in wakes for an opus-level agent, pick by assumed complexity (15 for risky or sprawling work, 30 for a well-bounded task)";

#[test]
fn checkin_is_required_for_an_opus_level_model_whole_minutes_from_15_to_30_and_kept_as_checkinmin()
{
    let rig = Rig::new();
    let opus = [
        "--model",
        "opus",
        "--why",
        "design: new document model touches every module",
    ];
    let ran = start_refused(&rig, "k1", &opus, "--checkin is required for opus");
    assert!(ran.err.contains(HELP));
    for bad in ["14", "31", "0", "20.5", "soon", "-20", "1e1"] {
        let mut extra = opus.to_vec();
        extra.extend(["--checkin", bad]);
        start_refused(
            &rig,
            "k1",
            &extra,
            "--checkin must be a whole number of minutes from 15 to 30",
        );
    }
    let mut bare = opus.to_vec();
    bare.push("--checkin");
    start_refused(&rig, "k1", &bare, "--checkin needs a value");
    assert_eq!(agent(&rig, "k1"), Value::Null);
    for (name, minutes) in [("k2", "15"), ("k3", "30")] {
        let mut extra = opus.to_vec();
        extra.extend(["--checkin", minutes]);
        start_ok(&rig, name, &extra);
    }
    assert_eq!(agent(&rig, "k2")["checkinMin"], 15);
    assert_eq!(agent(&rig, "k3")["checkinMin"], 30);
    let id = agent(&rig, "k3")["id"].as_str().unwrap().to_owned();
    let shown: Value =
        serde_json::from_str(&rig.ok(&["agent", "show", &id], As::Lead).out).unwrap();
    assert_eq!(shown["checkinMin"], 30);
}

#[test]
fn checkin_is_optional_and_not_kept_for_haiku_and_sonnet_but_a_bad_value_is_still_refused() {
    let rig = Rig::new();
    start_ok(
        &rig,
        "k4",
        &["--model", "sonnet", "--why", "known cause, one-module fix"],
    );
    start_ok(
        &rig,
        "k5",
        &[
            "--model",
            "haiku",
            "--why",
            "runs the tests and lint",
            "--checkin",
            "20",
        ],
    );
    assert!(agent(&rig, "k4").get("checkinMin").is_none());
    assert!(agent(&rig, "k5").get("checkinMin").is_none());
    start_refused(
        &rig,
        "k6",
        &[
            "--model",
            "sonnet",
            "--why",
            "known cause, one-module fix",
            "--checkin",
            "99",
        ],
        "from 15 to 30",
    );
}

#[test]
fn set_checkin_changes_it_and_a_new_opus_level_model_needs_one_unless_the_record_has_it() {
    let rig = Rig::new();
    let opus = [
        "--model",
        "opus",
        "--why",
        "design: new document model touches every module",
    ];
    let mut k2 = opus.to_vec();
    k2.extend(["--checkin", "15"]);
    start_ok(&rig, "k2", &k2);
    let k2_id = agent(&rig, "k2")["id"].as_str().unwrap().to_owned();
    rig.ok(&["agent", "set", &k2_id, "--checkin", "22"], As::Lead);
    assert_eq!(agent(&rig, "k2")["checkinMin"], 22);
    rig.refused(
        &["agent", "set", &k2_id, "--checkin", "5"],
        As::Lead,
        "from 15 to 30",
    );
    start_ok(
        &rig,
        "k7",
        &["--model", "sonnet", "--why", "known cause, one-module fix"],
    );
    let k7_id = agent(&rig, "k7")["id"].as_str().unwrap().to_owned();
    let redesign = [
        "agent",
        "set",
        &k7_id,
        "--model",
        "opus",
        "--why",
        "design: rework the data flow",
    ];
    rig.refused(&redesign, As::Lead, "--checkin is required for opus");
    assert_eq!(agent(&rig, "k7")["model"], "sonnet");
    let mut with = redesign.to_vec();
    with.extend(["--checkin", "18"]);
    rig.ok(&with, As::Lead);
    assert_eq!(
        (
            agent(&rig, "k7")["model"].clone(),
            agent(&rig, "k7")["checkinMin"].clone()
        ),
        (json!("opus"), json!(18))
    );
    rig.ok(
        &[
            "agent",
            "set",
            &k2_id,
            "--model",
            "opus",
            "--why",
            "design: still the same hard one",
        ],
        As::Lead,
    );
    assert_eq!(
        agent(&rig, "k2")["checkinMin"],
        22,
        "the record keeps the time it has"
    );
}

#[test]
fn registering_a_hook_recorded_agent_with_checkin_the_manual_value_wins() {
    let rig = Rig::new();
    rig.edit(|b| {
        let at = now_iso();
        b["agents"]["A-90"] = json!({
            "id": "A-90", "createdAt": at, "createdBy": "coordinator", "updatedAt": at,
            "updatedBy": "coordinator", "rank": 90000, "name": "rebuild-the-pad-overlay",
            "thread": "coordinator", "task": "Rebuild the pad overlay screen", "model": "opus",
            "modelWhy": null, "kind": "build", "item": null, "branch": null, "worktree": null,
            "status": "working", "note": null, "result": null, "startedAt": at,
            "finishedAt": null, "hookId": "hk-90", "source": "hook", "agentType": "general-purpose",
            "modelUsed": "claude-opus-5-5", "transcriptFile": null, "checkinMin": 20, "rev": 1,
        });
    });
    let before = rig.board()["agents"].as_object().unwrap().len();
    rig.ok(
        &[
            "agent",
            "start",
            "pad-overlay",
            "Rebuild the pad overlay screen with the new layout",
            "--model",
            "opus",
            "--why",
            "design: new overlay layout flow",
            "--checkin",
            "30",
        ],
        As::Lead,
    );
    let board = rig.board();
    assert_eq!(
        board["agents"].as_object().unwrap().len(),
        before,
        "bound to the hook record"
    );
    assert_eq!(board["agents"]["A-90"]["checkinMin"], 30);
    assert_eq!(board["agents"]["A-90"]["name"], "pad-overlay");
}

/// The time now, as the board writes it.
fn now_iso() -> String {
    argus_core::clock::now_iso()
}
