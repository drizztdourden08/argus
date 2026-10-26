//! Ports `build.verbs.test.mjs`: `coord build add|set|rm|close|reopen|ready|draft|hand|unhand|list`
//! against a temp `COORD_HOME`: the bundle record, who may manage it, limits, the refusal to remove
//! a bundle while live tests carry its tag, `--cmd` making the launcher named after the tag,
//! `--launcher` linking one, a bundle written before launchers, close and reopen, and the status.
//! The page docs (`exportDocs`) are the sync group's and are checked there.

#![allow(clippy::too_many_lines, clippy::similar_names)]

mod agent_common;
mod common;

use agent_common::{As, Rig};
use serde_json::{Value, json};

fn bundle_4(rig: &Rig) -> common::Ran {
    let root = rig.root();
    rig.ok(
        &[
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
            &root,
            "--thread",
            "worker",
        ],
        As::Lead,
    )
}

fn rig_with_bundle_4() -> Rig {
    let rig = Rig::new();
    rig.with_master();
    rig.with_test();
    bundle_4(&rig);
    rig
}

fn keys(v: &Value, names: &[&str]) -> Vec<Value> {
    names.iter().map(|n| v[*n].clone()).collect()
}

fn list_json(rig: &Rig) -> Value {
    serde_json::from_str(&rig.ok(&["build", "list", "--json"], As::Nobody).out).unwrap()
}

fn row(rig: &Rig, tag: &str) -> Value {
    list_json(rig)["builds"]
        .as_array()
        .and_then(|rows| rows.iter().find(|r| r["build"] == tag).cloned())
        .unwrap_or(Value::Null)
}

#[test]
fn add_makes_the_record_and_with_cmd_the_launcher_named_after_the_tag_that_it_links() {
    let rig = Rig::new();
    rig.with_master();
    rig.with_test();
    let added = bundle_4(&rig);
    assert!(
        added.out.contains("bundle-4: bundle added ("),
        "{}",
        added.out
    );
    assert!(added.out.contains("launcher bundle-4, cmd: rotp launch bundle-4 --visible). Launcher bundle-4 set. Play ready (launcher-bundle-4"));
    let board = rig.board();
    let rec = &board["builds"]["bundle-4"];
    assert_eq!(
        keys(
            rec,
            &["tag", "title", "note", "launcher", "thread", "createdBy"]
        ),
        [
            json!("bundle-4"),
            json!("Fourth bundle"),
            json!("login and settings"),
            json!("bundle-4"),
            json!("worker"),
            json!("coordinator")
        ]
    );
    assert!(
        rec.get("cmd").is_none() && rec.get("cwd").is_none(),
        "the bundle carries no command of its own"
    );
    assert!(rec["createdAt"].is_string() && rec["updatedAt"].is_string());
    let l = &board["launchers"]["bundle-4"];
    assert_eq!(
        keys(l, &["cmd", "cwd", "bundle", "createdBy", "kind"]),
        [
            json!("rotp launch bundle-4 --visible"),
            json!(rig.root()),
            json!("bundle-4"),
            json!("coordinator"),
            json!("app")
        ]
    );
    let again = rig.ok(
        &[
            "build",
            "add",
            "bundle-4",
            "--cmd",
            "rotp launch bundle-4 --visible",
        ],
        As::Lead,
    );
    assert!(again.out.contains("unchanged"), "{}", again.out);
    assert_eq!(
        rig.board()["builds"]["bundle-4"]["createdAt"],
        rec["createdAt"]
    );
}

#[test]
fn set_changes_named_fields_only_and_an_empty_value_clears_a_field() {
    let rig = rig_with_bundle_4();
    rig.ok(
        &[
            "build",
            "set",
            "bundle-4",
            "--note",
            "",
            "--title",
            "Bundle four",
        ],
        As::Lead,
    );
    let board = rig.board();
    assert_eq!(board["builds"]["bundle-4"]["note"], Value::Null);
    assert_eq!(board["builds"]["bundle-4"]["title"], "Bundle four");
    assert_eq!(
        board["launchers"]["bundle-4"]["cmd"],
        "rotp launch bundle-4 --visible"
    );
    rig.refused(&["build", "set", "bundle-4"], As::Lead, "nothing to change");
    rig.refused(
        &["build", "set", "nope", "--title", "x"],
        As::Lead,
        "no bundle \"nope\"",
    );
}

#[test]
fn the_thread_named_by_thread_manages_its_own_bundle_another_thread_and_a_missing_thread_are_refused()
 {
    let rig = rig_with_bundle_4();
    rig.ok(
        &["build", "set", "bundle-4", "--cmd", "rotp launch bundle-4"],
        As::Worker,
    );
    assert_eq!(
        rig.board()["launchers"]["bundle-4"]["cmd"],
        "rotp launch bundle-4"
    );
    let mine = rig.ok(
        &["build", "add", "bundle-5", "--thread", "worker"],
        As::Worker,
    );
    assert!(mine.out.contains("bundle added"));
    rig.refused(
        &["build", "set", "bundle-4", "--title", "mine now"],
        As::Other,
        "worker",
    );
    rig.fails(
        &["build", "add", "bundle-6", "--thread", "worker"],
        As::Other,
    );
    rig.refused(&["build", "add", "bundle-7"], As::Worker, "--thread worker");
    let board = rig.board();
    assert!(board["builds"].get("bundle-6").is_none() && board["builds"].get("bundle-7").is_none());
    rig.ok(
        &["launcher", "set", "bundle-4", "--branch", "feat/login"],
        As::Worker,
    );
    rig.refused(
        &["launcher", "set", "bundle-4", "--branch", "x"],
        As::Other,
        "launcher bundle-4 was set by coordinator; only the coordinator (or a thread that manages bundle bundle-4) can change it",
    );
}

