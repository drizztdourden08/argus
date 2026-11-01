//! Differential tests of the agent, build and launcher verbs: the same steps on the Node CLI and
//! on `argus coord`, each in its own temp home seeded alike, comparing the exit code, both
//! streams and every file after each step. A write verb is in here with its refusals; a mismatch
//! is a bug on the Rust side unless the harness lists the value as volatile. They skip with a
//! reason when node, `ai-config` or the `argus` binary is absent (`ARGUS_EXE`, or build
//! `-p argus`).

#![allow(clippy::too_many_lines, clippy::similar_names)]

use std::fs;

use crate::support::differ::fixture::SEED_TIME;
use crate::support::differ::{
    Differ, Engines, FixtureBoard, LEAD_SESSION, Report, Seed, Step, WORKER_SESSION,
};
use serde_json::{Value, json};
use tempfile::TempDir;

const ROOT: &str = "Z:/seed/work";

/// A step as `who`, on the project `alpha`.
fn st(args: &[&str], who: &str) -> Step {
    let mut argv = args.to_vec();
    argv.extend(["--project", "alpha", "--as", who]);
    Step::new(&argv)
}

fn lead(args: &[&str]) -> Step {
    st(args, "lead")
}

fn worker(args: &[&str]) -> Step {
    st(args, "worker")
}

fn other(args: &[&str]) -> Step {
    st(args, "other")
}

/// The seed: `alpha` with the coordinator `lead`, the threads `worker` and `other`, three items,
/// and what `edit` adds to the board file.
fn seed(edit: impl FnOnce(&mut Value)) -> TempDir {
    let board = FixtureBoard::new("alpha")
        .thread("lead", "coordinator", None)
        .thread("worker", "thread", None)
        .thread("other", "thread", None)
        .item("Plan the release", "lead", "doing", &["Notes written"])
        .item("Port the parser", "worker", "todo", &["Cases pass"])
        .item("Write the docs", "worker", "todo", &[]);
    let mut value: Value = serde_json::from_str(&board.to_json(ROOT)).expect("board json");
    edit(&mut value);
    let dir = TempDir::new().expect("seed dir");
    let folder = dir.path().join("alpha");
    fs::create_dir_all(&folder).expect("board folder");
    let text = serde_json::to_string_pretty(&value).expect("serialise");
    fs::write(folder.join("board.json"), format!("{text}\n")).expect("board file");
    dir
}

fn test_doc(id: &str, build: &str, mode: &str) -> Value {
    json!({
        "id": id, "createdAt": SEED_TIME, "createdBy": "worker", "updatedAt": SEED_TIME,
        "updatedBy": "worker", "rank": 1024, "item": "I-2", "title": id, "mode": mode,
        "status": "pending", "build": build, "thread": "worker", "rev": 1,
    })
}

fn run(test: &str, seed_dir: &TempDir, steps: &[Step]) -> Option<Report> {
    let engines = Engines::or_skip(test)?;
    let differ = Differ::new(engines, &Seed::CopyOf(seed_dir.path().to_path_buf()), &[])
        .expect("twin homes");
    let report = differ.run(steps).expect("run the steps");
    println!("{report}");
    Some(report)
}

fn check(test: &str, seed_dir: &TempDir, steps: &[Step]) {
    if let Some(report) = run(test, seed_dir, steps) {
        report.assert_no_mismatch();
        assert_eq!(report.not_ported(), 0, "{report}");
    }
}

const WHY: &str = "known cause, one-module fix";

