//! K2: the lines and records of coord worktree.

// test-tag: critical

use crate::common::{Home, Ran, coord};
use crate::remote::{ok, read_board_at, refused};
use argus_worktrees::worktree_id;
use serde_json::{Value, json};

const FOO: &str = "X:/Proj/.claude/worktrees/foo";

/// A home with a board `alpha` holding a coordinator and a worker, both rooted in the home.
struct World {
    home: Home,
}

impl World {
    fn new() -> World {
        let home = Home::new();
        let root = home.path().to_path_buf();
        let threads = [
            (
                "coordinator",
                "coordinator",
                "active",
                None,
                Some(root.as_path()),
            ),
            ("worker", "thread", "active", None, Some(root.as_path())),
        ];
        home.write_board("alpha", &crate::common::board_with("alpha", &threads));
        World { home }
    }

    /// `coord worktree <args> --project alpha --as <who>`.
    fn run(&self, words: &[&str], who: &str) -> Ran {
        let mut argv = vec!["worktree"];
        argv.extend(words);
        argv.extend(["--project", "alpha", "--as", who]);
        coord(&argv, self.home.env(), self.home.path())
    }

    fn worker(&self, args: &[&str]) -> Ran {
        self.run(args, "worker")
    }

    fn board(&self) -> Value {
        read_board_at(&self.home.coord_home(), "alpha")
    }

    fn rules(&self) -> Vec<Value> {
        self.board()["worktreeRules"]
            .as_array()
            .cloned()
            .unwrap_or_default()
    }

    fn refuse_all(&self, cases: &[(&[&str], &str)]) {
        for (args, text) in cases {
            refused(&self.worker(args), 1, text);
        }
    }
}

fn say(ran: &Ran) -> String {
    ok(ran).to_owned()
}

#[test]
fn rule_add_makes_a_rule_with_its_defaults_and_audits_it() {
    let w = World::new();
    let args = [
        "rule",
        "add",
        "X:\\Proj\\.claude\\worktrees\\",
        "--note",
        "the lead builds here",
    ];
    let out = say(&w.worker(&args));
    assert!(out.contains("W-1: rule added (folder X:/Proj/.claude/worktrees, tells worker, stale after 3 days, the lead builds here)."), "{out}");
    assert!(
        out.contains("warning: X:/Proj/.claude/worktrees does not exist on this machine"),
        "{out}"
    );
    let rule = &w.rules()[0];
    let seen = json!([
        rule["id"],
        rule["path"],
        rule["kind"],
        rule["thread"],
        rule["note"],
        rule["staleDays"],
        rule["addedBy"]
    ]);
    let want = json!([
        "W-1",
        "X:/Proj/.claude/worktrees",
        "folder",
        "worker",
        "the lead builds here",
        null,
        "worker"
    ]);
    assert_eq!(seen, want);
    assert!(rule["addedAt"].is_string());
    let audit = w.board()["log"]["audit"]["entries"].to_string();
    assert!(audit.contains("added worktree rule W-1"), "{audit}");
}

#[test]
fn the_same_path_and_kind_again_updates_the_rule_and_a_repo_rule_is_another_one() {
    let w = World::new();
    w.worker(&["rule", "add", "X:/Proj/.claude/worktrees"]);
    let again = [
        "rule",
        "add",
        "x:/proj/.claude/worktrees",
        "--stale-days",
        "5",
        "--thread",
        "coordinator",
    ];
    let updated = say(&w.worker(&again));
    let words = if cfg!(windows) {
        "W-1: rule updated (thread, staleDays)"
    } else {
        "W-2: rule added"
    };
    assert!(updated.contains(words), "{updated}");
    if cfg!(windows) {
        assert_eq!(w.rules().len(), 1);
        assert_eq!(
            (
                w.rules()[0]["staleDays"].clone(),
                w.rules()[0]["thread"].clone()
            ),
            (json!(5), json!("coordinator"))
        );
        let same = say(&w.worker(&["rule", "add", "x:/proj/.claude/worktrees"]));
        assert!(same.contains("W-1: unchanged"), "{same}");
    }
    let repo = [
        "rule",
        "add",
        "X:/Proj",
        "--kind",
        "repo",
        "--note",
        "the repo itself",
    ];
    ok(&w.run(&repo, "coordinator"));
    let last = w.rules().last().cloned().expect("a rule");
    assert_eq!(
        (last["kind"].clone(), last["thread"].clone()),
        (json!("repo"), json!("coordinator"))
    );
}