#[test]
fn limits_cmd_400_chars_note_600_chars() {
    let rig = rig_with_bundle_4();
    rig.refused(
        &["build", "set", "bundle-4", "--cmd", &"x".repeat(401)],
        As::Lead,
        "limit is 400",
    );
    rig.ok(
        &["build", "set", "bundle-4", "--cmd", &"x".repeat(400)],
        As::Lead,
    );
    rig.refused(
        &["build", "set", "bundle-4", "--note", &"n".repeat(601)],
        As::Lead,
        "limit is 600",
    );
    rig.ok(
        &[
            "build",
            "set",
            "bundle-4",
            "--note",
            &"n".repeat(600),
            "--cmd",
            "rotp launch bundle-4 --visible",
        ],
        As::Lead,
    );
}

#[test]
fn rm_refuses_while_live_tests_carry_the_tag_unless_force_the_launcher_made_for_it_is_retired() {
    let rig = rig_with_bundle_4();
    rig.ok(
        &["build", "add", "bundle-5", "--thread", "worker"],
        As::Worker,
    );
    rig.refused(
        &["build", "rm", "bundle-4"],
        As::Lead,
        "1 live test(s) still carry build bundle-4",
    );
    assert!(rig.board()["builds"].get("bundle-4").is_some());
    let forced = rig.ok(&["build", "rm", "bundle-4", "--force"], As::Lead);
    assert!(
        forced
            .out
            .contains("bundle removed (launcher bundle-4 retired)"),
        "{}",
        forced.out
    );
    let board = rig.board();
    assert!(board["builds"].get("bundle-4").is_none());
    assert!(
        board["launchers"]["bundle-4"]["retiredAt"].is_string(),
        "retired, not deleted"
    );
    assert!(board["runDefs"].get("launcher-bundle-4").is_none());
    assert_eq!(board["tests"]["T-1"]["build"], "bundle-4");
    let free = rig.ok(&["build", "rm", "bundle-5"], As::Worker);
    assert!(free.out.contains("bundle removed"));
}

#[test]
fn hand_makes_a_missing_record_and_list_shows_title_the_launcher_its_cmd_thread_and_counts() {
    let rig = rig_with_bundle_4();
    rig.ok(&["build", "rm", "bundle-4", "--force"], As::Lead);
    rig.ok(&["build", "hand", "bundle-4"], As::Lead);
    let board = rig.board();
    assert_eq!(board["handedBuild"], "bundle-4");
    let rec = &board["builds"]["bundle-4"];
    assert_eq!(
        keys(rec, &["tag", "launcher", "status"]),
        [json!("bundle-4"), Value::Null, json!("ready")]
    );
    let back = rig.ok(
        &[
            "build",
            "set",
            "bundle-4",
            "--title",
            "Bundle four",
            "--cmd",
            "rotp launch bundle-4 --visible",
            "--thread",
            "worker",
        ],
        As::Lead,
    );
    assert!(back.out.contains("updated"), "{}", back.out);
    assert_eq!(
        rig.board()["launchers"]["bundle-4"]["retiredAt"],
        Value::Null
    );
    let text = rig.ok(&["build", "list"], As::Nobody).out;
    assert!(text.contains("handed to the owner: bundle-4"));
    assert!(text.contains("Bundle four") && text.contains("rotp launch bundle-4 --visible"));
    let json = list_json(&rig);
    assert_eq!(json["handed"]["build"], "bundle-4");
    let r = row(&rig, "bundle-4");
    assert_eq!(
        keys(
            &r,
            &[
                "title",
                "launcher",
                "launcherState",
                "cmd",
                "thread",
                "tests",
                "toRun",
                "pass",
                "fail",
                "handed"
            ]
        ),
        [
            json!("Bundle four"),
            json!("bundle-4"),
            json!("linked"),
            json!("rotp launch bundle-4 --visible"),
            json!("worker"),
            json!(1),
            json!(1),
            json!(0),
            json!(0),
            json!(true)
        ]
    );
}

#[test]
fn a_board_written_before_builds_existed_reads_with_builds_empty() {
    let rig = rig_with_bundle_4();
    rig.edit(|b| {
        b.as_object_mut().unwrap().shift_remove("builds");
    });
    let list = list_json(&rig);
    let tagged = list["builds"]
        .as_array()
        .unwrap()
        .iter()
        .any(|b| b["build"] == "bundle-4" && b["record"] == false);
    assert!(tagged, "the tag still lists, without a record");
    rig.ok(&["build", "add", "bundle-9"], As::Lead);
    assert!(rig.board()["builds"].get("bundle-9").is_some());
}

fn def_of(board: &Value, name: &str) -> Value {
    board["runDefs"][name].clone()
}

