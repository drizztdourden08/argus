//! Ports `launcher.verbs.test.mjs`: `coord launcher set|keep|retire|list|show` against a temp
//! `COORD_HOME`: the record and its every field, the refusals (name, limits, kind, a new one without
//! a command, an empty command, master retired, a caller with no thread, another thread's
//! launcher), the Play definition `launcher-<name>` kept in step, the order master first then
//! current, retire and bring back, the usage window and `keep`. The page docs (`exportDocs`) and
//! the pure usage rule (`usageOf`, `parseEvery`) have their own tests in `argus-board` and
//! `argus-launchers`.

#![allow(clippy::too_many_lines, clippy::similar_names)]

use crate::agent_common::{As, Rig};
use argus_core::clock::{iso_from_ms, now_ms};
use serde_json::{Value, json};

fn keys(v: &Value, names: &[&str]) -> Vec<Value> {
    names.iter().map(|n| v[*n].clone()).collect()
}

fn launcher(rig: &Rig, name: &str) -> Value {
    rig.board()["launchers"][name].clone()
}

fn list_json(rig: &Rig, extra: &[&str]) -> Vec<Value> {
    let mut args = vec!["launcher", "list", "--json"];
    args.extend(extra);
    let parsed: Value = serde_json::from_str(&rig.ok(&args, As::Nobody).out).unwrap();
    parsed["launchers"].as_array().unwrap().clone()
}

fn made_build_2(rig: &Rig) -> crate::common::Ran {
    rig.ok(
        &[
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
        ],
        As::Worker,
    )
}

#[test]
fn set_makes_a_launcher_with_every_field_and_its_play_definition_the_same_set_again_changes_nothing()
 {
    let rig = Rig::new();
    let r = made_build_2(&rig);
    assert!(r.out.contains("build-2: launcher added"), "{}", r.out);
    assert!(r.out.contains("Play ready (launcher-build-2"));
    assert!(
        r.out
            .contains("warning: this project has no master launcher and needs one"),
        "no master yet"
    );
    let l = launcher(&rig, "build-2");
    assert_eq!(
        keys(
            &l,
            &[
                "name",
                "title",
                "cmd",
                "cwd",
                "kind",
                "branch",
                "commit",
                "note",
                "current",
                "retiredAt",
                "createdBy",
                "by"
            ]
        ),
        [
            json!("build-2"),
            json!("Build two"),
            json!("bxel worktree launch build-2 none --visible"),
            Value::Null,
            json!("dev"),
            json!("feat/two"),
            json!("abc1234"),
            json!("the login fix"),
            json!(true),
            Value::Null,
            json!("worker"),
            json!("worker")
        ]
    );
    assert!(l["createdAt"].is_string() && l["updatedAt"].is_string());
    let def = rig.board()["runDefs"]["launcher-build-2"].clone();
    assert_eq!(
        keys(
            &def,
            &[
                "name",
                "label",
                "launcher",
                "cmd",
                "cwd",
                "kind",
                "hand",
                "visible",
                "window",
                "parallel",
                "timeoutSec",
                "rev"
            ]
        ),
        [
            json!("launcher-build-2"),
            json!("Play build-2"),
            json!("build-2"),
            json!("bxel worktree launch build-2 none --visible"),
            json!(rig.root()),
            json!("launch"),
            json!(true),
            json!(false),
            json!(false),
            json!(1),
            json!(0),
            json!(1)
        ]
    );
    let again = rig.ok(
        &[
            "launcher",
            "set",
            "build-2",
            "--cmd",
            "bxel worktree launch build-2 none --visible",
        ],
        As::Worker,
    );
    assert!(again.out.contains("build-2: unchanged."), "{}", again.out);
    assert_eq!(rig.board()["runDefs"]["launcher-build-2"]["rev"], 1);
    assert_eq!(launcher(&rig, "build-2")["updatedAt"], l["updatedAt"]);
}