#[test]
fn agents_start_note_set_and_end() {
    let steps = [
        lead(&[
            "agent",
            "start",
            "a1",
            "do the work",
            "--model",
            "sonnet",
            "--why",
            WHY,
        ]),
        lead(&[
            "agent",
            "start",
            "a1",
            "do the work again",
            "--model",
            "haiku",
            "--why",
            "runs the tests and lint",
            "--kind",
            "test",
        ]),
        lead(&[
            "agent",
            "start",
            "b2",
            "write docs",
            "--model",
            "opus",
            "--why",
            "design: new document model touches every module",
            "--checkin",
            "20",
            "--item",
            "I-3",
            "--branch",
            "feat/docs",
            "--worktree",
            "wt-docs",
        ]),
        lead(&[
            "agent",
            "start",
            "w-agent",
            "parser port",
            "--model",
            "sonnet",
            "--why",
            WHY,
            "--for",
            "worker",
        ]),
        worker(&[
            "agent",
            "start",
            "own",
            "a worker agent",
            "--model",
            "haiku",
            "--why",
            "a chore for the cheap model",
            "--kind",
            "chore",
        ]),
        worker(&[
            "agent",
            "start",
            "forbidden",
            "for the lead",
            "--model",
            "sonnet",
            "--why",
            WHY,
            "--for",
            "lead",
        ]),
        lead(&[
            "agent", "start", "x", "task", "--model", "sonnet", "--why", WHY, "--kind", "other",
        ]),
        lead(&[
            "agent", "start", "x", "task", "--model", "sonnet", "--why", WHY, "--item", "I-99",
        ]),
        lead(&[
            "agent", "start", "!!!", "task", "--model", "sonnet", "--why", WHY,
        ]),
        lead(&["agent", "start", "x"]),
        lead(&["agent", "start"]),
        lead(&[
            "agent",
            "start",
            "x",
            "task",
            "--model",
            "sonnet",
            "--why",
            WHY,
            "--claude-id",
            "bad id",
        ]),
        lead(&[
            "agent",
            "start",
            "cid",
            "task with id",
            "--model",
            "sonnet",
            "--why",
            WHY,
            "--claude-id",
            "a1b2c3d4e5f60718",
        ]),
        lead(&[
            "agent",
            "start",
            "emoji",
            "task",
            "--model",
            "haiku",
            "--why",
            "a justification with enough chars",
        ]),
        lead(&["agent", "note", "a1", "working on it"]),
        lead(&["agent", "note", "A-1", "a note on the replaced one"]),
        lead(&["agent", "note", "a1"]),
        lead(&["agent", "note", "nobody", "text"]),
        lead(&["agent", "note"]),
        other(&["agent", "note", "A-3", "not mine"]),
        worker(&["agent", "note", "w-agent", "mine"]),
        lead(&[
            "agent",
            "set",
            "a1",
            "--task",
            "a new task",
            "--kind",
            "research",
            "--item",
            "I-1",
            "--branch",
            "b1",
            "--worktree",
            "w1",
        ]),
        lead(&["agent", "set", "a1", "--item", ""]),
        lead(&["agent", "set", "a1", "--branch", ""]),
        lead(&["agent", "set", "a1"]),
        lead(&["agent", "set", "a1", "--status", "waiting"]),
        lead(&["agent", "set", "a1", "--status", "working"]),
        lead(&["agent", "set", "a1", "--status", "done"]),
        lead(&[
            "agent",
            "set",
            "a1",
            "--model",
            "opus",
            "--why",
            "design: rework the data flow",
            "--checkin",
            "25",
        ]),
        lead(&[
            "agent",
            "set",
            "a1",
            "--model",
            "sonnet",
            "--why",
            "back to the cheap one again",
        ]),
        lead(&["agent", "set", "a1", "--why", "a plainer reason for this"]),
        lead(&["agent", "set", "a1", "--checkin", "18"]),
        lead(&["agent", "set", "a1", "--checkin", "99"]),
        lead(&["agent", "set", "A-1", "--claude-id", "cid-001"]),
        lead(&["agent", "set", "cid", "--claude-id", "cid-001"]),
        lead(&["agent", "set", "a1", "--model"]),
        other(&["agent", "set", "w-agent", "--task", "taken over"]),
        lead(&["agent", "done", "a1", "all good"]),
        lead(&["agent", "done", "b2"]),
        lead(&["agent", "fail", "w-agent"]),
        lead(&["agent", "fail", "w-agent", "it could not parse"]),
        lead(&["agent", "stop", "own", "--for", "worker"]),
        lead(&["agent", "stop", "cid"]),
        lead(&["agent", "stop", "cid"]),
        lead(&["agent", "set", "A-4", "--status", "working"]),
        lead(&[
            "agent",
            "set",
            "A-5",
            "--status",
            "working",
            "--claude-id",
            "a1b2c3d4e5f60718",
        ]),
        lead(&["agent", "list"]),
        lead(&["agent", "list", "--all"]),
        lead(&["agent", "list", "--json"]),
        lead(&["agent", "list", "--thread", "worker", "--all"]),
        lead(&["agent", "list", "--status", "done"]),
        lead(&["agent", "list", "--status", "bogus"]),
        lead(&["agent", "list", "--status"]),
        lead(&["agent", "show", "A-1"]),
        lead(&["agent", "show", "a1"]),
        lead(&["agent", "show", "own", "--for", "worker"]),
        lead(&["agent", "show", "A-99"]),
        lead(&["agent", "show"]),
        lead(&["agent", "frobnicate"]),
        lead(&["agent"]),
    ];
    check("agents_start_note_set_and_end", &seed(|_| {}), &steps);
}