#[test]
fn rule_add_refuses_a_bad_path_kind_thread_days_and_note() {
    let w = World::new();
    let name_of_100 = "x".repeat(101);
    let distinct: Vec<String> = (0..9)
        .flat_map(|i| ["--exclude".to_owned(), format!("n{i}")])
        .collect();
    let mut nine = vec!["rule", "add", "X:/a"];
    nine.extend(distinct.iter().map(String::as_str));
    let long_path = format!("X:/{}", "a".repeat(300));
    let long_note = "n".repeat(301);
    w.refuse_all(&[
        (&["rule", "add"], "give the PARENT FOLDER"),
        (&["rule", "add", "X:/relic-of-the-past/.claude/worktrees/web-*"], "a rule path is the parent folder (no wildcards): use --include/--exclude to choose names"),
        (&["rule", "add", "X:/*/worktrees"], "no wildcards"),
        (&["rule", "add", "X:/a/b?"], "no wildcards"),
        (&["rule", "add", "X:/a/[x]"], "no wildcards"),
        (&["rule", "add", "X:/a/{b,c}"], "no wildcards"),
        (&["rule", "add", "X:/a", "--include", "a/b"], "holds a slash"),
        (&["rule", "add", "X:/a", "--include", &name_of_100], "limit is 100"),
        (&nine, "8 times at most"),
        (&["rule", "add", "X:/a", "--kind", "repo", "--include", "web-*"], "choose folder names inside a folder rule"),
        (&["rule", "add", "relative/dir"], "not an absolute path"),
        (&["rule", "add", "X:/a/../b"], ".. segments"),
        (&["rule", "add", "~/work"], "~ is not expanded"),
        (&["rule", "add", &long_path], "limit is 300"),
        (&["rule", "add", "X:/a", "--kind", "tree"], "--kind must be one of folder, repo"),
        (&["rule", "add", "X:/a", "--thread", "nobody"], "no thread \"nobody\""),
        (&["rule", "add", "X:/a", "--stale-days", "0"], "1 to 365"),
        (&["rule", "add", "X:/a", "--stale-days", "400"], "1 to 365"),
        (&["rule", "add", "X:/a", "--note", &long_note], "limit is 300"),
    ]);
    refused(
        &w.run(&["rule", "add", "X:/a"], "ghost"),
        1,
        "no thread named \"ghost\"",
    );
    assert!(w.rules().is_empty(), "nothing was added");
}

#[test]
fn rules_lists_them_and_rule_rm_removes_one_and_says_so() {
    let w = World::new();
    w.worker(&[
        "rule",
        "add",
        "X:/Proj/.claude/worktrees",
        "--stale-days",
        "5",
        "--thread",
        "coordinator",
    ]);
    ok(&w.run(&["rule", "add", "X:/Proj", "--kind", "repo"], "coordinator"));
    let table = say(&w.worker(&["rules"]));
    let flat = table.split_whitespace().collect::<Vec<_>>().join(" ");
    assert!(
        flat.contains("W-1 folder X:/Proj/.claude/worktrees coordinator 5d"),
        "{flat}"
    );
    assert!(flat.contains("W-2 repo X:/Proj coordinator 3d"), "{flat}");
    let json: Value = serde_json::from_str(&say(&w.worker(&["rules", "--json"]))).expect("json");
    let ids: Vec<&Value> = json["rules"]
        .as_array()
        .expect("rules")
        .iter()
        .map(|r| &r["id"])
        .collect();
    assert_eq!(ids, [&json!("W-1"), &json!("W-2")]);
    let rm = say(&w.worker(&["rule", "rm", "W-2"]));
    assert!(rm.contains("W-2: rule removed (X:/Proj)"), "{rm}");
    assert_eq!(w.rules().len(), 1);
    w.refuse_all(&[
        (&["rule", "rm", "W-9"], "no worktree rule \"W-9\""),
        (&["rule", "rm"], "name the rule"),
    ]);
}

#[test]
fn include_and_exclude_share_a_parent_folder_between_rules_and_a_stored_wildcard_shows_as_invalid()
{
    let w = World::new();
    let dir = "X:/relic-of-the-past/.claude/worktrees";
    let web = say(&w.worker(&[
        "rule",
        "add",
        dir,
        "--include",
        "web-*",
        "--include",
        "site-?",
    ]));
    assert!(web.contains("rule added (folder X:/relic-of-the-past/.claude/worktrees, include web-* site-?, tells worker"), "{web}");
    let rest = say(&w.worker(&[
        "rule",
        "add",
        dir,
        "--exclude",
        "web-*",
        "--exclude",
        "site-?",
    ]));
    assert!(
        rest.contains("exclude web-* site-?, tells worker"),
        "other lists make another rule: {rest}"
    );
    let rules = w.rules();
    assert_eq!(
        (rules[0]["include"].clone(), rules[0]["exclude"].clone()),
        (json!(["web-*", "site-?"]), json!([]))
    );
    assert_eq!(
        (rules[1]["include"].clone(), rules[1]["exclude"].clone()),
        (json!([]), json!(["web-*", "site-?"]))
    );
    let again = say(&w.worker(&[
        "rule",
        "add",
        dir,
        "--include",
        "site-?",
        "--include",
        "web-*",
        "--stale-days",
        "9",
    ]));
    assert!(again.contains("W-1: rule updated (staleDays)"), "{again}");
}