#[test]
fn set_changes_only_the_flags_given_empty_clears_a_text_field_not_current_window_and_no_window() {
    let rig = Rig::new();
    made_build_2(&rig);
    let r = rig.ok(
        &[
            "launcher",
            "set",
            "build-2",
            "--note",
            "",
            "--not-current",
            "--cwd",
            "sub",
        ],
        As::Worker,
    );
    assert!(
        r.out
            .contains("build-2: updated (cwd, note, current). Play updated"),
        "{}",
        r.out
    );
    let l = launcher(&rig, "build-2");
    assert_eq!(
        keys(&l, &["note", "current", "cwd", "title", "kind"]),
        [
            Value::Null,
            json!(false),
            json!("sub"),
            json!("Build two"),
            json!("dev")
        ]
    );
    let sub = rig.home.path().join("sub").to_string_lossy().into_owned();
    assert_eq!(
        rig.board()["runDefs"]["launcher-build-2"]["cwd"],
        json!(sub)
    );
    assert!(
        r.out.contains("does not exist yet"),
        "a missing folder is said, not refused"
    );
    rig.ok(&["launcher", "set", "build-2", "--window"], As::Worker);
    assert_eq!(launcher(&rig, "build-2")["window"], true);
    let def = rig.board()["runDefs"]["launcher-build-2"].clone();
    assert_eq!(
        keys(&def, &["visible", "window"]),
        [json!(true), json!(true)]
    );
    rig.ok(
        &["launcher", "set", "build-2", "--no-window", "--cwd", ""],
        As::Worker,
    );
    assert!(
        launcher(&rig, "build-2").get("window").is_none(),
        "a window that is off is not stored"
    );
    let def = rig.board()["runDefs"]["launcher-build-2"].clone();
    assert_eq!(
        keys(&def, &["window", "cwd"]),
        [json!(false), json!(rig.root())]
    );
    rig.refused(
        &["launcher", "set", "build-2"],
        As::Worker,
        "nothing to change",
    );
}

#[test]
fn refusals_a_bad_name_limits_a_kind_no_command_for_a_new_one_an_empty_command() {
    let rig = Rig::new();
    rig.refused(
        &["launcher", "set", "Build_2", "--cmd", "x"],
        As::Lead,
        "not a launcher name",
    );
    rig.refused(
        &["launcher", "set", &"x".repeat(32), "--cmd", "x"],
        As::Lead,
        "not a launcher name",
    );
    rig.ok(
        &[
            "launcher",
            "set",
            &"x".repeat(31),
            "--cmd",
            "x",
            "--use-every",
            "7d",
        ],
        As::Lead,
    );
    rig.refused(
        &["launcher", "set", "long-1", "--cmd", &"c".repeat(401)],
        As::Lead,
        "limit is 400",
    );
    rig.ok(
        &[
            "launcher",
            "set",
            "long-1",
            "--cmd",
            &"c".repeat(400),
            "--use-every",
            "7d",
        ],
        As::Lead,
    );
    rig.refused(
        &["launcher", "set", "long-1", "--note", &"n".repeat(301)],
        As::Lead,
        "limit is 300",
    );
    rig.refused(
        &["launcher", "set", "long-1", "--commit", &"c".repeat(41)],
        As::Lead,
        "limit is 40",
    );
    rig.refused(
        &["launcher", "set", "long-1", "--kind", "game"],
        As::Lead,
        "--kind must be one of app, dev, other",
    );
    rig.refused(
        &["launcher", "set", "long-1", "--current", "--not-current"],
        As::Lead,
        "not both",
    );
    rig.refused(
        &["launcher", "set", "fresh-1", "--title", "no command"],
        As::Lead,
        "launcher fresh-1 is new: give it the command",
    );
    assert!(rig.board()["launchers"].get("fresh-1").is_none());
    rig.refused(
        &["launcher", "set", "long-1", "--cmd", ""],
        As::Lead,
        "a launcher always has a command",
    );
    rig.refused(
        &["launcher", "show", "nope"],
        As::Nobody,
        "no launcher \"nope\"",
    );
    rig.refused(
        &["launcher", "retire", "nope"],
        As::Lead,
        "no launcher \"nope\"",
    );
    rig.refused(
        &["launcher", "frobnicate"],
        As::Nobody,
        "launcher verb must be one of set, keep, retire, list, show",
    );
}