#[test]
fn a_cmd_writes_the_launcher_and_its_run_definition_a_changed_cmd_or_cwd_updates_it_clearing_or_rm_retires_it()
 {
    let rig = Rig::new();
    rig.with_master();
    let root = rig.root();
    let added = rig.ok(
        &[
            "build",
            "add",
            "play-1",
            "--cmd",
            "node app.js",
            "--thread",
            "worker",
        ],
        As::Worker,
    );
    assert!(
        added.out.contains("Play ready (launcher-play-1"),
        "{}",
        added.out
    );
    let def = def_of(&rig.board(), "launcher-play-1");
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
                "rev",
                "definedBy"
            ]
        ),
        [
            json!("launcher-play-1"),
            json!("Play play-1"),
            json!("play-1"),
            json!("node app.js"),
            json!(root),
            json!("launch"),
            json!(true),
            json!(false),
            json!(false),
            json!(1),
            json!(0),
            json!(1),
            json!("worker")
        ]
    );
    assert!(rig.board()["runDefs"].get("bundle-play-1").is_none());
    rig.ok(
        &["build", "set", "play-1", "--cmd", "node app.js"],
        As::Worker,
    );
    assert_eq!(def_of(&rig.board(), "launcher-play-1")["rev"], 1);
    rig.ok(
        &["build", "set", "play-1", "--cmd", "node app.js --fast"],
        As::Worker,
    );
    let def = def_of(&rig.board(), "launcher-play-1");
    assert_eq!(
        (def["cmd"].clone(), def["rev"].clone()),
        (json!("node app.js --fast"), json!(2))
    );
    let sub = rig.home.folder("sub");
    rig.ok(&["build", "set", "play-1", "--cwd", "sub"], As::Worker);
    let def = def_of(&rig.board(), "launcher-play-1");
    assert_eq!(
        (def["cwd"].clone(), def["rev"].clone()),
        (json!(sub.to_string_lossy()), json!(3))
    );
    let cleared = rig.ok(&["build", "set", "play-1", "--cmd", ""], As::Worker);
    assert!(
        cleared
            .out
            .contains("updated (launcher). Launcher play-1 retired"),
        "{}",
        cleared.out
    );
    let board = rig.board();
    assert_eq!(board["builds"]["play-1"]["launcher"], Value::Null);
    assert!(board["runDefs"].get("launcher-play-1").is_none());
    rig.ok(
        &["build", "set", "play-1", "--cmd", "node app.js"],
        As::Worker,
    );
    let board = rig.board();
    assert!(board["runDefs"].get("launcher-play-1").is_some());
    assert_eq!(
        board["launchers"]["play-1"]["retiredAt"],
        Value::Null,
        "the same name comes back"
    );
    assert!(
        rig.ok(&["build", "rm", "play-1"], As::Worker)
            .out
            .contains("bundle removed")
    );
    assert!(rig.board()["runDefs"].get("launcher-play-1").is_none());
}

#[test]
fn window_gives_the_launcher_a_visible_console_and_no_window_takes_it_away() {
    let rig = Rig::new();
    rig.with_master();
    let added = rig.ok(
        &["build", "add", "win-1", "--cmd", "node app.js", "--window"],
        As::Lead,
    );
    assert!(added.out.contains("with a console window"), "{}", added.out);
    let board = rig.board();
    assert_eq!(board["launchers"]["win-1"]["window"], true);
    assert!(board["builds"]["win-1"].get("window").is_none());
    let def = def_of(&board, "launcher-win-1");
    assert_eq!(
        keys(&def, &["visible", "window", "hand", "rev"]),
        [json!(true), json!(true), json!(true), json!(1)]
    );
    let same = rig.ok(&["build", "set", "win-1", "--window"], As::Lead);
    assert!(same.out.contains("unchanged"));
    assert_eq!(def_of(&rig.board(), "launcher-win-1")["rev"], 1);
    rig.ok(
        &["build", "set", "win-1", "--cmd", "node app.js --x"],
        As::Lead,
    );
    let def = def_of(&rig.board(), "launcher-win-1");
    assert_eq!(
        (def["window"].clone(), def["rev"].clone()),
        (json!(true), json!(2))
    );
    let off = rig.ok(&["build", "set", "win-1", "--no-window"], As::Lead);
    assert!(off.out.contains("updated (window)"), "{}", off.out);
    let board = rig.board();
    assert!(board["launchers"]["win-1"].get("window").is_none());
    let def = def_of(&board, "launcher-win-1");
    assert_eq!(
        keys(&def, &["visible", "window", "rev"]),
        [json!(false), json!(false), json!(3)]
    );
    rig.ok(&["build", "set", "win-1", "--window"], As::Lead);
    rig.ok(&["build", "set", "win-1", "--window=false"], As::Lead);
    assert_eq!(def_of(&rig.board(), "launcher-win-1")["window"], false);
}