#[test]
fn a_stored_wildcard_rule_shows_as_invalid_in_the_table_and_in_json() {
    let w = World::new();
    let dir = "X:/relic-of-the-past/.claude/worktrees";
    ok(&w.worker(&["rule", "add", dir]));
    let mut board = w.board();
    board["worktreeRules"].as_array_mut().expect("rules").push(json!({ "id": "W-99", "path": format!("{dir}/web-*"), "kind": "folder", "thread": "worker", "note": null, "staleDays": null }));
    w.home.write_board("alpha", &board);
    let table = say(&w.worker(&["rules"]));
    assert!(
        table.contains("W-99 ")
            && table.contains(
                "invalid: wildcard path (ignored: a rule path is the parent folder, no wildcards"
            ),
        "{table}"
    );
    let json: Value = serde_json::from_str(&say(&w.worker(&["rules", "--json"]))).expect("json");
    let bad = json["rules"]
        .as_array()
        .expect("rules")
        .iter()
        .find(|r| r["id"] == "W-99")
        .cloned();
    assert_eq!(
        bad.map(|r| r["invalid"].clone()),
        Some(json!("invalid: wildcard path"))
    );
}

#[test]
fn at_most_40_rules_and_an_update_is_not_an_addition() {
    let w = World::new();
    for i in 0..40 {
        ok(&w.worker(&["rule", "add", &format!("X:/cap/{i}")]));
    }
    assert_eq!(w.rules().len(), 40);
    refused(
        &w.worker(&["rule", "add", "X:/cap/over"]),
        1,
        "40 worktree rules at most",
    );
    ok(&w.worker(&["rule", "add", "X:/cap/1", "--stale-days", "9"]));
    assert_eq!(w.rules().len(), 40);
}

#[test]
fn keep_checks_its_arguments_then_asks_the_dashboard_and_days_is_gone() {
    let w = World::new();
    w.refuse_all(&[
        (&["keep"], "name the worktree"),
        (&["keep", "X:/Proj/*", "--note", "x"], "one worktree"),
        (
            &["keep", FOO, "--note", "x", "--days", "14"],
            "--days is gone",
        ),
        (
            &["keep", FOO, "--note", "x", "--until", "2026-12-01"],
            "the dashboard is not running",
        ),
        (&["keep", FOO, "--clear"], "the dashboard is not running"),
    ]);
}

#[test]
fn done_answers_an_owner_request_and_anything_else_is_refused() {
    let w = World::new();
    let mut board = w.board();
    let id = worktree_id(FOO, cfg!(windows));
    board["entries"]["O-1-aaaa"] = json!({ "id": "O-1-aaaa", "author": "human", "kind": "followup", "thread": "worker", "text": "[worktree delete] X:/Proj/.claude/worktrees/foo: please", "createdAt": "2026-10-25T10:00:00.000Z", "worktree": { "id": id, "path": FOO, "kind": "delete", "message": "please" } });
    board["entries"]["O-2-bbbb"] = json!({ "id": "O-2-bbbb", "author": "human", "kind": "followup", "thread": "worker", "text": "a plain note", "createdAt": "2026-10-25T10:01:00.000Z" });
    w.home.write_board("alpha", &board);
    w.refuse_all(&[
        (&["done", "O-1-aaaa"], "note text is required"),
        (
            &["done", "O-2-bbbb", "--note", "x"],
            "not an owner worktree request",
        ),
        (
            &["done", "O-9-zzzz", "--note", "x"],
            "not an owner worktree request",
        ),
    ]);
    let out = say(&w.worker(&[
        "done",
        "O-1-aaaa",
        "--note",
        "removed it with git worktree remove",
    ]));
    assert!(out.contains("O-1-aaaa: answered"), "{out}");
    let answer = w.board()["worktreeAnswers"]["O-1-aaaa"].clone();
    assert_eq!(
        (answer["note"].clone(), answer["by"].clone()),
        (
            json!("removed it with git worktree remove"),
            json!("worker")
        )
    );
    assert!(answer["at"].is_string());
}