#[test]
fn who_may_write_a_thread_makes_one_only_its_maker_or_the_coordinator_change_it_no_thread_no_write()
{
    let rig = Rig::new();
    made_build_2(&rig);
    rig.refused(
        &["launcher", "set", "build-2", "--title", "mine now"],
        As::Other,
        "launcher build-2 was set by worker; only worker or the coordinator",
    );
    rig.refused(
        &["launcher", "retire", "build-2"],
        As::Other,
        "set by worker",
    );
    rig.ok(
        &["launcher", "set", "build-2", "--title", "Build two (lead)"],
        As::Lead,
    );
    assert_eq!(launcher(&rig, "build-2")["by"], "coordinator");
    rig.ok(
        &[
            "launcher",
            "set",
            "other-1",
            "--cmd",
            "node other.js",
            "--use-every",
            "2w",
        ],
        As::Other,
    );
    let argv = [
        "launcher",
        "set",
        "ghost-1",
        "--cmd",
        "x",
        "--project",
        "alpha",
    ];
    let outside = std::env::temp_dir();
    let ran = crate::common::coord(&argv, rig.home.env(), &outside);
    assert_ne!(ran.code, 0);
    assert!(ran.err.contains("no thread registered"), "{}", ran.err);
    assert!(rig.board()["launchers"].get("ghost-1").is_none());
}

#[test]
fn master_is_listed_first_then_the_current_ones_it_is_never_retired_only_updated() {
    let rig = Rig::new();
    made_build_2(&rig);
    rig.ok(
        &[
            "launcher",
            "set",
            "master",
            "--cmd",
            "game run",
            "--title",
            "Main checkout",
        ],
        As::Lead,
    );
    rig.ok(&["launcher", "set", "build-2", "--current"], As::Lead);
    let list = list_json(&rig, &[]);
    let names: Vec<&Value> = list.iter().take(2).map(|l| &l["name"]).collect();
    assert_eq!(names, [&json!("master"), &json!("build-2")]);
    assert_eq!(
        list.iter().find(|l| l["name"] == "master").unwrap()["play"],
        "launcher-master"
    );
    let text = rig.ok(&["launcher", "list"], As::Nobody).out;
    assert!(
        text.lines().next().unwrap().starts_with("name")
            && text.contains("kind")
            && text.contains("current")
    );
    assert!(
        text.lines()
            .any(|l| l.starts_with("build-2") && l.contains("dev") && l.contains("current"))
    );
    assert!(!text.contains("no master launcher"));
    rig.refused(
        &["launcher", "retire", "master"],
        As::Lead,
        "master is never retired",
    );
    assert_eq!(launcher(&rig, "master")["retiredAt"], Value::Null);
    rig.ok(
        &["launcher", "set", "master", "--cmd", "game run --fast"],
        As::Lead,
    );
    let def = rig.board()["runDefs"]["launcher-master"].clone();
    assert_eq!(
        (def["cmd"].clone(), def["rev"].clone()),
        (json!("game run --fast"), json!(2))
    );
    let show = rig.ok(&["launcher", "show", "master"], As::Nobody).out;
    assert!(show.starts_with("master: Main checkout"), "{show}");
    assert!(
        show.contains("command  game run --fast") && show.contains("play     launcher-master"),
        "{show}"
    );
}

