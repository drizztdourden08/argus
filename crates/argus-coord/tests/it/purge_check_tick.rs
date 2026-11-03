//! K3: purge, restore, trash, check and tick.

// test-tag: critical

use std::fs;

use crate::common::coord;
use crate::thread_support::Cli;
use serde_json::{Value, json};

const OLD: &str = "2020-01-01T00:00:00.000Z";

fn fixture() -> Cli {
    let fx = Cli::new("work", &["--project", "alpha", "--as", "lead"]);
    fx.home.write_board("alpha", &board());
    fx
}

fn thread(id: &str, role: &str, status: &str) -> Value {
    json!({"id": id, "name": id, "role": role, "status": status, "flags": {"needsHuman": false, "blocked": false},
           "note": null, "subject": "s", "brief": null, "registeredAt": OLD, "createdBy": id, "updatedAt": OLD,
           "updatedBy": id, "rev": 1, "rank": 1024, "root": null, "session": null, "lastSeen": OLD})
}

fn board() -> Value {
    json!({
        "schema": 2, "project": "alpha", "createdAt": OLD, "updatedAt": OLD,
        "counters": {"I": 2, "Q": 1},
        "threads": {
            "lead": thread("lead", "coordinator", "active"),
            "gone": thread("gone", "thread", "done"),
            "busy": thread("busy", "thread", "blocked"),
        },
        "items": {
            "I-1": {"id": "I-1", "thread": "gone", "title": "old", "status": "done", "ac": [], "rev": 1, "rank": 1024},
            "I-2": {"id": "I-2", "thread": "busy", "title": "live", "status": "doing", "ac": [], "rev": 1, "rank": 2048},
        },
        "tests": {}, "entries": {
            "Q-1": {"id": "Q-1", "kind": "question", "status": "open", "thread": "busy", "text": "which way", "createdAt": OLD, "rev": 1, "rank": 1024},
        },
        "locks": {}, "actions": {}, "agents": {}, "log": {},
        "inbox": {"busy": [{"id": "m-1-a", "ts": OLD, "from": "lead", "kind": "note", "text": "hello", "deliveredAt": null}]},
        "runDefs": {}, "builds": {}, "launchers": {}, "dirty": {}, "versions": {},
    })
}

#[test]
fn a_purge_is_a_dry_run_until_confirmed_and_says_the_exact_command() {
    let fx = fixture();
    let dry = fx.ok(&["purge", "--status", "done"]);
    assert!(
        dry.out.starts_with(
            "dry run, nothing written. coord purge --status done would remove:\n  threads"
        ),
        "{}",
        dry.out
    );
    assert!(
        dry.out.contains("A restorable backup is written to "),
        "{}",
        dry.out
    );
    assert!(
        dry.out
            .ends_with("  coord purge --status done --confirm alpha\n"),
        "{}",
        dry.out
    );
    assert!(fx.board()["threads"]["gone"].is_object());
    let words = [
        "purge",
        "--status",
        "done",
        "--reason",
        "tidy up",
        "--confirm",
        "alpha",
    ];
    let done = fx.ok(&words);
    assert!(
        done.out.starts_with("purged 1 threads, 1 items."),
        "{}",
        done.out
    );
    assert!(done.out.contains("\nbackup: "), "{}", done.out);
    assert!(
        done.out.contains("\nrestore with: coord restore --from "),
        "{}",
        done.out
    );
    let board = fx.board();
    assert!(board["threads"]["gone"].is_null());
    assert!(board["items"]["I-1"].is_null());
    assert_eq!(board["counters"]["I"], 2, "ids are never reused");
    let backups: Vec<_> = fs::read_dir(fx.home.coord_home().join("alpha").join("purged"))
        .unwrap()
        .collect();
    assert_eq!(backups.len(), 1);
    assert_eq!(
        fx.ok(&["purge", "--status", "done", "--confirm", "alpha"])
            .out,
        "nothing matches: nothing was purged.\n"
    );
}