#[test]
fn texts_are_counted_in_utf16_units_at_each_limit() {
    let face = "\u{1F600}";
    let why_max = face.repeat(100);
    let why_over = face.repeat(101);
    let why_min = face.repeat(6);
    let task_max = face.repeat(60);
    let task_over = face.repeat(61);
    let note_max = face.repeat(50);
    let note_over = face.repeat(51);
    let listed = format!("a{}", face.repeat(30));
    let steps = [
        lead(&[
            "agent", "start", "e1", "task", "--model", "sonnet", "--why", &why_max,
        ]),
        lead(&[
            "agent", "start", "e2", "task", "--model", "sonnet", "--why", &why_over,
        ]),
        lead(&[
            "agent", "start", "e3", "task", "--model", "sonnet", "--why", &why_min,
        ]),
        lead(&[
            "agent", "start", "e4", &task_max, "--model", "sonnet", "--why", WHY,
        ]),
        lead(&[
            "agent", "start", "e5", &task_over, "--model", "sonnet", "--why", WHY,
        ]),
        lead(&[
            "agent", "start", "e6", &listed, "--model", "sonnet", "--why", WHY,
        ]),
        lead(&[
            "agent",
            "start",
            "\u{e9}mojis and spaces",
            "task",
            "--model",
            "sonnet",
            "--why",
            WHY,
        ]),
        lead(&["agent", "note", "e1", &note_max]),
        lead(&["agent", "note", "e1", &note_over]),
        lead(&[
            "agent",
            "start",
            "e7",
            "task",
            "--model",
            &face.repeat(21),
            "--why",
            WHY,
        ]),
        lead(&[
            "agent",
            "start",
            "e8",
            "task",
            "--model",
            "two\u{a0}words",
            "--why",
            WHY,
        ]),
        lead(&["agent", "list"]),
        lead(&["agent", "show", "e1"]),
        lead(&[
            "build",
            "add",
            "b-1",
            "--title",
            &face.repeat(40),
            "--cmd",
            &face.repeat(200),
        ]),
        lead(&["build", "add", "b-2", "--title", &face.repeat(41)]),
        lead(&[
            "launcher",
            "set",
            "l-1",
            "--cmd",
            &face.repeat(200),
            "--use-every",
            "7d",
            "--title",
            &face.repeat(40),
        ]),
        lead(&[
            "launcher",
            "set",
            "l-2",
            "--cmd",
            &face.repeat(201),
            "--use-every",
            "7d",
        ]),
        lead(&["launcher", "list"]),
        lead(&["build", "list"]),
    ];
    check(
        "texts_are_counted_in_utf16_units_at_each_limit",
        &seed(|_| {}),
        &steps,
    );
}

#[test]
fn session_callers_refresh_their_heartbeat_and_never_act_as_another_thread() {
    let Some(engines) = Engines::or_skip("session_callers") else {
        return;
    };
    let differ = Differ::standard(engines).expect("twin homes");
    let model = ["--model", "sonnet", "--why", WHY];
    let with = |args: &[&str]| {
        let mut all = args.to_vec();
        all.extend(model);
        all.extend(["--project", "alpha"]);
        Step::new(&all)
    };
    let plain = |args: &[&str]| {
        let mut all = args.to_vec();
        all.extend(["--project", "alpha"]);
        Step::new(&all)
    };
    let steps = [
        with(&["agent", "start", "lead-agent", "the lead's own task"]).session(LEAD_SESSION),
        with(&["agent", "start", "worker-agent", "the worker's own task"]).session(WORKER_SESSION),
        with(&[
            "agent",
            "start",
            "for-lead",
            "worker acting as lead",
            "--as",
            "lead",
        ])
        .session(WORKER_SESSION),
        with(&[
            "agent",
            "start",
            "for-worker",
            "lead acting as worker",
            "--as",
            "worker",
        ])
        .session(LEAD_SESSION),
        plain(&[
            "agent",
            "note",
            "worker-agent",
            "from the lead",
            "--for",
            "worker",
        ])
        .session(LEAD_SESSION),
        plain(&["agent", "note", "lead-agent", "from the worker"]).session(WORKER_SESSION),
        plain(&["agent", "set", "A-1", "--task", "changed by the worker"]).session(WORKER_SESSION),
        plain(&["agent", "stop", "A-2"]).session(WORKER_SESSION),
        plain(&["agent", "list", "--thread", "worker"]).session(WORKER_SESSION),
        plain(&["agent", "show", "worker-agent"]).session(WORKER_SESSION),
        plain(&["agent", "show", "lead-agent"]).session(WORKER_SESSION),
        plain(&["build", "add", "s-1", "--thread", "worker"]).session(WORKER_SESSION),
        plain(&["build", "hand", "s-1"]).session(WORKER_SESSION),
        plain(&["launcher", "set", "master", "--cmd", "game run"]).session(WORKER_SESSION),
        plain(&["launcher", "set", "master", "--title", "Main"]).session(LEAD_SESSION),
        plain(&["agent", "list"]).session("33333333-3333-4333-8333-333333333333"),
        plain(&["agent", "list"]),
    ];
    let report = differ.run(&steps).expect("run the steps");
    println!("{report}");
    report.assert_no_mismatch();
    assert_eq!(report.not_ported(), 0, "{report}");
}