#[test]
fn the_launcher_name_is_a_slug_of_the_tag_a_clash_or_a_run_name_made_by_hand_is_refused() {
    let rig = Rig::new();
    rig.with_master();
    rig.ok(&["build", "add", "V1.2 RC", "--cmd", "node a.js"], As::Lead);
    let board = rig.board();
    assert_eq!(board["builds"]["V1.2 RC"]["launcher"], "v1-2-rc");
    assert_eq!(board["launchers"]["v1-2-rc"]["bundle"], "V1.2 RC");
    assert!(board["runDefs"].get("launcher-v1-2-rc").is_some());
    rig.refused(
        &["build", "add", "v1.2-rc", "--cmd", "node b.js"],
        As::Lead,
        "launcher v1-2-rc is already linked to bundle V1.2 RC",
    );
    let board = rig.board();
    assert!(board["builds"].get("v1.2-rc").is_none());
    assert_eq!(board["launchers"]["v1-2-rc"]["cmd"], "node a.js");
    rig.edit(|b| {
        b["runDefs"]["launcher-hand-made"] =
            json!({ "name": "launcher-hand-made", "cmd": "node c.js", "cwd": "x", "kind": "task" });
    });
    rig.refused(
        &["build", "add", "hand-made", "--cmd", "node d.js"],
        As::Lead,
        "already taken by a launch defined with coord run define",
    );
    let board = rig.board();
    assert_eq!(board["runDefs"]["launcher-hand-made"]["cmd"], "node c.js");
    assert!(
        board["launchers"].get("hand-made").is_none(),
        "nothing is written"
    );
    rig.ok(&["build", "add", "hand-made"], As::Lead);
    rig.ok(&["build", "rm", "hand-made"], As::Lead);
    assert!(rig.board()["runDefs"].get("launcher-hand-made").is_some());
    let nowhere = rig.home.path().join("nowhere");
    let far = rig.ok(
        &[
            "build",
            "add",
            "far-1",
            "--cmd",
            "node e.js",
            "--cwd",
            &nowhere.to_string_lossy(),
        ],
        As::Lead,
    );
    assert!(far.out.contains("does not exist yet"), "{}", far.out);
    assert!(rig.board()["runDefs"].get("launcher-far-1").is_some());
    rig.ok(&["build", "add", "bare-1"], As::Lead);
    rig.refused(
        &["build", "set", "bare-1", "--cwd", "sub"],
        As::Lead,
        "bundle bare-1 has no launcher yet: pass --cmd",
    );
}

#[test]
fn launcher_links_an_existing_launcher_a_missing_or_retired_one_is_refused_and_empty_unlinks() {
    let rig = Rig::new();
    rig.with_master();
    rig.ok(
        &[
            "build",
            "add",
            "link-1",
            "--launcher",
            "master",
            "--thread",
            "worker",
        ],
        As::Worker,
    );
    assert_eq!(rig.board()["builds"]["link-1"]["launcher"], "master");
    let state = |rig: &Rig| {
        let r = row(rig, "link-1");
        (
            r["launcher"].clone(),
            r["launcherState"].clone(),
            r["cmd"].clone(),
        )
    };
    assert_eq!(
        state(&rig),
        (json!("master"), json!("linked"), json!("game run"))
    );
    rig.ok(
        &["launcher", "set", "master", "--cmd", "game run --new"],
        As::Lead,
    );
    assert_eq!(row(&rig, "link-1")["cmd"], "game run --new");
    rig.refused(
        &["build", "set", "link-1", "--launcher", "nope-1"],
        As::Worker,
        "no launcher \"nope-1\": coord launcher set nope-1",
    );
    rig.refused(
        &["build", "set", "link-1", "--launcher", "Bad Name"],
        As::Worker,
        "not a launcher name",
    );
    rig.refused(
        &[
            "build",
            "set",
            "link-1",
            "--launcher",
            "master",
            "--cmd",
            "x",
        ],
        As::Worker,
        "pass --launcher <name> or --cmd",
    );
    rig.refused(
        &["build", "set", "link-1", "--cwd", "sub"],
        As::Worker,
        "starts launcher master: change its folder or window with coord launcher set master",
    );
    rig.ok(
        &[
            "launcher",
            "set",
            "old-9",
            "--cmd",
            "game run --old",
            "--use-every",
            "7d",
        ],
        As::Lead,
    );
    rig.ok(&["launcher", "retire", "old-9"], As::Lead);
    rig.refused(
        &["build", "set", "link-1", "--launcher", "old-9"],
        As::Worker,
        "launcher old-9 is retired",
    );
    rig.ok(
        &[
            "launcher",
            "set",
            "feat-3",
            "--cmd",
            "game run --feat",
            "--use-every",
            "7d",
        ],
        As::Lead,
    );
    rig.ok(
        &["build", "set", "link-1", "--launcher", "feat-3"],
        As::Worker,
    );
    let ret = rig.ok(&["launcher", "retire", "feat-3"], As::Lead);
    assert!(
        ret.out.contains("Bundle link-1 still links it"),
        "{}",
        ret.out
    );
    assert_eq!(
        state(&rig),
        (json!("feat-3"), json!("retired"), Value::Null)
    );
    assert!(
        rig.ok(&["build", "list"], As::Nobody)
            .out
            .contains("feat-3 (retired)")
    );
    let un = rig.ok(&["build", "set", "link-1", "--launcher", ""], As::Worker);
    assert!(
        un.out.trim_end().ends_with("link-1: updated (launcher)."),
        "{}",
        un.out
    );
    assert_eq!(rig.board()["builds"]["link-1"]["launcher"], Value::Null);
    rig.ok(
        &["build", "set", "link-1", "--launcher", "master"],
        As::Worker,
    );
    rig.ok(&["build", "set", "link-1", "--cmd", ""], As::Worker);
    assert_eq!(
        rig.board()["launchers"]["master"]["retiredAt"],
        Value::Null,
        "master is never retired by a bundle"
    );
}