#[test]
fn purge_refuses_what_it_cannot_do() {
    let fx = fixture();
    fx.refused(&["purge"], "say what to purge, exactly one of:");
    fx.refused(&["purge", "thread"], "name at least one thread.");
    fx.refused(
        &["purge", "thread", "ghost"],
        "no thread \"ghost\" on this board.",
    );
    fx.refused(
        &["purge", "thread", "lead"],
        "lead cannot be purged: the coordinator and the calling thread are never purged.",
    );
    fx.refused(
        &["purge", "--status", "bogus"],
        "--status takes a comma list of done, archived, placeholder, dispatched.",
    );
    fx.refused(
        &["purge", "--idle", "0"],
        "--idle takes a number of days, e.g. --idle 30.",
    );
    fx.refused(
        &["purge", "--trash", "--older", "x"],
        "--older takes a number of days",
    );
    fx.refused(
        &["purge", "thread", "gone", "--confirm", "wrong"],
        "--confirm must be the project key \"alpha\", not \"wrong\". Nothing was changed.",
    );
    let worker = coord(
        &["purge", "--trash", "--project", "alpha", "--as", "busy"],
        fx.home.env(),
        &fx.home.folder("work"),
    );
    assert_eq!(worker.code, 1);
    assert!(
        worker
            .err
            .contains("only the coordinator can purge. Ask with coord inbox send coordinator"),
        "{}",
        worker.err
    );
    assert_eq!(
        fx.ok(&["purge", "--idle", "99999"]).out,
        "nothing matches: there is nothing to purge.\n"
    );
}

#[test]
fn restore_puts_a_backup_back_and_leaves_what_is_there() {
    let fx = fixture();
    let done = fx.ok(&["purge", "thread", "busy", "--confirm", "alpha"]);
    let file = done
        .out
        .lines()
        .find_map(|l| l.strip_prefix("backup: "))
        .unwrap()
        .to_owned();
    assert!(fx.board()["threads"]["busy"].is_null());
    fx.refused(
        &["restore"],
        "pass --from <backup.json>, the file coord purge printed.",
    );
    fx.refused(
        &["restore", "--from", &file],
        "restore needs --confirm alpha (the project key).",
    );
    fx.refused(
        &[
            "restore",
            "--from",
            &file,
            "--confirm",
            "alpha",
            "--thread",
            "ghost",
        ],
        "the backup holds no thread \"ghost\".",
    );
    let back = fx.ok(&["restore", "--from", &file, "--confirm", "alpha"]);
    assert!(
        back.out
            .starts_with("restored 1 threads, 1 items, 1 entries, "),
        "{}",
        back.out
    );
    assert!(
        back.out
            .ends_with("with their original ids. Records already on the board were left alone.\n"),
        "{}",
        back.out
    );
    let board = fx.board();
    assert_eq!(board["threads"]["busy"]["status"], "blocked");
    assert_eq!(board["entries"]["Q-1"]["status"], "open");
    assert_eq!(board["inbox"]["busy"].as_array().unwrap().len(), 1);
    let again = fx.ok(&["restore", "--from", &file, "--confirm", "alpha"]);
    assert_eq!(
        again.out,
        "restored 0, skipped 3 already present: every record in the file is already on the board.\n"
    );
}

#[test]
fn trash_lists_soft_deleted_records_newest_first() {
    let fx = fixture();
    assert_eq!(fx.ok(&["trash"]).out, "the trash is empty.\n");
    let mut board = board();
    board["items"]["I-1"]["deletedAt"] = json!("2026-10-01T00:00:00.000Z");
    board["items"]["I-1"]["deletedBy"] = json!("lead");
    board["items"]["I-1"]["deleteReason"] = json!("duplicate");
    board["entries"]["Q-1"]["deletedAt"] = json!("2026-10-02T00:00:00.000Z");
    fx.home.write_board("alpha", &board);
    let text = fx.ok(&["trash"]).out;
    let lines: Vec<&str> = text.lines().collect();
    assert!(lines[0].starts_with("id"), "{text}");
    assert!(lines[1].starts_with("Q-1  entries"), "{text}");
    assert!(
        lines[2].contains("I-1  items") && lines[2].contains("duplicate"),
        "{text}"
    );
    assert!(
        text.ends_with("Remove them for good: coord purge --trash [--older <days>].\n"),
        "{text}"
    );
    let json: Value = serde_json::from_str(&fx.ok(&["trash", "--json"]).out).unwrap();
    assert_eq!(json[0]["collection"], "entries");
    assert_eq!(json[0]["reason"], Value::Null);
    assert_eq!(json[1]["reason"], "duplicate");
    let mine = fx.ok(&["trash", "--for", "gone"]).out;
    assert!(mine.contains("I-1") && !mine.contains("Q-1"), "{mine}");
}