#[test]
fn a_board_written_before_worktrees_reads_as_no_rules_and_list_says_the_dashboard_is_not_running() {
    let w = World::new();
    let none: Value = serde_json::from_str(&say(&w.worker(&["rules", "--json"]))).expect("json");
    assert_eq!(none["rules"], json!([]));
    assert!(say(&w.worker(&["rules"])).contains("no worktree rules yet"));
    w.refuse_all(&[
        (&["list"], "the dashboard is not running"),
        (&["frobnicate"], "worktree verb must be one of rule add|rm, rules, auto-plan, list, leftovers, keep, done, remove, prune"),
    ]);
}

#[test]
fn remove_and_prune_check_their_arguments_before_the_dashboard_is_asked() {
    let w = World::new();
    w.refuse_all(&[
        (
            &["remove"],
            "name the worktree: coord worktree remove <id|path>",
        ),
        (&["remove", "X:/Proj/*"], "one worktree"),
        (
            &["remove", FOO, "--owner-entry", "O-1"],
            "--owner-entry goes with --discard",
        ),
        (
            &["remove", FOO, "--discard", "--owner-entry", "O-1"],
            "the dashboard is not running",
        ),
        (&["prune"], "the dashboard is not running"),
        (
            &["release"],
            "name the worktree: coord worktree release <id|path>",
        ),
        (&["release", "X:/Proj/*"], "one worktree"),
        (&["release", FOO], "the dashboard is not running"),
        (
            &["approve"],
            "name the worktree: coord worktree approve <id|path>",
        ),
        (&["approve", "X:/Proj/*"], "one worktree"),
        (
            &["approve", FOO, "X:/Proj/b"],
            "the dashboard is not running",
        ),
    ]);
    refused(
        &w.run(&["remove", FOO], "ghost"),
        1,
        "no thread named \"ghost\"",
    );
}