#[test]
fn a_legacy_bundle_keeps_its_play_the_next_add_or_set_moves_the_command_to_its_launcher() {
    let rig = Rig::new();
    rig.with_master();
    let sub = rig.home.path().join("sub").to_string_lossy().into_owned();
    rig.edit(|b| {
        let at = b["createdAt"].clone();
        b["builds"]["old-rec"] = json!({
            "tag": "old-rec", "title": "Old", "note": null, "cmd": "node old.js", "cwd": "sub",
            "window": true, "thread": null, "createdAt": at, "updatedAt": at, "createdBy": "coordinator",
        });
        b["runDefs"]["bundle-old-rec"] = json!({
            "name": "bundle-old-rec", "label": "Play old-rec", "bundle": "old-rec", "cmd": "node old.js",
            "cwd": sub, "kind": "launch", "env": {}, "hand": true, "visible": true, "window": true,
            "timeoutSec": 0, "parallel": 1, "definedBy": "coordinator", "definedAt": at, "rev": 1,
        });
    });
    let before = row(&rig, "old-rec");
    assert_eq!(
        keys(&before, &["launcherState", "cmd", "cwd"]),
        [json!("legacy"), json!("node old.js"), json!("sub")]
    );
    let moved = rig.ok(&["build", "set", "old-rec", "--title", "Old one"], As::Lead);
    let text = &moved.out;
    assert!(text.contains("old-rec: updated (title, launcher, cmd). Launcher old-rec made from its own launch command. Play ready (launcher-old-rec"), "{text}");
    assert!(
        text.contains("with a console window")
            && text.contains(". Old Play bundle-old-rec removed."),
        "{text}"
    );
    let board = rig.board();
    let rec = &board["builds"]["old-rec"];
    assert_eq!(rec["launcher"], "old-rec");
    assert!(
        ["cmd", "cwd", "window"]
            .iter()
            .all(|k| rec.get(*k).is_none())
    );
    let l = &board["launchers"]["old-rec"];
    assert_eq!(
        keys(l, &["cmd", "cwd", "window", "bundle"]),
        [
            json!("node old.js"),
            json!("sub"),
            json!(true),
            json!("old-rec")
        ]
    );
    assert!(board["runDefs"].get("bundle-old-rec").is_none());
    assert_eq!(board["runDefs"]["launcher-old-rec"]["cwd"], json!(sub));
    rig.edit(|b| {
        let at = b["createdAt"].clone();
        b["builds"]["master"] = json!({
            "tag": "master", "title": null, "note": null, "cmd": "node legacy-master.js", "cwd": null,
            "thread": null, "createdAt": at, "updatedAt": at, "createdBy": "coordinator",
        });
    });
    let kept = rig.ok(&["build", "set", "master", "--note", "n"], As::Lead);
    assert!(
        kept.out.contains("Its launch command stays on the bundle"),
        "{}",
        kept.out
    );
    let board = rig.board();
    assert_eq!(board["builds"]["master"]["cmd"], "node legacy-master.js");
    assert_eq!(
        board["launchers"]["master"]["cmd"], "game run",
        "the launcher was not overwritten"
    );
}

fn manual_tests(rig: &Rig, tag: &str) {
    rig.edit(|b| {
        let at = "2026-10-26T12:00:00.000Z";
        for (id, mode) in [("T-2", "manual"), ("T-3", "manual"), ("T-4", "auto")] {
            b["tests"][id] = json!({
                "id": id, "item": "I-1", "title": id, "mode": mode, "status": "pending", "build": tag,
                "thread": "worker", "rev": 1, "rank": 2048, "createdAt": at, "createdBy": "worker",
                "updatedAt": at, "updatedBy": "worker",
            });
        }
    });
}