#[test]
fn agents_soft_verbs_and_the_ownership_rules() {
    let steps = [
        lead(&[
            "agent", "start", "a1", "task one", "--model", "sonnet", "--why", WHY,
        ]),
        lead(&[
            "agent", "start", "a2", "task two", "--model", "sonnet", "--why", WHY,
        ]),
        lead(&[
            "agent",
            "start",
            "a3",
            "task three",
            "--model",
            "sonnet",
            "--why",
            WHY,
        ]),
        lead(&["agent", "done", "a3", "finished"]),
        lead(&["agent", "archive", "A-3", "--reason", "old news"]),
        lead(&["agent", "archive", "A-3"]),
        lead(&["agent", "list", "--archived"]),
        lead(&["agent", "unarchive", "A-3"]),
        lead(&["agent", "unarchive", "A-3"]),
        lead(&["agent", "rm", "A-1", "A-2", "--reason", "cleanup"]),
        lead(&["agent", "list", "--deleted"]),
        lead(&["agent", "list", "--all"]),
        lead(&["agent", "restore", "A-1"]),
        lead(&[
            "agent",
            "restore",
            "A-2",
            "--reason",
            "x".repeat(121).as_str(),
        ]),
        lead(&["agent", "restore", "A-2", "--if-rev", "1"]),
        lead(&["agent", "restore", "A-2", "--if-rev", "x"]),
        lead(&["agent", "restore", "a2"]),
        lead(&["agent", "archive"]),
        lead(&["agent", "rm", "nobody"]),
        other(&["agent", "archive", "A-3"]),
        worker(&[
            "agent",
            "start",
            "mine",
            "worker task",
            "--model",
            "haiku",
            "--why",
            "a chore for the cheap model",
            "--kind",
            "chore",
        ]),
        worker(&["agent", "rm", "mine"]),
        worker(&["agent", "restore", "A-4"]),
        other(&["agent", "restore", "A-4"]),
        lead(&["agent", "show", "A-4"]),
    ];
    check(
        "agents_soft_verbs_and_the_ownership_rules",
        &seed(|_| {}),
        &steps,
    );
}

#[test]
fn agents_bind_to_the_hook_record_and_resume_a_closed_one() {
    let dir = seed(|b| {
        let at = argus_core::clock::now_iso();
        b["agents"]["A-90"] = json!({
            "id": "A-90", "createdAt": at, "createdBy": "lead", "updatedAt": at, "updatedBy": "lead",
            "rank": 90000, "name": "rebuild-the-pad-overlay", "thread": "lead",
            "task": "Rebuild the pad overlay screen", "model": "opus", "modelWhy": null, "kind": "build",
            "item": null, "branch": null, "worktree": null, "status": "working", "note": null,
            "result": null, "startedAt": at, "finishedAt": null, "hookId": "hk-90", "source": "hook",
            "agentType": "general-purpose", "modelUsed": "claude-opus-5-5", "transcriptFile": null,
            "checkinMin": 20, "rev": 1,
        });
        b["agents"]["A-91"] = json!({
            "id": "A-91", "createdAt": at, "createdBy": "lead", "updatedAt": at, "updatedBy": "lead",
            "rank": 91000, "name": "closed-one", "thread": "lead", "task": "A closed sub-agent",
            "model": "sonnet", "modelWhy": null, "kind": "build", "item": null, "branch": null,
            "worktree": null, "status": "stopped", "note": null, "result": null, "startedAt": at,
            "finishedAt": at, "hookId": "hk-91", "claudeId": "hk-91", "source": "hook",
            "agentType": "general-purpose", "reaped": true, "leadBound": true, "rev": 1,
        });
        b["agentLaunches"] = json!([{ "thread": "lead", "claudeId": "launch-77", "at": at, "description": "Port the parser cases for the lexer" }]);
        b["counters"]["A"] = json!(91);
    });
    let steps = [
        lead(&[
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
        ]),
        lead(&["agent", "show", "A-90"]),
        lead(&[
            "agent",
            "start",
            "parser",
            "Port the parser cases for the lexer",
            "--model",
            "sonnet",
            "--why",
            WHY,
        ]),
        lead(&["agent", "show", "parser"]),
        lead(&[
            "agent",
            "start",
            "again",
            "closed sub-agent resumed",
            "--model",
            "sonnet",
            "--why",
            WHY,
            "--claude-id",
            "hk-91",
        ]),
        lead(&["agent", "show", "A-91"]),
        lead(&["agent", "stop", "A-91"]),
        lead(&["agent", "set", "A-91", "--status", "working"]),
        lead(&[
            "agent",
            "set",
            "A-91",
            "--status",
            "working",
            "--claude-id",
            "hk-91",
        ]),
        lead(&["agent", "set", "A-91", "--status", "waiting"]),
        lead(&["agent", "list", "--all"]),
    ];
    check(
        "agents_bind_to_the_hook_record_and_resume_a_closed_one",
        &dir,
        &steps,
    );
}

#[test]
fn agents_reconcile_says_the_dashboard_is_down() {
    let steps = [
        lead(&["agent", "reconcile"]),
        lead(&["agent", "reconcile", "--all", "--json"]),
    ];
    check(
        "agents_reconcile_says_the_dashboard_is_down",
        &seed(|_| {}),
        &steps,
    );
}