#[test]
fn retire_is_off_the_list_back_with_all_no_play_not_current_set_brings_it_back_with_its_play() {
    let rig = Rig::new();
    rig.with_master();
    made_build_2(&rig);
    let r = rig.ok(
        &[
            "launcher",
            "retire",
            "build-2",
            "--note",
            "merged into main",
        ],
        As::Lead,
    );
    assert!(
        r.out
            .contains("build-2: retired. Play removed (launcher-build-2)"),
        "{}",
        r.out
    );
    let l = launcher(&rig, "build-2");
    assert_eq!(
        keys(&l, &["retiredBy", "retireNote", "current"]),
        [
            json!("coordinator"),
            json!("merged into main"),
            json!(false)
        ]
    );
    assert!(l["retiredAt"].is_string());
    assert!(rig.board()["runDefs"].get("launcher-build-2").is_none());
    assert!(
        rig.ok(&["launcher", "retire", "build-2"], As::Lead)
            .out
            .contains("already retired")
    );
    assert!(!list_json(&rig, &[]).iter().any(|x| x["name"] == "build-2"));
    let all = list_json(&rig, &["--all"]);
    assert_eq!(
        all.iter().find(|x| x["name"] == "build-2").unwrap()["play"],
        Value::Null
    );
    let back = rig.ok(&["launcher", "set", "build-2"], As::Lead);
    assert!(
        back.out
            .contains("build-2: brought back from retired. Play ready"),
        "{}",
        back.out
    );
    assert_eq!(launcher(&rig, "build-2")["retiredAt"], Value::Null);
    assert!(rig.board()["runDefs"].get("launcher-build-2").is_some());
    assert!(
        rig.board()["log"]
            .to_string()
            .contains("retired launcher build-2: merged into main"),
        "the audit log keeps it"
    );
}

#[test]
fn the_play_name_a_definition_made_by_hand_is_never_taken_a_multi_line_command_gets_no_play() {
    let rig = Rig::new();
    rig.edit(|b| {
        b["runDefs"]["launcher-taken"] =
            json!({ "name": "launcher-taken", "cmd": "node mine.js", "cwd": "x", "kind": "task" });
    });
    rig.refused(
        &[
            "launcher",
            "set",
            "taken",
            "--cmd",
            "node theirs.js",
            "--use-every",
            "7d",
        ],
        As::Lead,
        "the run name launcher-taken is already taken by a launch defined with coord run define",
    );
    let board = rig.board();
    assert!(
        board["launchers"].get("taken").is_none(),
        "nothing is written"
    );
    assert_eq!(board["runDefs"]["launcher-taken"]["cmd"], "node mine.js");
    let multi = rig.ok(
        &[
            "launcher",
            "set",
            "multi-1",
            "--cmd",
            "line one\nline two",
            "--use-every",
            "7d",
        ],
        As::Lead,
    );
    assert!(
        multi
            .out
            .contains("no Play button: the command spans several lines"),
        "{}",
        multi.out
    );
    assert!(rig.board()["runDefs"].get("launcher-multi-1").is_none());
    assert!(
        rig.ok(&["launcher", "list"], As::Nobody)
            .out
            .lines()
            .any(|l| l.contains("multi-1") && l.contains("no Play"))
    );
}

#[test]
fn a_board_written_before_launchers_reads_with_launchers_empty() {
    let rig = Rig::new();
    rig.edit(|b| {
        b.as_object_mut().unwrap().shift_remove("launchers");
    });
    let out = rig.ok(&["launcher", "list"], As::Nobody).out;
    assert!(out.contains("no launcher yet"), "{out}");
}