#[test]
fn close_refuses_while_manual_tests_are_still_to_run_force_closes_anyway_reopen_undoes_it() {
    let rig = Rig::new();
    rig.with_master();
    rig.with_test();
    manual_tests(&rig, "c-1");
    rig.ok(
        &[
            "build",
            "add",
            "c-1",
            "--thread",
            "worker",
            "--cmd",
            "node c.js",
        ],
        As::Worker,
    );
    rig.refused(
        &["build", "close", "c-1"],
        As::Worker,
        "2 manual test(s) of c-1 are still to run: T-2, T-3",
    );
    assert!(
        rig.board()["builds"]["c-1"]
            .get("closedAt")
            .is_none_or(Value::is_null)
    );
    let forced = rig.ok(
        &[
            "build",
            "close",
            "c-1",
            "--force",
            "--note",
            "landed in 1.4",
        ],
        As::Worker,
    );
    assert!(forced.out.contains("c-1: closed"));
    let board = rig.board();
    let rec = &board["builds"]["c-1"];
    assert_eq!(rec["closedBy"], "worker");
    assert_eq!(rec["closeNote"], "landed in 1.4");
    assert!(rec["closedAt"].is_string());
    assert_eq!(board["launchers"]["c-1"]["cmd"], "node c.js");
    assert_eq!(rec["launcher"], "c-1");
    assert!(
        forced
            .out
            .contains("launcher c-1 made for it retired with it")
    );
    let l = &board["launchers"]["c-1"];
    assert_eq!(
        keys(l, &["retiredBy", "retireNote", "retiredByBundle"]),
        [json!("worker"), json!("bundle c-1 closed"), json!("c-1")]
    );
    assert!(board["runDefs"].get("launcher-c-1").is_none());
    let stamp = rec["closedAt"].clone();
    assert!(
        rig.ok(&["build", "close", "c-1"], As::Worker)
            .out
            .contains("already closed")
    );
    rig.ok(
        &["build", "set", "c-1", "--title", "Settings bundle"],
        As::Worker,
    );
    rig.ok(&["build", "add", "c-1", "--note", "n"], As::Worker);
    assert_eq!(rig.board()["builds"]["c-1"]["closedAt"], stamp);
    assert!(
        rig.ok(&["build", "list"], As::Nobody)
            .out
            .contains("closed")
    );
    let r = row(&rig, "c-1");
    assert_eq!(
        (r["closedAt"].clone(), r["closedBy"].clone()),
        (stamp, json!("worker"))
    );
    let re = rig.ok(&["build", "reopen", "c-1"], As::Worker);
    assert!(
        re.out.contains("reopened") && re.out.contains("launcher c-1 made for it is back"),
        "{}",
        re.out
    );
    let board = rig.board();
    assert_eq!(
        (
            board["builds"]["c-1"]["closedAt"].clone(),
            board["builds"]["c-1"]["closedBy"].clone()
        ),
        (Value::Null, Value::Null)
    );
    assert_eq!(board["launchers"]["c-1"]["retiredAt"], Value::Null);
    assert!(board["launchers"]["c-1"].get("retiredByBundle").is_none());
    assert!(
        board["runDefs"].get("launcher-c-1").is_some(),
        "its Play is back"
    );
    assert!(
        rig.ok(&["build", "reopen", "c-1"], As::Worker)
            .out
            .contains("is not closed")
    );
    rig.edit(|b| {
        for id in ["T-2", "T-3"] {
            b["tests"][id]["status"] = json!("pass");
        }
    });
    assert!(
        rig.ok(&["build", "close", "c-1"], As::Worker)
            .out
            .contains("closed")
    );
}

#[test]
fn closing_the_handed_bundle_unhands_it_hand_refuses_a_closed_tag_and_a_tag_with_no_record_can_be_closed()
 {
    let rig = Rig::new();
    rig.with_master();
    rig.ok(&["build", "add", "h-1", "--cmd", "node h.js"], As::Lead);
    rig.ok(&["build", "ready", "h-1"], As::Lead);
    rig.ok(&["build", "hand", "h-1"], As::Lead);
    assert_eq!(rig.board()["handedBuild"], "h-1");
    let closed = rig.ok(&["build", "close", "h-1"], As::Lead);
    assert!(closed.out.contains("closed and unhanded"), "{}", closed.out);
    let board = rig.board();
    assert_eq!(
        (board["handedBuild"].clone(), board["handedAt"].clone()),
        (Value::Null, Value::Null)
    );
    rig.refused(
        &["build", "hand", "h-1"],
        As::Lead,
        "is closed: coord build reopen h-1",
    );
    rig.ok(&["build", "reopen", "h-1"], As::Lead);
    rig.ok(&["build", "hand", "h-1"], As::Lead);
    assert_eq!(rig.board()["handedBuild"], "h-1");
    assert!(rig.board()["builds"].get("ghost-1").is_none());
    rig.fails(&["build", "close", "ghost-1"], As::Worker);
    rig.ok(&["build", "close", "ghost-1"], As::Lead);
    assert_eq!(rig.board()["builds"]["ghost-1"]["closedBy"], "coordinator");
}

#[test]
fn close_and_reopen_belong_to_the_coordinator_or_the_bundles_own_thread_nobody_else() {
    let rig = Rig::new();
    rig.with_master();
    rig.ok(&["build", "add", "auth-1", "--thread", "worker"], As::Lead);
    rig.refused(&["build", "close", "auth-1"], As::Other, "worker");
    assert!(
        rig.board()["builds"]["auth-1"]
            .get("closedAt")
            .is_none_or(Value::is_null)
    );
    rig.ok(&["build", "close", "auth-1"], As::Worker);
    rig.fails(&["build", "reopen", "auth-1"], As::Other);
    assert!(rig.board()["builds"]["auth-1"]["closedAt"].is_string());
    rig.ok(&["build", "reopen", "auth-1"], As::Lead);
    assert_eq!(rig.board()["builds"]["auth-1"]["closedAt"], Value::Null);
    rig.refused(
        &["build", "reopen", "nope-1"],
        As::Lead,
        "no bundle \"nope-1\"",
    );
    let log = rig.board()["log"].to_string();
    assert!(log.contains("closed bundle auth-1") && log.contains("reopened bundle auth-1"));
}

