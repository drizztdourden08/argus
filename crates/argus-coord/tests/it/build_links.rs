//! I-78 P12: builds serve items from the CLI (`--for`, `--master`, `coord launcher link`), the
//! item verbs say which builds closed or came back with an item, and `coord build migrate-links`
//! gives the builds of an older board their place.

// test-tag: critical

use crate::agent_common::{As, Rig};
use serde_json::{Value, json};

fn launcher(rig: &Rig, name: &str) -> Value {
    rig.board()["launchers"][name].clone()
}

/// A test plan `id` that names the bundle `tag`.
fn plan(rig: &Rig, id: &str, tag: &str) {
    rig.edit(|b| {
        let at = "2026-10-26T12:00:00.000Z";
        b["items"][id] = json!({
            "id": id, "type": "test-plan", "title": "Plan", "thread": "worker", "status": "todo",
            "rev": 1, "rank": 2048, "fields": { "bundle": tag },
            "createdAt": at, "createdBy": "worker", "updatedAt": at, "updatedBy": "worker",
        });
    });
}

#[test]
fn a_new_build_needs_the_item_it_serves_and_the_item_must_exist() {
    let rig = Rig::new();
    rig.with_item();
    let bare = rig.refused(
        &["launcher", "set", "web-1", "--cmd", "node web.js"],
        As::Worker,
        "--for",
    );
    assert!(bare.err.contains("--master"), "{}", bare.err);
    assert!(rig.board()["launchers"].get("web-1").is_none());
    rig.refused(
        &[
            "launcher",
            "set",
            "web-1",
            "--cmd",
            "node web.js",
            "--for",
            "I-9",
        ],
        As::Worker,
        "I-9",
    );
    let made = rig.ok(
        &[
            "launcher",
            "set",
            "web-1",
            "--cmd",
            "node web.js",
            "--for",
            "I-1",
        ],
        As::Worker,
    );
    assert!(made.out.contains("serves I-1"), "{}", made.out);
    assert_eq!(launcher(&rig, "web-1")["for"], "I-1");
    assert_eq!(launcher(&rig, "web-1").get("useEvery"), None);
}

#[test]
fn master_hands_the_role_over_and_only_the_coordinator_may() {
    let rig = Rig::new();
    rig.with_item();
    rig.legacy_launcher("master", None);
    rig.ok(
        &[
            "launcher",
            "set",
            "web-1",
            "--cmd",
            "node web.js",
            "--for",
            "I-1",
        ],
        As::Worker,
    );
    rig.refused(
        &["launcher", "set", "web-1", "--master"],
        As::Worker,
        "coordinator",
    );
    let handed = rig.ok(&["launcher", "set", "web-1", "--master"], As::Lead);
    assert!(handed.out.contains("master"), "{}", handed.out);
    assert_eq!(launcher(&rig, "web-1")["master"], true);
    assert_ne!(launcher(&rig, "master")["master"], true);
    assert_eq!(launcher(&rig, "master")["relink"], true);
    rig.refused(&["launcher", "retire", "web-1"], As::Lead, "never retired");
}

#[test]
fn launcher_link_moves_a_build_to_another_open_item_and_refuses_a_closed_or_missing_one() {
    let rig = Rig::new();
    rig.with_item();
    rig.with_item_status("I-2", "todo");
    rig.with_item_status("I-3", "done");
    rig.ok(
        &[
            "launcher",
            "set",
            "web-1",
            "--cmd",
            "node web.js",
            "--for",
            "I-1",
        ],
        As::Worker,
    );
    let same = rig.ok(&["launcher", "link", "web-1", "I-1"], As::Worker);
    assert!(same.out.contains("already serves I-1"), "{}", same.out);
    let moved = rig.ok(&["launcher", "link", "web-1", "I-2"], As::Worker);
    assert!(moved.out.contains("web-1: serves I-2 now"), "{}", moved.out);
    assert_eq!(launcher(&rig, "web-1")["for"], "I-2");
    rig.refused(&["launcher", "link", "web-1", "I-3"], As::Worker, "I-3");
    rig.refused(&["launcher", "link", "web-1", "I-9"], As::Worker, "I-9");
    rig.refused(
        &["launcher", "link", "web-1"],
        As::Worker,
        "coord launcher link <name> I-n",
    );
    rig.refused(
        &["launcher", "link", "nope-1", "I-2"],
        As::Worker,
        "no launcher \"nope-1\"",
    );
}