#[test]
fn the_command_line_is_explained_when_set_which_shell_runs_it_hidden_and_warnings_never_block() {
    let rig = Rig::new();
    let shell = if cfg!(windows) {
        "cmd.exe /d /s /c"
    } else {
        "sh -c"
    };
    let good = rig.ok(
        &[
            "launcher",
            "set",
            "shelly",
            "--cmd",
            "bxel build --if-stale && bxel start",
            "--use-every",
            "7d",
        ],
        As::Lead,
    );
    assert!(
        good.out
            .contains("command line: bxel build --if-stale && bxel start")
    );
    assert!(
        good.out.contains(&format!(
            "Play runs the command line with {shell} from the launcher's folder (--cwd), hidden"
        )),
        "{}",
        good.out
    );
    assert!(
        good.out
            .contains("stdin is a pipe (nothing can ask a question)")
    );
    assert!(good.out.contains(
        "chain with && (not ; and not a newline), use double quotes, set the folder with --cwd"
    ));
    assert!(
        !good.out.contains("warning: ") || good.out.contains("no master launcher"),
        "{}",
        good.out
    );
    if cfg!(windows) {
        let bad = rig.ok(
            &[
                "launcher",
                "set",
                "shelly",
                "--cmd",
                "export NODE_ENV=dev; cd apps && echo '$HOME'",
            ],
            As::Lead,
        );
        for needle in [
            "warning: cmd.exe does not expand $VAR",
            "warning: export and source are bash",
            "warning: single quotes do not quote in cmd.exe",
            "warning: \";\" does not chain commands",
            "warning: do not cd: set the folder with --cwd",
        ] {
            assert!(bad.out.contains(needle), "{needle}\n{}", bad.out);
        }
        assert_eq!(
            launcher(&rig, "shelly")["cmd"],
            "export NODE_ENV=dev; cd apps && echo '$HOME'",
            "a warning never blocks"
        );
    }
    let shown = rig.ok(&["launcher", "show", "shelly"], As::Lead).out;
    assert!(
        shown.contains(&format!(
            "runs     {shell}, from the folder above, hidden, stdin a pipe"
        )),
        "{shown}"
    );
    assert!(shown.lines().any(|l| l.starts_with("command ")));
    let json: Value = serde_json::from_str(
        &rig.ok(&["launcher", "show", "shelly", "--json"], As::Lead)
            .out,
    )
    .unwrap();
    assert_eq!(json["launcher"]["runsWith"], shell);
    assert!(
        rig.ok(&["launcher", "list"], As::Lead)
            .out
            .contains("command line")
    );
    let again = rig.ok(
        &["launcher", "set", "shelly", "--note", "only a note changed"],
        As::Lead,
    );
    assert!(
        !again.out.contains("Play runs the command line"),
        "the explanation comes with a command"
    );
}

#[test]
fn use_every_is_required_for_a_new_standalone_launcher_validated_and_kept_when_set_once() {
    let rig = Rig::new();
    let none = rig.refused(
        &["launcher", "set", "tool-1", "--cmd", "node tool.js"],
        As::Worker,
        "needs the window you expect it to be used in: add --use-every <window> (12h, 3d, 2w or 1mo, from 1h to 365d)",
    );
    assert!(none.err.contains("reported to you: keep it, retire it or change the window. master and a launcher made for a bundle do not take it"));
    assert!(
        rig.board()["launchers"].get("tool-1").is_none(),
        "nothing is written"
    );
    for (value, needle) in [
        ("weekly", "--use-every \"weekly\" is not a duration"),
        ("366d", "--use-every 366d is out of range"),
        ("30m", "not a duration"),
        ("", "not a duration"),
    ] {
        rig.refused(
            &[
                "launcher",
                "set",
                "tool-1",
                "--cmd",
                "node tool.js",
                "--use-every",
                value,
            ],
            As::Worker,
            needle,
        );
    }
    let made = rig.ok(
        &[
            "launcher",
            "set",
            "tool-1",
            "--cmd",
            "node tool.js",
            "--use-every",
            "2W",
        ],
        As::Worker,
    );
    assert!(
        made.out.contains("tool-1: launcher added (") && made.out.contains("used every 2w"),
        "{}",
        made.out
    );
    assert_eq!(launcher(&rig, "tool-1")["useEvery"], "2w");
    rig.ok(
        &["launcher", "set", "tool-1", "--note", "a note"],
        As::Worker,
    );
    assert_eq!(launcher(&rig, "tool-1")["useEvery"], "2w");
    let changed = rig.ok(
        &["launcher", "set", "tool-1", "--use-every", "3d"],
        As::Worker,
    );
    assert!(
        changed.out.contains("tool-1: updated (useEvery)"),
        "{}",
        changed.out
    );
    assert_eq!(launcher(&rig, "tool-1")["useEvery"], "3d");
}

fn days_ago(days: i64) -> String {
    iso_from_ms(now_ms() - days * 86_400_000)
}