#[test]
fn closing_a_bundle_retires_the_launcher_made_for_it_only_a_shared_launcher_and_master_stay() {
    let rig = Rig::new();
    rig.with_master();
    rig.ok(
        &[
            "launcher",
            "set",
            "shared-1",
            "--cmd",
            "game run --shared",
            "--use-every",
            "30d",
        ],
        As::Lead,
    );
    rig.ok(
        &[
            "build",
            "add",
            "j-1",
            "--thread",
            "worker",
            "--cmd",
            "node j.js",
        ],
        As::Worker,
    );
    rig.ok(
        &[
            "build",
            "add",
            "j-2",
            "--thread",
            "worker",
            "--launcher",
            "shared-1",
        ],
        As::Worker,
    );
    let board = rig.board();
    assert_eq!(board["launchers"]["j-1"]["bundle"], "j-1");
    assert!(
        board["launchers"]["shared-1"].get("bundle").is_none(),
        "linked, not made for the bundle"
    );
    let closed = rig.ok(&["build", "close", "j-2"], As::Worker);
    assert!(!closed.out.contains("retired with it"));
    assert_eq!(
        rig.board()["launchers"]["shared-1"]["retiredAt"],
        Value::Null
    );
    let j1 = rig.ok(&["build", "close", "j-1"], As::Worker);
    assert!(j1.out.contains("launcher j-1 made for it retired with it"));
    let board = rig.board();
    assert_eq!(board["launchers"]["j-1"]["retireNote"], "bundle j-1 closed");
    assert!(board["runDefs"].get("launcher-j-1").is_none());
    assert_eq!(board["launchers"]["master"]["retiredAt"], Value::Null);
    let log = board["log"].to_string();
    assert!(
        log.contains("closed bundle j-1") && log.contains("retired launcher j-1"),
        "{log}"
    );
    rig.ok(&["build", "reopen", "j-1"], As::Worker);
    let board = rig.board();
    assert_eq!(board["launchers"]["j-1"]["retiredAt"], Value::Null);
    assert!(board["runDefs"].get("launcher-j-1").is_some());
    rig.ok(
        &["launcher", "retire", "j-1", "--note", "replaced"],
        As::Worker,
    );
    rig.ok(&["build", "close", "j-1"], As::Worker);
    rig.ok(&["build", "reopen", "j-1"], As::Worker);
    let l = &rig.board()["launchers"]["j-1"];
    assert_eq!(l["retireNote"], "replaced");
    assert!(l["retiredAt"].is_string());
    rig.ok(&["build", "set", "j-1", "--cmd", "node j.js"], As::Worker);
    rig.ok(
        &[
            "build",
            "add",
            "j-3",
            "--thread",
            "worker",
            "--launcher",
            "j-1",
        ],
        As::Worker,
    );
    rig.ok(&["build", "close", "j-1"], As::Worker);
    assert_eq!(
        rig.board()["launchers"]["j-1"]["retiredAt"],
        Value::Null,
        "j-3 still links it"
    );
    rig.ok(&["build", "close", "j-3"], As::Worker);
    rig.ok(&["build", "reopen", "j-1"], As::Worker);
}

#[test]
fn rm_retires_every_launcher_made_for_the_bundle_once_and_for_good() {
    let rig = Rig::new();
    rig.with_master();
    rig.ok(
        &[
            "build",
            "add",
            "r-1",
            "--thread",
            "worker",
            "--cmd",
            "node r.js",
        ],
        As::Worker,
    );
    let gone = rig.ok(&["build", "rm", "r-1"], As::Worker);
    assert!(gone.out.contains("launcher r-1 retired"), "{}", gone.out);
    let l = &rig.board()["launchers"]["r-1"];
    assert!(l["retiredAt"].is_string());
    assert!(l.get("retiredByBundle").is_none());
}

#[test]
fn a_launcher_made_for_a_bundle_takes_no_use_every_and_master_takes_none_either() {
    let rig = Rig::new();
    rig.with_master();
    rig.ok(
        &[
            "build",
            "add",
            "j-1",
            "--thread",
            "worker",
            "--cmd",
            "node j.js",
        ],
        As::Worker,
    );
    let set = rig.ok(&["launcher", "set", "j-1", "--use-every", "3d"], As::Worker);
    assert!(
        set.out
            .contains("--use-every ignored: j-1 is joined to a bundle"),
        "{}",
        set.out
    );
    assert!(rig.board()["launchers"]["j-1"].get("useEvery").is_none());
    let master = rig.ok(
        &["launcher", "set", "master", "--use-every", "3d"],
        As::Lead,
    );
    assert!(
        master
            .out
            .contains("--use-every ignored: master has no usage window")
    );
    assert!(rig.board()["launchers"]["master"].get("useEvery").is_none());
}