#[test]
fn an_item_that_closes_names_the_builds_that_closed_with_it_and_a_reopen_names_them_again() {
    let rig = Rig::new();
    rig.with_item();
    rig.ok(
        &[
            "launcher",
            "set",
            "web-1",
            "--cmd",
            "node web.js",
            "--for",
            "I-1",
        ],
        As::Worker,
    );
    let done = rig.ok(&["item", "status", "I-1", "done"], As::Worker);
    assert!(
        done.out.contains("I-1: closed with it: build web-1."),
        "{}",
        done.out
    );
    assert!(launcher(&rig, "web-1")["retiredAt"].is_string());
    assert_eq!(launcher(&rig, "web-1")["retiredByItem"], "I-1");
    let back = rig.ok(&["item", "status", "I-1", "todo"], As::Worker);
    assert!(
        back.out.contains("I-1: reopened with it: build web-1."),
        "{}",
        back.out
    );
    assert_eq!(launcher(&rig, "web-1")["retiredAt"], Value::Null);
}

/// A board written before builds served items: a master, a bundle named by one test plan, a
/// bundle whose tests verify one item, a build nothing points at and a retired one.
fn old_board() -> Rig {
    let rig = Rig::new();
    rig.with_test();
    plan(&rig, "I-2", "alpha-1");
    rig.legacy_launcher("master", None);
    rig.legacy_launcher("alpha-1", None);
    rig.legacy_launcher("bundle-4", None);
    rig.legacy_launcher("tool-1", Some("7d"));
    rig.legacy_launcher("old-1", Some("7d"));
    rig.edit(|b| {
        b["launchers"]["alpha-1"]["bundle"] = json!("alpha-1");
        b["launchers"]["bundle-4"]["bundle"] = json!("bundle-4");
        b["launchers"]["old-1"]["retiredAt"] = json!("2026-10-20T12:00:00.000Z");
        b["builds"]["alpha-1"] = json!({ "tag": "alpha-1", "launcher": "alpha-1" });
        b["builds"]["bundle-4"] = json!({ "tag": "bundle-4", "launcher": "bundle-4" });
    });
    rig
}

#[test]
fn migrate_links_dry_run_says_what_it_would_do_and_writes_nothing() {
    let rig = old_board();
    let before = rig.board();
    let ran = rig.ok(&["build", "migrate-links", "--dry-run"], As::Lead);
    for line in [
        "[dry run] master: the master build (flagged master).",
        "[dry run] alpha-1: serves I-2 (the one test plan that names bundle alpha-1).",
        "[dry run] bundle-4: serves I-1 (every test of bundle bundle-4 verifies it).",
        "[dry run] tool-1: no clear item, needs a link: coord launcher link tool-1 I-n.",
        "[dry run] old-1: retired, no clear item, left as it is.",
        "dry run: 1 master, 2 linked, 1 need a link, 1 retired left unlinked; nothing written.",
    ] {
        assert!(ran.out.contains(line), "{line}\n{}", ran.out);
    }
    assert_eq!(rig.board()["launchers"], before["launchers"]);
}

#[test]
fn migrate_links_writes_the_flags_and_links_and_a_second_run_changes_nothing() {
    let rig = old_board();
    rig.refused(
        &["build", "migrate-links"],
        As::Worker,
        "belongs to the coordinator",
    );
    let ran = rig.ok(&["build", "migrate-links"], As::Lead);
    assert!(
        ran.out
            .contains("build links migrated: 1 master, 2 linked, 1 need a link"),
        "{}",
        ran.out
    );
    assert_eq!(launcher(&rig, "master")["master"], true);
    assert_eq!(launcher(&rig, "alpha-1")["for"], "I-2");
    assert_eq!(launcher(&rig, "bundle-4")["for"], "I-1");
    assert_eq!(launcher(&rig, "tool-1")["relink"], true);
    assert_eq!(launcher(&rig, "tool-1").get("for"), None);
    assert_eq!(launcher(&rig, "old-1").get("for"), None);
    assert_ne!(launcher(&rig, "old-1")["relink"], true);
    let once = rig.board()["launchers"].clone();
    let again = rig.ok(&["build", "migrate-links"], As::Lead);
    assert!(
        again.out.contains("tool-1: no clear item, needs a link"),
        "{}",
        again.out
    );
    assert_eq!(rig.board()["launchers"], once);
}

#[test]
fn a_linked_build_leaves_the_relink_list_and_a_board_with_nothing_to_migrate_says_so() {
    let rig = old_board();
    rig.ok(&["build", "migrate-links"], As::Lead);
    rig.ok(&["launcher", "link", "tool-1", "I-1"], As::Lead);
    assert_ne!(launcher(&rig, "tool-1")["relink"], true);
    let quiet = Rig::new();
    let none = quiet.ok(&["build", "migrate-links"], As::Lead);
    assert!(
        none.out
            .contains("no builds to link: every build is linked."),
        "{}",
        none.out
    );
}