#[test]
fn check_reports_what_needs_a_nudge() {
    let fx = fixture();
    let text = fx.ok(&["check"]).out;
    assert!(text.starts_with("# check alpha  (dashboard http://127.0.0.1:47615, coord board open)\nstale > 30m: \n  - lead (active, seen "), "{text}");
    assert!(
        text.contains("undelivered inbox: \n  - busy x1 (session \"?\"): hello\n"),
        "{text}"
    );
    assert!(text.contains("blocked: \n  - busy: \n"), "{text}");
    assert!(text.contains("open questions: \n  - Q-1 busy "), "{text}");
    assert!(text.contains("decision requests: none   open messages: none\ntests failing: none   manual tests to run: none\n"), "{text}");
    assert!(
        text.contains("manual tests to run, by build (handed to the owner: none): none\n"),
        "{text}"
    );
    let json: Value = serde_json::from_str(&fx.ok(&["check", "--json"]).out).unwrap();
    assert_eq!(json["project"], "alpha");
    assert_eq!(json["artifact"], Value::Null);
    assert_eq!(json["toPush"], 0);
    assert_eq!(json["undelivered"][0]["latest"], "hello");
    let keys: Vec<&str> = json
        .as_object()
        .unwrap()
        .keys()
        .map(String::as_str)
        .collect();
    assert_eq!(keys[..4], ["project", "artifact", "toPush", "stale"]);
    assert_eq!(keys.last(), Some(&"agents"));
    let none = fx.ok(&["check", "--stale", "99999999", "--json"]).out;
    assert_eq!(
        serde_json::from_str::<Value>(&none).unwrap()["stale"],
        json!([])
    );
}

#[test]
fn tick_lists_who_to_wake_and_woke_records_it() {
    let fx = fixture();
    let tick: Value = serde_json::from_str(&fx.ok(&["tick"]).out).unwrap();
    assert_eq!(
        tick["wake"],
        json!([{"thread": "busy", "session": null, "count": 1}])
    );
    let notes: Vec<&str> = tick["attention"]
        .as_array()
        .unwrap()
        .iter()
        .map(|n| n.as_str().unwrap())
        .collect();
    assert!(notes.contains(&"busy is blocked: "), "{notes:?}");
    assert!(
        notes
            .iter()
            .any(|n| n.starts_with("lead silent for ") && n.ends_with('h')),
        "{notes:?}"
    );
    assert!(
        notes.contains(&"busy has no session title, it cannot be woken"),
        "{notes:?}"
    );
    assert_eq!(
        fx.ok(&["tick", "woke", "busy", "nobody"]).out,
        "woke: busy, nobody\n"
    );
    assert!(fx.board()["inbox"]["busy"][0]["wokeAt"].is_string());
    assert_eq!(fx.ok(&["tick", "woke"]).out, "woke: nobody\n");
    let after: Value = serde_json::from_str(&fx.ok(&["tick"]).out).unwrap();
    assert_eq!(after["wake"], json!([]));
    let with_dir = fx.run(&["tick", "--dir", "x"]);
    assert_eq!(
        with_dir.err,
        "coord: --dir is ignored; the dashboard syncs the board itself.\n"
    );
}

#[test]
fn sync_import_replays_saved_entries_and_the_other_verbs_are_retired() {
    let fx = fixture();
    let dir = fx.home.folder("work").join("saved").join("entries");
    fs::create_dir_all(&dir).unwrap();
    let entry = json!({"id": "O-1", "kind": "followup", "replyTo": "Q-1", "text": "go left", "author": "human", "createdAt": "2026-10-02T00:00:00.000Z"});
    fs::write(dir.join("a.json"), entry.to_string()).unwrap();
    fs::write(dir.join("notes.txt"), "not json").unwrap();
    let out: Value = serde_json::from_str(&fx.ok(&["sync", "import", "saved"]).out).unwrap();
    assert_eq!(out["imported"], 1);
    assert_eq!(
        out["ops"],
        json!({"applied": 0, "conflict": 0, "refused": 0})
    );
    assert_eq!(out["notes"], json!([]));
    assert_eq!(out["deliveries"]["busy"].as_array().unwrap().len(), 1);
    assert_eq!(fx.board()["entries"]["Q-1"]["status"], "answered");
    let again: Value = serde_json::from_str(&fx.ok(&["sync", "import", "saved"]).out).unwrap();
    assert_eq!(again["imported"], 0);
    let none: Value = serde_json::from_str(&fx.ok(&["sync", "import", "nowhere"]).out).unwrap();
    assert_eq!(none["imported"], 0);
    assert_eq!(
        fx.ok(&["sync", "export"]).out,
        "retired: coord sync export is gone; the dashboard mirrors the board live (coord board open). Only coord sync import <dir> remains, to replay saved entries.\n"
    );
    fx.refused(
        &["sync", "frobnicate"],
        "sync verb must be: import <dir> (replays <dir>/entries/*.json onto the board).",
    );
}