#[test]
fn master_is_mandatory_build_hand_is_refused_without_a_live_master_and_says_how_to_set_it() {
    let rig = Rig::new();
    let refused = rig.fails(&["build", "hand", "x-1"], As::Lead);
    assert!(
        refused
            .err
            .contains("no master launcher, so nothing can be handed to the owner yet"),
        "{}",
        refused.err
    );
    assert!(
        refused
            .err
            .contains("Set master first: coord launcher set master --cmd")
    );
    let board = rig.board();
    assert!(board["handedBuild"].is_null());
    assert!(board["builds"].get("x-1").is_none(), "nothing is written");
    let tip = rig.ok(&["launcher", "list"], As::Lead);
    assert!(
        tip.out.contains("no launcher yet") && tip.out.contains("a master launcher is required"),
        "{}",
        tip.out
    );
    rig.with_master();
    rig.ok(&["build", "hand", "x-1"], As::Lead);
    assert_eq!(rig.board()["handedBuild"], "x-1");
}

#[test]
fn a_new_bundle_is_a_draft_that_cannot_be_handed_ready_allows_hand_draft_goes_back_unless_handed() {
    let rig = Rig::new();
    rig.with_master();
    rig.with_test();
    let status = |tag: &str| row(&rig, tag)["status"].clone();
    let added = rig.ok(
        &[
            "build",
            "add",
            "s-1",
            "--cmd",
            "node s.js",
            "--thread",
            "worker",
        ],
        As::Lead,
    );
    assert!(
        added.out.contains("It is a draft") && added.out.contains("coord build ready s-1"),
        "{}",
        added.out
    );
    assert_eq!(rig.board()["builds"]["s-1"]["status"], "draft");
    assert_eq!(status("s-1"), "draft");
    assert!(
        rig.ok(&["build", "list"], As::Nobody)
            .out
            .lines()
            .any(|l| l.starts_with("s-1") && l.contains("draft"))
    );
    let was = rig.board()["handedBuild"].clone();
    rig.refused(
        &["build", "hand", "s-1"],
        As::Lead,
        "bundle s-1 is a draft (not built yet)",
    );
    assert_eq!(rig.board()["handedBuild"], was, "nothing is handed");
    rig.fails(&["build", "ready", "s-1"], As::Other);
    rig.refused(
        &["build", "ready", "nope-1"],
        As::Lead,
        "no bundle \"nope-1\"",
    );
    let ready = rig.ok(&["build", "ready", "s-1"], As::Worker);
    assert!(
        ready.out.contains("s-1: ready.") && ready.out.contains("Play is shown"),
        "{}",
        ready.out
    );
    assert_eq!(rig.board()["builds"]["s-1"]["status"], "ready");
    assert_eq!(
        rig.ok(&["build", "ready", "s-1"], As::Worker).out.trim(),
        "s-1: already ready."
    );
    rig.ok(&["build", "hand", "s-1"], As::Lead);
    assert_eq!(rig.board()["handedBuild"], "s-1");
    rig.refused(
        &["build", "draft", "s-1"],
        As::Lead,
        "handed build: coord build unhand first",
    );
    assert_eq!(rig.board()["builds"]["s-1"]["status"], "ready");
    rig.ok(&["build", "unhand"], As::Lead);
    assert!(
        rig.ok(&["build", "draft", "s-1"], As::Lead)
            .out
            .contains("s-1: draft")
    );
    assert_eq!(status("s-1"), "draft");
    rig.fails(&["build", "hand", "s-1"], As::Lead);
    rig.ok(&["build", "ready", "s-1"], As::Lead);
    rig.ok(&["build", "close", "s-1", "--force"], As::Lead);
    assert_eq!(status("s-1"), "closed");
    rig.refused(
        &["build", "draft", "s-1"],
        As::Lead,
        "is closed: coord build reopen s-1",
    );
    rig.ok(&["build", "reopen", "s-1"], As::Lead);
    assert_eq!(
        status("s-1"),
        "ready",
        "the status survives a close and a reopen"
    );
}

#[test]
fn a_record_without_a_status_reads_ready_when_it_was_ever_handed_or_starts_something_else_draft() {
    let rig = Rig::new();
    rig.with_master();
    rig.edit(|b| {
        let at = argus_core::clock::now_iso();
        let old = |tag: &str, extra: Value| {
            let mut rec = json!({
                "tag": tag, "title": null, "note": null, "launcher": null, "thread": null,
                "createdAt": at, "updatedAt": at, "createdBy": "coordinator",
            });
            for (k, v) in extra.as_object().unwrap() {
                rec[k] = v.clone();
            }
            rec
        };
        b["builds"]["old-launcher"] = old("old-launcher", json!({ "launcher": "master" }));
        b["builds"]["old-cmd"] = old("old-cmd", json!({ "cmd": "node old.js" }));
        b["builds"]["old-handed"] = old("old-handed", json!({ "lastHandedAt": at }));
        b["builds"]["old-bare"] = old("old-bare", json!({}));
    });
    let statuses: Vec<Value> = ["old-launcher", "old-cmd", "old-handed", "old-bare"]
        .iter()
        .map(|t| row(&rig, t)["status"].clone())
        .collect();
    assert_eq!(
        statuses,
        [
            json!("ready"),
            json!("ready"),
            json!("ready"),
            json!("draft")
        ]
    );
    assert!(
        rig.board()["builds"]["old-bare"].get("status").is_none(),
        "nothing was written to the record"
    );
    rig.refused(&["build", "hand", "old-bare"], As::Lead, "is a draft");
    rig.ok(&["build", "hand", "old-launcher"], As::Lead);
    rig.ok(&["build", "unhand"], As::Lead);
}