#[test]
fn builds_add_set_hand_list_and_remove() {
    let dir = seed(|b| {
        b["tests"]["T-1"] = test_doc("T-1", "bundle-4", "manual");
        b["counters"]["T"] = json!(1);
    });
    let steps = [
        lead(&["launcher", "set", "master", "--cmd", "game run"]),
        lead(&[
            "build",
            "add",
            "bundle-4",
            "--title",
            "Fourth bundle",
            "--note",
            "login and settings",
            "--cmd",
            "rotp launch bundle-4 --visible",
            "--cwd",
            "sub",
            "--thread",
            "worker",
        ]),
        lead(&[
            "build",
            "add",
            "bundle-4",
            "--cmd",
            "rotp launch bundle-4 --visible",
        ]),
        lead(&[
            "build",
            "set",
            "bundle-4",
            "--note",
            "",
            "--title",
            "Bundle four",
        ]),
        lead(&["build", "set", "bundle-4"]),
        lead(&["build", "set", "nope", "--title", "x"]),
        worker(&["build", "set", "bundle-4", "--cmd", "rotp launch bundle-4"]),
        worker(&["build", "add", "bundle-5", "--thread", "worker"]),
        other(&["build", "set", "bundle-4", "--title", "mine now"]),
        other(&["build", "add", "bundle-6", "--thread", "worker"]),
        worker(&["build", "add", "bundle-7"]),
        worker(&["build", "add", "bundle-8", "--thread", "ghost"]),
        lead(&[
            "build",
            "set",
            "bundle-4",
            "--cmd",
            "x".repeat(401).as_str(),
        ]),
        lead(&[
            "build",
            "set",
            "bundle-4",
            "--note",
            "n".repeat(601).as_str(),
        ]),
        lead(&["build", "set", "bundle-4", "--title"]),
        lead(&["build", "set", "bundle-4", "--thread"]),
        lead(&["build", "set", "bundle-4", "--window=maybe"]),
        lead(&[
            "build",
            "set",
            "bundle-4",
            "--launcher",
            "master",
            "--cmd",
            "x",
        ]),
        lead(&["build", "set", "bundle-4", "--launcher", "nope-1"]),
        lead(&["build", "set", "bundle-4", "--launcher", "Bad Name"]),
        lead(&["build", "rm", "bundle-4"]),
        lead(&["build", "rm", "bundle-4", "--force"]),
        worker(&["build", "rm", "bundle-5"]),
        lead(&["build", "rm", "nope"]),
        lead(&["build", "hand", "bundle-4"]),
        lead(&["build", "hand"]),
        worker(&["build", "hand", "bundle-4"]),
        lead(&[
            "build",
            "set",
            "bundle-4",
            "--title",
            "Bundle four",
            "--cmd",
            "rotp launch bundle-4 --visible",
            "--thread",
            "worker",
        ]),
        lead(&["build", "list"]),
        lead(&["build", "list", "--json"]),
        lead(&["build", "unhand"]),
        lead(&["build", "unhand"]),
        worker(&["build", "unhand"]),
        lead(&["build", "list"]),
        lead(&["build", "frobnicate"]),
        lead(&["build"]),
    ];
    check("builds_add_set_hand_list_and_remove", &dir, &steps);
}

#[test]
fn builds_launch_flags_play_definitions_and_the_launcher_made_for_a_bundle() {
    let dir = seed(|b| {
        b["runDefs"]["launcher-hand-made"] =
            json!({ "name": "launcher-hand-made", "cmd": "node c.js", "cwd": "x", "kind": "task" });
    });
    let steps = [
        lead(&["launcher", "set", "master", "--cmd", "game run"]),
        worker(&[
            "build",
            "add",
            "play-1",
            "--cmd",
            "node app.js",
            "--thread",
            "worker",
        ]),
        worker(&["build", "set", "play-1", "--cmd", "node app.js"]),
        worker(&["build", "set", "play-1", "--cmd", "node app.js --fast"]),
        worker(&["build", "set", "play-1", "--cwd", "sub"]),
        worker(&["build", "set", "play-1", "--cmd", ""]),
        worker(&["build", "set", "play-1", "--cmd", "node app.js"]),
        worker(&["build", "rm", "play-1"]),
        lead(&["build", "add", "win-1", "--cmd", "node app.js", "--window"]),
        lead(&["build", "set", "win-1", "--window"]),
        lead(&["build", "set", "win-1", "--cmd", "node app.js --x"]),
        lead(&["build", "set", "win-1", "--no-window"]),
        lead(&["build", "set", "win-1", "--window"]),
        lead(&["build", "set", "win-1", "--window=false"]),
        lead(&["build", "add", "V1.2 RC", "--cmd", "node a.js"]),
        lead(&["build", "add", "v1.2-rc", "--cmd", "node b.js"]),
        lead(&["build", "add", "hand-made", "--cmd", "node d.js"]),
        lead(&["build", "add", "hand-made"]),
        lead(&["build", "rm", "hand-made"]),
        lead(&[
            "build",
            "add",
            "far-1",
            "--cmd",
            "node e.js",
            "--cwd",
            "Z:/nowhere/at/all",
        ]),
        lead(&["build", "add", "bare-1"]),
        lead(&["build", "set", "bare-1", "--cwd", "sub"]),
        lead(&["build", "add", "...", "--cmd", "node z.js"]),
        worker(&[
            "build",
            "add",
            "link-1",
            "--launcher",
            "master",
            "--thread",
            "worker",
        ]),
        lead(&["launcher", "set", "master", "--cmd", "game run --new"]),
        worker(&["build", "set", "link-1", "--cwd", "sub"]),
        lead(&[
            "launcher",
            "set",
            "old-9",
            "--cmd",
            "game run --old",
            "--use-every",
            "7d",
        ]),
        lead(&["launcher", "retire", "old-9"]),
        worker(&["build", "set", "link-1", "--launcher", "old-9"]),
        lead(&[
            "launcher",
            "set",
            "feat-3",
            "--cmd",
            "game run --feat",
            "--use-every",
            "7d",
        ]),
        worker(&["build", "set", "link-1", "--launcher", "feat-3"]),
        lead(&["launcher", "retire", "feat-3"]),
        lead(&["build", "list"]),
        lead(&["build", "list", "--json"]),
        worker(&["build", "set", "link-1", "--launcher", ""]),
        worker(&["build", "set", "link-1", "--launcher", "master"]),
        worker(&["build", "set", "link-1", "--cmd", ""]),
        lead(&["launcher", "list", "--json"]),
    ];
    check(
        "builds_launch_flags_play_definitions_and_the_launcher_made_for_a_bundle",
        &dir,
        &steps,
    );
}