#[test]
fn a_launcher_from_before_the_field_has_none_and_is_treated_as_7d_resaving_it_asks_for_the_field() {
    let rig = Rig::new();
    rig.edit(|b| {
        b["launchers"]["old-tool"] = json!({
            "name": "old-tool", "title": null, "cmd": "node old.js", "cwd": null, "kind": "app",
            "branch": null, "commit": null, "note": null, "current": false, "retiredAt": null,
            "retiredBy": null, "retireNote": null, "createdAt": days_ago(30), "createdBy": "worker",
            "updatedAt": days_ago(20), "by": "worker",
        });
    });
    let shown: Value = serde_json::from_str(
        &rig.ok(&["launcher", "show", "old-tool", "--json"], As::Nobody)
            .out,
    )
    .unwrap();
    let row = &shown["launcher"];
    assert_eq!(
        keys(
            row,
            &["standalone", "useEveryEffective", "useEveryLegacy", "stale"]
        ),
        [json!(true), json!("7d"), json!(true), json!(true)]
    );
    let list = rig.ok(&["launcher", "list"], As::Nobody).out;
    assert!(
        list.lines().any(|l| l.contains("old-tool")
            && l.contains("7d (default), unused")
            && l.contains("STALE")),
        "{list}"
    );
    assert!(list.contains("old-tool: no --use-every yet (treated as 7d)"));
    rig.refused(
        &["launcher", "set", "old-tool", "--current"],
        As::Worker,
        "launcher old-tool is a standalone build",
    );
    rig.ok(
        &[
            "launcher",
            "set",
            "old-tool",
            "--current",
            "--use-every",
            "12h",
        ],
        As::Worker,
    );
    let after: Value = serde_json::from_str(
        &rig.ok(&["launcher", "show", "old-tool", "--json"], As::Nobody)
            .out,
    )
    .unwrap();
    let row = &after["launcher"];
    assert_eq!(
        keys(row, &["useEvery", "useEveryLegacy", "stale"]),
        [json!("12h"), json!(false), json!(false)],
        "saving it counts as a use"
    );
}

#[test]
fn keep_counts_as_a_use_without_running_it_who_and_why_are_recorded_days_holds_it_fresh() {
    let rig = Rig::new();
    rig.with_master();
    rig.ok(
        &[
            "launcher",
            "set",
            "keep-1",
            "--cmd",
            "node keep.js",
            "--use-every",
            "1d",
        ],
        As::Worker,
    );
    let before = launcher(&rig, "keep-1");
    let r = rig.ok(
        &[
            "launcher",
            "keep",
            "keep-1",
            "--note",
            "still the nightly check",
        ],
        As::Worker,
    );
    assert!(r.out.contains("keep-1: kept: it counts as used now, so the next 1d are quiet (still the nightly check)"), "{}", r.out);
    let l = launcher(&rig, "keep-1");
    assert_eq!(
        keys(&l, &["keptBy", "keptNote", "updatedAt"]),
        [
            json!("worker"),
            json!("still the nightly check"),
            before["updatedAt"].clone()
        ]
    );
    assert!(l.get("keptUntil").is_none());
    assert!(l["keptAt"].is_string());
    let days = rig.ok(&["launcher", "keep", "keep-1", "--days", "30"], As::Lead);
    assert!(
        days.out.contains("keep-1: kept for 30 days"),
        "{}",
        days.out
    );
    let held = launcher(&rig, "keep-1");
    assert_eq!(held["keptBy"], "coordinator");
    assert!(held["keptUntil"].is_string());
    assert!(
        rig.board()["log"]
            .to_string()
            .contains("kept launcher keep-1 for 30d")
    );
    rig.refused(
        &["launcher", "keep", "keep-1", "--days", "0"],
        As::Worker,
        "--days takes a whole number of days, 1 to 365",
    );
    rig.refused(
        &["launcher", "keep", "keep-1", "--days", "400"],
        As::Worker,
        "1 to 365",
    );
    rig.refused(
        &["launcher", "keep", "keep-1", "--days", "soon"],
        As::Worker,
        "1 to 365",
    );
    rig.refused(
        &["launcher", "keep", "keep-1"],
        As::Other,
        "launcher keep-1 was set by worker",
    );
    rig.refused(
        &["launcher", "keep", "master"],
        As::Lead,
        "master has no usage window",
    );
    rig.refused(
        &["launcher", "keep", "nope"],
        As::Lead,
        "no launcher \"nope\"",
    );
    rig.ok(&["launcher", "retire", "keep-1"], As::Worker);
    rig.refused(
        &["launcher", "keep", "keep-1"],
        As::Worker,
        "launcher keep-1 is retired",
    );
}