#[test]
fn leftovers_and_all_leftovers_check_their_arguments_before_the_dashboard_is_asked() {
    let w = World::new();
    let all = ["remove", "--all-leftovers"];
    let with =
        |more: &[&'static str]| -> Vec<&'static str> { all.iter().chain(more).copied().collect() };
    let no_branch = "--all-leftovers deletes leftover folders only: they have no branch and no work git knows, so --branch, --discard and --owner-entry do not apply (and no backup is possible)";
    w.refuse_all(&[
        (&with(&["--branch"]), no_branch),
        (&with(&["--discard"]), "do not apply"),
        (
            &["remove", FOO, "--all-leftovers"],
            "--all-leftovers takes no folder",
        ),
        (
            &["remove", FOO, "--older-than", "3d"],
            "--older-than goes with --all-leftovers",
        ),
    ]);
    for bad in ["soon", "-1", "3 weeks", ""] {
        let argv = with(&["--older-than", bad]);
        refused(
            &w.worker(&argv),
            1,
            "--older-than takes an age such as 3d, 12h, 1w or a number of days",
        );
    }
    for low in ["5h", "1h", "0d", "0.1"] {
        let argv = with(&["--older-than", low, "--reason", "x"]);
        refused(
            &w.worker(&argv),
            1,
            "--older-than goes down to 6h at the least",
        );
    }
    for (age, reason) in [("6h", "finished agents"), ("3d", ""), ("12h", "")] {
        let mut argv = with(&["--older-than", age]);
        if !reason.is_empty() {
            argv.extend(["--reason", reason]);
        }
        refused(&w.worker(&argv), 1, "the dashboard is not running");
    }
    refused(&w.run(&all, "ghost"), 1, "no thread named \"ghost\"");
    refused(&w.worker(&["leftovers"]), 1, "the dashboard is not running");
}

#[test]
fn request_and_requests_check_their_arguments_before_the_dashboard_is_asked() {
    let w = World::new();
    let all = "--all-leftovers";
    w.refuse_all(&[
        (
            &["request"],
            "ask for one thing: coord worktree request <id|path>...",
        ),
        (&["request", "--finished", FOO], "ask for one thing"),
        (&["request", all, FOO], "ask for one thing"),
        (
            &["request", "--finished", "--discard"],
            "never discard work",
        ),
        (
            &["request", all, "--owner-entry", "O-1"],
            "goes with --discard",
        ),
        (
            &["request", FOO, "--owner-entry", "O-1"],
            "goes with --discard",
        ),
        (
            &["request", all, "--branch"],
            "leftover folders have no branch: --branch does not apply to --all-leftovers.",
        ),
        (
            &["request", FOO, "--older-than", "3d"],
            "--older-than goes with --all-leftovers.",
        ),
        (
            &["request", all, "--older-than", "soon"],
            "--older-than takes an age such as 3d, 12h, 1w or a number of days",
        ),
        (
            &["request", all, "--older-than", "5h"],
            "--older-than goes down to 6h at the least",
        ),
        (&["request", "X:/Proj/*"], "name one worktree"),
        (
            &["request", FOO, "--owner-entry"],
            "--owner-entry takes one value.",
        ),
    ]);
}

/// A switch takes no value: the path after it is a worktree again, so it reaches the dashboard.
#[test]
fn a_switch_takes_no_value_so_the_path_after_it_reaches_the_dashboard() {
    let w = World::new();
    for args in [
        vec!["request", "--branch", FOO],
        vec!["request", FOO, "--discard", "--owner-entry", "O-1"],
        vec!["request", "--all-leftovers", "--older-than", "6h"],
        vec!["request", "--finished", "--branch"],
        vec!["requests"],
        vec!["requests", "--json"],
    ] {
        refused(&w.worker(&args), 1, "the dashboard is not running");
    }
}

#[test]
fn request_checks_its_idle_age_before_the_dashboard_is_asked() {
    let w = World::new();
    let all = "--all-leftovers";
    w.refuse_all(&[
        (
            &["request", FOO, "--idle", "6h"],
            "--idle goes with --finished",
        ),
        (
            &["request", all, "--idle", "6h"],
            "--idle goes with --finished",
        ),
        (
            &["request", "--finished", "--idle", "30m"],
            "--idle goes from 1h to 30d",
        ),
        (
            &["request", "--finished", "--idle", "31d"],
            "--idle goes from 1h to 30d",
        ),
        (
            &["request", "--finished", "--idle", "6h", "--discard"],
            "never discard work",
        ),
        (
            &["request", "--finished", "--idle", FOO],
            "ask for one thing",
        ),
    ]);
    for idle in [
        &["--idle", "6h"][..],
        &["--idle", "2d"],
        &["--idle", "90m"],
        &["--idle"],
    ] {
        let mut argv = vec!["request", "--finished"];
        argv.extend(idle);
        refused(&w.worker(&argv), 1, "the dashboard is not running");
    }
    refused(
        &w.run(&["request", FOO], "ghost"),
        1,
        "no thread named \"ghost\"",
    );
}

/// The board with an open item `I-1` and a done item `I-2`.
fn with_items(w: &World) {
    let mut board = w.board();
    let at = "2026-10-26T12:00:00.000Z";
    for (id, status) in [("I-1", "todo"), ("I-2", "done")] {
        board["items"][id] = json!({
            "id": id, "title": "Login", "thread": "worker", "status": status, "rev": 1,
            "rank": 1024, "createdAt": at, "createdBy": "worker", "updatedAt": at, "updatedBy": "worker",
        });
    }
    w.home.write_board("alpha", &board);
}

#[test]
fn link_writes_the_item_the_worktree_belongs_to_and_unlink_takes_it_away() {
    let w = World::new();
    with_items(&w);
    let id = worktree_id(FOO, cfg!(windows));
    let out = say(&w.worker(&["link", FOO, "I-1"]));
    assert!(out.contains(&format!("{id}: linked to I-1.")), "{out}");
    let link = w.board()["worktreeLinks"][id.as_str()].clone();
    assert_eq!(
        (link["item"].clone(), link["by"].clone()),
        (json!("I-1"), json!("worker"))
    );
    assert!(link["at"].is_string());
    let moved = say(&w.worker(&["link", &id, "I-1"]));
    assert!(moved.contains("It was linked to I-1."), "{moved}");
    let gone = say(&w.worker(&["unlink", &id]));
    assert!(
        gone.contains(&format!("{id}: unlinked from I-1.")),
        "{gone}"
    );
    assert_eq!(w.board()["worktreeLinks"].get(id.as_str()), None);
}

#[test]
fn link_and_unlink_refuse_a_missing_word_a_missing_or_closed_item_and_a_link_that_is_not_there() {
    let w = World::new();
    with_items(&w);
    w.refuse_all(&[
        (&["link", FOO], "coord worktree link <id|path> <I-n>"),
        (&["link", FOO, "I-9"], "no item \"I-9\""),
        (&["link", FOO, "I-2"], "I-2 is"),
        (&["unlink"], "coord worktree unlink <id|path>"),
        (&["unlink", FOO], "has no link made by hand"),
    ]);
}