#[test]
fn builds_legacy_bundles_close_reopen_and_status() {
    let dir = seed(|b| {
        for (id, tag, mode) in [
            ("T-1", "c-1", "manual"),
            ("T-2", "c-1", "manual"),
            ("T-3", "c-1", "auto"),
        ] {
            b["tests"][id] = test_doc(id, tag, mode);
        }
        b["counters"]["T"] = json!(3);
        b["builds"]["old-rec"] = json!({
            "tag": "old-rec", "title": "Old", "note": null, "cmd": "node old.js", "cwd": "sub",
            "window": true, "thread": null, "createdAt": SEED_TIME, "updatedAt": SEED_TIME, "createdBy": "lead",
        });
        b["runDefs"]["bundle-old-rec"] = json!({
            "name": "bundle-old-rec", "label": "Play old-rec", "bundle": "old-rec", "cmd": "node old.js",
            "cwd": "Z:/seed/work/sub", "kind": "launch", "env": {}, "hand": true, "visible": true,
            "window": true, "timeoutSec": 0, "parallel": 1, "definedBy": "lead", "definedAt": SEED_TIME, "rev": 1,
        });
        b["builds"]["master"] = json!({
            "tag": "master", "title": null, "note": null, "cmd": "node legacy-master.js", "cwd": null,
            "thread": null, "createdAt": SEED_TIME, "updatedAt": SEED_TIME, "createdBy": "lead",
        });
        for (tag, extra) in [
            ("old-launcher", json!({ "launcher": "master" })),
            ("old-cmd", json!({ "cmd": "node o.js" })),
            ("old-handed", json!({ "lastHandedAt": SEED_TIME })),
            ("old-bare", json!({})),
        ] {
            let mut rec = json!({ "tag": tag, "title": null, "note": null, "launcher": null, "thread": null, "createdAt": SEED_TIME, "updatedAt": SEED_TIME, "createdBy": "lead" });
            for (k, v) in extra.as_object().unwrap() {
                rec[k] = v.clone();
            }
            b["builds"][tag] = rec;
        }
    });
    let steps = [
        lead(&["launcher", "set", "master", "--cmd", "game run"]),
        lead(&["build", "list", "--json"]),
        lead(&["build", "set", "old-rec", "--title", "Old one"]),
        lead(&["build", "set", "master", "--note", "n"]),
        lead(&["build", "hand", "old-bare"]),
        lead(&["build", "hand", "old-launcher"]),
        lead(&["build", "unhand"]),
        worker(&[
            "build",
            "add",
            "c-1",
            "--thread",
            "worker",
            "--cmd",
            "node c.js",
        ]),
        worker(&["build", "close", "c-1"]),
        worker(&[
            "build",
            "close",
            "c-1",
            "--force",
            "--note",
            "landed in 1.4",
        ]),
        worker(&["build", "close", "c-1"]),
        worker(&["build", "set", "c-1", "--title", "Settings bundle"]),
        worker(&["build", "add", "c-1", "--note", "n"]),
        lead(&["build", "list"]),
        worker(&["build", "reopen", "c-1"]),
        worker(&["build", "reopen", "c-1"]),
        other(&["build", "reopen", "c-1"]),
        other(&["build", "close", "c-1"]),
        lead(&["build", "add", "h-1", "--cmd", "node h.js"]),
        lead(&["build", "hand", "h-1"]),
        lead(&["build", "ready", "h-1"]),
        lead(&["build", "hand", "h-1"]),
        lead(&["build", "draft", "h-1"]),
        lead(&["build", "close", "h-1"]),
        lead(&["build", "hand", "h-1"]),
        lead(&["build", "ready", "h-1"]),
        lead(&["build", "reopen", "h-1"]),
        lead(&["build", "hand", "h-1"]),
        lead(&["build", "unhand"]),
        lead(&["build", "draft", "h-1"]),
        lead(&["build", "draft", "h-1"]),
        lead(&["build", "ready", "nope-1"]),
        other(&["build", "ready", "h-1"]),
        lead(&["build", "close", "ghost-1"]),
        worker(&["build", "close", "ghost-2"]),
        lead(&["build", "close", "j-1", "--note"]),
        lead(&["build", "reopen", "nope-1"]),
        lead(&["build", "list", "--json"]),
        lead(&[
            "build",
            "add",
            "j-1",
            "--thread",
            "worker",
            "--cmd",
            "node j.js",
        ]),
        lead(&[
            "launcher",
            "set",
            "shared-1",
            "--cmd",
            "game run --shared",
            "--use-every",
            "30d",
        ]),
        lead(&[
            "build",
            "add",
            "j-2",
            "--thread",
            "worker",
            "--launcher",
            "shared-1",
        ]),
        lead(&["build", "close", "j-2"]),
        lead(&["build", "close", "j-1"]),
        lead(&["build", "reopen", "j-1"]),
        lead(&["launcher", "retire", "j-1", "--note", "replaced"]),
        lead(&["build", "close", "j-1"]),
        lead(&["build", "reopen", "j-1"]),
        lead(&["build", "set", "j-1", "--cmd", "node j.js"]),
        lead(&[
            "build",
            "add",
            "j-3",
            "--thread",
            "worker",
            "--launcher",
            "j-1",
        ]),
        lead(&["build", "close", "j-1"]),
        lead(&["build", "close", "j-3"]),
        lead(&["build", "reopen", "j-1"]),
        lead(&[
            "build",
            "add",
            "r-1",
            "--thread",
            "worker",
            "--cmd",
            "node r.js",
        ]),
        lead(&["build", "rm", "r-1"]),
        lead(&["build", "list", "--json"]),
    ];
    check(
        "builds_legacy_bundles_close_reopen_and_status",
        &dir,
        &steps,
    );
}

#[test]
fn builds_hand_needs_a_master_launcher() {
    let steps = [
        lead(&["build", "hand", "x-1"]),
        lead(&["launcher", "list"]),
        lead(&["launcher", "set", "master", "--cmd", "game run"]),
        lead(&["build", "hand", "x-1"]),
        lead(&["build", "list"]),
    ];
    check("builds_hand_needs_a_master_launcher", &seed(|_| {}), &steps);
}

#[test]
fn launchers_set_retire_keep_list_and_show() {
    let day = 86_400_000_i64;
    let old = |days: i64| argus_core::clock::iso_from_ms(argus_core::clock::now_ms() - days * day);
    let dir = seed(|b| {
        b["launchers"]["old-tool"] = json!({
            "name": "old-tool", "title": null, "cmd": "node old.js", "cwd": null, "kind": "app",
            "branch": null, "commit": null, "note": null, "current": false, "retiredAt": null,
            "retiredBy": null, "retireNote": null, "createdAt": old(30), "createdBy": "worker",
            "updatedAt": old(20), "by": "worker",
        });
        b["runDefs"]["launcher-taken"] =
            json!({ "name": "launcher-taken", "cmd": "node mine.js", "cwd": "x", "kind": "task" });
    });
    let steps = [
        worker(&[
            "launcher",
            "set",
            "build-2",
            "--cmd",
            "bxel worktree launch build-2 none --visible",
            "--title",
            "Build two",
            "--branch",
            "feat/two",
            "--commit",
            "abc1234",
            "--note",
            "the login fix",
            "--kind",
            "dev",
            "--current",
            "--use-every",
            "7d",
        ]),
        worker(&[
            "launcher",
            "set",
            "build-2",
            "--cmd",
            "bxel worktree launch build-2 none --visible",
        ]),
        worker(&[
            "launcher",
            "set",
            "build-2",
            "--note",
            "",
            "--not-current",
            "--cwd",
            "sub",
        ]),
        worker(&["launcher", "set", "build-2", "--window"]),
        worker(&["launcher", "set", "build-2", "--no-window", "--cwd", ""]),
        worker(&["launcher", "set", "build-2"]),
        lead(&["launcher", "set", "Build_2", "--cmd", "x"]),
        lead(&["launcher", "set", "x".repeat(32).as_str(), "--cmd", "x"]),
        lead(&[
            "launcher",
            "set",
            "x".repeat(31).as_str(),
            "--cmd",
            "x",
            "--use-every",
            "7d",
        ]),
        lead(&[
            "launcher",
            "set",
            "long-1",
            "--cmd",
            "c".repeat(401).as_str(),
        ]),
        lead(&[
            "launcher",
            "set",
            "long-1",
            "--cmd",
            "c".repeat(400).as_str(),
            "--use-every",
            "7d",
        ]),
        lead(&[
            "launcher",
            "set",
            "long-1",
            "--note",
            "n".repeat(301).as_str(),
        ]),
        lead(&[
            "launcher",
            "set",
            "long-1",
            "--commit",
            "c".repeat(41).as_str(),
        ]),
        lead(&["launcher", "set", "long-1", "--kind", "game"]),
        lead(&["launcher", "set", "long-1", "--current", "--not-current"]),
        lead(&["launcher", "set", "long-1", "--current", "x"]),
        lead(&["launcher", "set", "long-1", "--window", "--window"]),
        lead(&["launcher", "set", "long-1", "--cmd"]),
        lead(&["launcher", "set", "long-1", "--title"]),
        lead(&["launcher", "set", "fresh-1", "--title", "no command"]),
        lead(&["launcher", "set", "long-1", "--cmd", ""]),
        lead(&["launcher", "set"]),
        lead(&["launcher", "show", "nope"]),
        lead(&["launcher", "show"]),
        lead(&["launcher", "retire", "nope"]),
        lead(&["launcher", "frobnicate"]),
        other(&["launcher", "set", "build-2", "--title", "mine now"]),
        other(&["launcher", "retire", "build-2"]),
        lead(&["launcher", "set", "build-2", "--title", "Build two (lead)"]),
        other(&[
            "launcher",
            "set",
            "other-1",
            "--cmd",
            "node other.js",
            "--use-every",
            "2w",
        ]),
        lead(&[
            "launcher",
            "set",
            "master",
            "--cmd",
            "game run",
            "--title",
            "Main checkout",
        ]),
        lead(&["launcher", "set", "build-2", "--current"]),
        lead(&["launcher", "list"]),
        lead(&["launcher", "list", "--json"]),
        lead(&["launcher", "retire", "master"]),
        lead(&["launcher", "set", "master", "--cmd", "game run --fast"]),
        lead(&["launcher", "show", "master"]),
        lead(&["launcher", "show", "build-2", "--json"]),
        lead(&[
            "launcher",
            "retire",
            "build-2",
            "--note",
            "merged into main",
        ]),
        lead(&["launcher", "retire", "build-2"]),
        lead(&["launcher", "list"]),
        lead(&["launcher", "list", "--all"]),
        lead(&["launcher", "show", "build-2"]),
        lead(&["launcher", "set", "build-2"]),
        lead(&[
            "launcher",
            "set",
            "taken",
            "--cmd",
            "node theirs.js",
            "--use-every",
            "7d",
        ]),
        lead(&[
            "launcher",
            "set",
            "multi-1",
            "--cmd",
            "line one\nline two",
            "--use-every",
            "7d",
        ]),
        lead(&["launcher", "list"]),
        lead(&[
            "launcher",
            "set",
            "shelly",
            "--cmd",
            "export NODE_ENV=dev; cd apps && echo '$HOME'",
            "--use-every",
            "7d",
        ]),
        lead(&["launcher", "set", "shelly", "--note", "only a note changed"]),
        lead(&["launcher", "show", "shelly"]),
        worker(&["launcher", "set", "tool-1", "--cmd", "node tool.js"]),
        worker(&[
            "launcher",
            "set",
            "tool-1",
            "--cmd",
            "node tool.js",
            "--use-every",
            "weekly",
        ]),
        worker(&[
            "launcher",
            "set",
            "tool-1",
            "--cmd",
            "node tool.js",
            "--use-every",
            "366d",
        ]),
        worker(&[
            "launcher",
            "set",
            "tool-1",
            "--cmd",
            "node tool.js",
            "--use-every",
            "",
        ]),
        worker(&[
            "launcher",
            "set",
            "tool-1",
            "--cmd",
            "node tool.js",
            "--use-every",
        ]),
        worker(&[
            "launcher",
            "set",
            "tool-1",
            "--cmd",
            "node tool.js",
            "--use-every",
            "2W",
        ]),
        worker(&["launcher", "set", "tool-1", "--note", "a note"]),
        worker(&["launcher", "set", "tool-1", "--use-every", "3d"]),
        worker(&["launcher", "set", "old-tool", "--current"]),
        lead(&["launcher", "list"]),
        worker(&[
            "launcher",
            "set",
            "old-tool",
            "--current",
            "--use-every",
            "12h",
        ]),
        lead(&["launcher", "show", "old-tool", "--json"]),
        worker(&[
            "launcher",
            "set",
            "keep-1",
            "--cmd",
            "node keep.js",
            "--use-every",
            "1d",
        ]),
        worker(&[
            "launcher",
            "keep",
            "keep-1",
            "--note",
            "still the nightly check",
        ]),
        lead(&["launcher", "keep", "keep-1", "--days", "30"]),
        lead(&["launcher", "show", "keep-1"]),
        worker(&["launcher", "keep", "keep-1", "--days", "0"]),
        worker(&["launcher", "keep", "keep-1", "--days", "400"]),
        worker(&["launcher", "keep", "keep-1", "--days", "soon"]),
        worker(&["launcher", "keep", "keep-1", "--days"]),
        worker(&["launcher", "keep", "keep-1", "--note"]),
        other(&["launcher", "keep", "keep-1"]),
        lead(&["launcher", "keep", "master"]),
        lead(&["launcher", "keep", "nope"]),
        worker(&["launcher", "keep", "keep-1"]),
        worker(&["launcher", "retire", "keep-1"]),
        worker(&["launcher", "keep", "keep-1"]),
        worker(&["launcher", "keep", "tool-1"]),
        worker(&["launcher", "set", "keep-1", "--use-every", "9d"]),
        lead(&["launcher", "list", "--json", "--all"]),
    ];
    check("launchers_set_retire_keep_list_and_show", &dir, &steps);
}
