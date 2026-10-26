//! The server client verbs on the Node CLI and on `argus coord`, in twin temp homes: the board
//! verbs and every refusal with the dashboard down, then the verbs that read the dashboard with
//! both engines pointed at one Argus test server, so the answers compare. They skip with a
//! reason when node, `ai-config` or the `argus` binary is absent (set `ARGUS_EXE`, or build
//! `-p argus`). A dashboard is never started: every step carries a missing install folder.

mod support;

use argus_testkit::{TestOptions, TestServer};
use serde_json::json;
use std::fs;
use support::differ::exec::{Which, run_step};
use support::differ::{
    Differ, Engines, FixtureBoard, LEAD_SESSION, Report, Seed, Step, WORKER_SESSION,
};

const MAIN_SESSION: &str = "33333333-3333-4333-8333-333333333333";

/// Twin homes with `alpha`, `beta` and an Overlord project `portfolio`.
fn differ(test: &str) -> Option<Differ> {
    let engines = Engines::or_skip(test)?;
    let boards = vec![
        FixtureBoard::new("alpha")
            .thread("lead", "coordinator", Some(LEAD_SESSION))
            .thread("worker", "thread", Some(WORKER_SESSION)),
        FixtureBoard::new("beta").thread("lead", "coordinator", None),
        FixtureBoard::new("portfolio").thread("coordinator", "coordinator", Some(MAIN_SESSION)),
    ];
    let sessions = [LEAD_SESSION, WORKER_SESSION, MAIN_SESSION];
    Some(Differ::new(engines, &Seed::Fixture(boards), &sessions).expect("twin homes"))
}

/// A step that can never start a dashboard: no install folder, a short wait.
fn step(words: &[&str], session: &str) -> Step {
    Step::new(words)
        .session(session)
        .env("COORD_DASHBOARD_INSTALL", "no-such-install")
        .env("COORD_DASHBOARD_WAIT_MS", "300")
}

fn lead(words: &[&str]) -> Step {
    step(words, LEAD_SESSION)
}

fn worker(words: &[&str]) -> Step {
    step(words, WORKER_SESSION)
}

fn overlord(words: &[&str]) -> Step {
    step(words, MAIN_SESSION)
}

fn compare(differ: &Differ, steps: &[Step]) -> Report {
    let report = differ.run(steps).expect("run the steps");
    println!("{report}");
    report.assert_no_mismatch();
    assert_eq!(report.steps.len(), steps.len());
    assert_eq!(report.not_ported(), 0, "{report}");
    report
}

#[test]
fn run_definitions_and_every_refusal_with_the_dashboard_down() {
    let Some(d) = differ("run_definitions") else {
        return;
    };
    compare(
        &d,
        &[
            lead(&["run", "defs", "--project", "alpha"]),
            lead(&[
                "run",
                "define",
                "build",
                "--cmd",
                "npm run build",
                "--kind",
                "build",
                "--env",
                "TOKEN=s3cret",
                "--env",
                "MODE=prod",
            ]),
            lead(&[
                "run",
                "define",
                "build",
                "--cmd",
                "npm run build:all",
                "--timeout",
                "0",
            ]),
            lead(&[
                "run",
                "define",
                "app",
                "--cmd",
                "start /wait app.exe",
                "--hand",
                "--visible",
                "--parallel",
                "2",
            ]),
            lead(&["run", "defs"]),
            lead(&["run", "defs", "--json"]),
            worker(&["run", "define", "evil", "--cmd", "calc"]),
            lead(&["run", "define", "x", "--cmd", "echo", "--visible"]),
            lead(&["run", "define", "Bad Name", "--cmd", "echo"]),
            lead(&["run", "define", "x"]),
            lead(&[
                "run",
                "define",
                "x",
                "--cmd",
                "echo",
                "--cwd",
                "missing-folder",
            ]),
            lead(&["run", "define", "x", "--cmd", "echo", "--kind", "weird"]),
            lead(&["run", "define", "x", "--cmd", "echo", "--env", "NOEQUALS"]),
            lead(&["run", "define", "x", "--cmd", "echo", "--timeout", "soon"]),
            lead(&["run", "define", "x", "--cmd", "echo", "--parallel", "9"]),
            worker(&["run", "undefine", "build"]),
            lead(&["run", "undefine", "build"]),
            lead(&["run", "undefine", "build"]),
            lead(&["run", "nonsense"]),
            lead(&["run"]),
            lead(&["run", "start", "build"]),
            lead(&["run", "list"]),
            lead(&["run", "stop", "R-1"]),
            lead(&["run", "logs", "R-1", "--follow"]),
            lead(&["run", "status"]),
        ],
    );
}

#[test]
fn the_verbs_that_need_the_dashboard_say_so_with_it_down() {
    let Some(d) = differ("verbs_need_the_dashboard") else {
        return;
    };
    compare(
        &d,
        &[
            lead(&["ping", "the build is ready"]),
            lead(&["ping", "x", "--level", "loud"]),
            lead(&["ping", "y"])
                .cwd("elsewhere")
                .env("COORD_PROJECT", "nope"),
            lead(&["ping"]),
            lead(&["ping", &"y".repeat(241)]),
            lead(&["ping", "x", "--title", &"T".repeat(61)]),
            lead(&["activity"]),
            lead(&["activity", "--limit", "0"]),
            lead(&[
                "activity", "--limit", "5", "--kind", "note", "--level", "warn", "--json",
            ]),
            lead(&["proc", "list"]),
            lead(&["proc", "rules", "--json"]),
            lead(&["proc", "help"]),
            lead(&["proc", "frobnicate"]),
            lead(&["wake-fault", "list"]),
            lead(&["wake-fault", "list", "--all", "--json"]),
            lead(&["wake-fault", "frobnicate"]),
            lead(&["dashboard"]),
        ],
    );
}

#[test]
fn the_overlord_only_verbs_refuse_everyone_else_and_check_their_arguments() {
    let Some(d) = differ("overlord_only") else {
        return;
    };
    let note = "the ui session had crashed, restarted its wake-stream";
    compare(
        &d,
        &[
            lead(&[
                "proc",
                "rule",
                "set",
                "py",
                "--name-match",
                "^python",
                "--max-mem",
                "800",
            ]),
            worker(&["proc", "kill", "123", "--reason", "testing it"]),
            lead(&["proc", "kill-rule", "test-kids"]),
            lead(&["proc", "kill", "123"]),
            overlord(&["proc", "rule"]),
            overlord(&["proc", "rule", "set"]),
            overlord(&["proc", "rule", "set", "py"]),
            overlord(&["proc", "rule", "set", "py", "--idle", "soon"]),
            overlord(&["proc", "rule", "set", "py", "--max-mem", "lots"]),
            overlord(&[
                "proc",
                "rule",
                "set",
                "py",
                "--max-age",
                "2x",
                "--note",
                "n",
            ]),
            overlord(&["proc", "rule", "set", "py", "--note", "python servers"]),
            overlord(&["proc", "rule", "rm", "py"]),
            overlord(&["proc", "kill"]),
            overlord(&["proc", "kill", "999999"]),
            overlord(&["proc", "kill", "999999", "--reason", "not there"]),
            overlord(&["proc", "kill-rule"]),
            overlord(&["proc", "kill-rule", "x", "--older-than", "soon"]),
            lead(&["wake-fault", "clear", "alpha/ui", "--note", note]),
            overlord(&["wake-fault", "clear", "alpha/ui"]),
            overlord(&["wake-fault", "clear", "alpha/ui", "--note", "short"]),
            overlord(&["wake-fault", "clear", "api", "--note", note]),
            overlord(&["wake-fault", "clear", "alpha/ui", "--note", note]),
        ],
    );
}

#[test]
fn worktree_rules_keeps_and_answers_on_the_board() {
    let Some(d) = differ("worktree_rules") else {
        return;
    };
    let dir = "X:/relic-of-the-past/.claude/worktrees";
    compare(
        &d,
        &[
            worker(&["worktree", "rules"]),
            worker(&["worktree", "rules", "--json"]),
            worker(&[
                "worktree",
                "rule",
                "add",
                "X:\\Proj\\.claude\\worktrees\\",
                "--note",
                "the lead builds here",
            ]),
            worker(&[
                "worktree",
                "rule",
                "add",
                "x:/proj/.claude/worktrees",
                "--stale-days",
                "5",
                "--thread",
                "lead",
            ]),
            worker(&["worktree", "rule", "add", "x:/proj/.claude/worktrees"]),
            lead(&[
                "worktree",
                "rule",
                "add",
                "X:/Proj",
                "--kind",
                "repo",
                "--note",
                "the repo itself",
            ]),
            worker(&[
                "worktree",
                "rule",
                "add",
                dir,
                "--include",
                "web-*",
                "--include",
                "site-?",
            ]),
            worker(&[
                "worktree",
                "rule",
                "add",
                dir,
                "--exclude",
                "web-*",
                "--exclude",
                "site-?",
            ]),
            worker(&[
                "worktree",
                "rule",
                "add",
                dir,
                "--include",
                "site-?",
                "--include",
                "web-*",
                "--stale-days",
                "9",
            ]),
            worker(&["worktree", "rules"]),
            worker(&["worktree", "rules", "--json"]),
            worker(&["worktree", "rule", "rm", "W-2"]),
            worker(&["worktree", "rule", "remove", "W-9"]),
            worker(&[
                "worktree",
                "keep",
                "X:/Proj/.claude/worktrees/foo",
                "--note",
                "the owner wants it",
            ]),
            worker(&[
                "worktree",
                "keep",
                "X:/Proj/.claude/worktrees/foo",
                "--note",
                "two weeks",
                "--days",
                "14",
            ]),
            worker(&[
                "worktree",
                "keep",
                "X:/Proj/.claude/worktrees/foo",
                "--clear",
            ]),
            worker(&[
                "worktree",
                "keep",
                "X:/Proj/.claude/worktrees/foo",
                "--clear",
            ]),
            worker(&["worktree", "done", "O-9-zzzz", "--note", "x"]),
            worker(&["worktree", "list"]),
            worker(&["worktree", "leftovers"]),
            worker(&["worktree", "prune"]),
            worker(&["worktree", "remove", "X:/Proj/.claude/worktrees/foo"]),
            worker(&[
                "worktree",
                "remove",
                "--all-leftovers",
                "--older-than",
                "3d",
            ]),
            worker(&["worktree", "frobnicate"]),
        ],
    );
}

#[test]
fn worktree_arguments_are_refused_alike() {
    let Some(d) = differ("worktree_refusals") else {
        return;
    };
    let long = format!("X:/{}", "a".repeat(300));
    let mut steps = vec![worker(&["worktree", "rule", "add"])];
    for path in [
        "X:/*/worktrees",
        "X:/a/b?",
        "X:/a/[x]",
        "relative/dir",
        "X:/a/../b",
        "~/work",
        "X:/a/b|c",
        "X:/a/**/b",
        &long,
    ] {
        steps.push(worker(&["worktree", "rule", "add", path]));
    }
    for (flag, value) in [
        ("--kind", "tree"),
        ("--thread", "nobody"),
        ("--stale-days", "0"),
        ("--stale-days", "400"),
        ("--include", "a/b"),
        ("--include", "x\u{1}y"),
    ] {
        steps.push(worker(&["worktree", "rule", "add", "X:/a", flag, value]));
    }
    steps.push(worker(&[
        "worktree",
        "rule",
        "add",
        "X:/a",
        "--kind",
        "repo",
        "--include",
        "web-*",
    ]));
    steps.push(worker(&["worktree", "rule", "add", "X:/a", "--include"]));
    steps.push(worker(&["worktree", "keep", "X:/Proj/*", "--note", "x"]));
    steps.push(worker(&[
        "worktree", "keep", "X:/a", "--note", "x", "--days", "0",
    ]));
    steps.push(worker(&["worktree", "keep", "X:/a", "--note"]));
    for age in ["soon", "-1", "3 weeks", "5h", "0.1"] {
        steps.push(worker(&[
            "worktree",
            "remove",
            "--all-leftovers",
            "--older-than",
            age,
            "--reason",
            "x",
        ]));
    }
    steps.push(worker(&[
        "worktree",
        "remove",
        "--all-leftovers",
        "--branch",
    ]));
    compare(&d, &steps);
}

#[test]
fn portfolio_registry_sends_lead_messages_and_the_overview() {
    let Some(d) = differ("portfolio_and_lead") else {
        return;
    };
    let long = "z".repeat(241);
    compare(
        &d,
        &[
            lead(&["portfolio", "show"]),
            lead(&["portfolio", "init"]),
            lead(&[
                "portfolio",
                "add",
                "alpha",
                "--name",
                "Alpha",
                "--root",
                "checkouts/alpha",
            ]),
            lead(&["portfolio", "add", "beta", "--name", "Beta"]),
            lead(&["portfolio", "add", "Bad"]),
            lead(&["portfolio", "add", "ghost"]),
            lead(&["portfolio", "add", "ghost", "--name", "Ghost"]),
            lead(&["portfolio", "show"]),
            lead(&["portfolio", "show", "--json"]),
            lead(&["portfolio", "send", "beta", "please look at the lint rule"]),
            lead(&[
                "portfolio",
                "send",
                "all",
                "freeze the config",
                "--decision",
            ]),
            lead(&["portfolio", "send", "ghost", "anyone home"]),
            lead(&["portfolio", "send", "nowhere", "hello"]),
            lead(&["portfolio", "send", "beta", &long]),
            lead(&["portfolio", "send", "beta", &"z".repeat(201), "--decision"]),
            lead(&["portfolio", "remove", "ghost"]),
            lead(&["portfolio", "remove", "Bad"]),
            lead(&["portfolio", "page"]),
            lead(&["portfolio", "frobnicate"]),
            lead(&["lead", "beta", "rule X fires on Y, I think it is wrong"]),
            lead(&[
                "lead",
                "beta",
                "please freeze the lint config",
                "--kind",
                "decision",
            ]),
            lead(&["lead", "alpha", "to myself"]),
            lead(&["lead", "nowhere", "hello"]),
            lead(&["lead", "ghost", "hello"]),
            lead(&["lead", "beta"]),
            lead(&["lead", "beta", &long]),
            lead(&["lead", "beta", "x", "--kind", "shout"]),
            lead(&["lead"]),
            lead(&["lead", "beta", "from elsewhere", "--project", "alpha"]).cwd("elsewhere"),
            Step::new(&["lead", "beta", "no session"])
                .cwd("elsewhere")
                .env("COORD_DASHBOARD_INSTALL", "no-such-install"),
        ],
    );
}

#[tokio::test(flavor = "multi_thread")]
async fn the_verbs_that_read_the_dashboard_answer_alike_from_one_server() {
    let Some(d) = differ("one_server") else {
        return;
    };
    let server = TestServer::start(TestOptions {
        seed: true,
        config: json!({ "notify": { "os": false } }),
        ..TestOptions::default()
    })
    .await;
    let twin = d.twin();
    for home in [&twin.node, &twin.rust] {
        let config = json!({ "port": server.port }).to_string();
        fs::write(home.dashboard().join("config.json"), config).expect("config.json");
    }
    compare(
        &d,
        &[
            lead(&["activity"]),
            lead(&["activity", "--limit", "5", "--json"]),
            lead(&["activity", "--kind", "nothing-like-this"]),
            lead(&["activity", "--level", "loud"]),
            lead(&["proc", "list"]),
            lead(&["proc", "list", "--json", "--rule", "none"]),
            lead(&["proc", "rules"]),
            lead(&["wake-fault", "list"]),
            lead(&["wake-fault", "list", "--all", "--json"]),
            overlord(&[
                "wake-fault",
                "clear",
                "alpha/ui",
                "--note",
                "the ui session had crashed, restarted",
            ]),
            overlord(&["proc", "rule", "rm", "nothing"]),
            overlord(&["proc", "kill", "999999", "--reason", "not there"]),
            overlord(&["proc", "kill-rule", "none-such", "--older-than", "6h"]),
            lead(&["worktree", "list"]),
            lead(&["worktree", "leftovers", "--json"]),
            worker(&["worktree", "prune"]),
            worker(&["worktree", "remove", "X:/Proj/.claude/worktrees/foo"]),
            lead(&["run", "list"]),
            lead(&["run", "list", "--json", "--status", "running"]),
            lead(&["run", "status"]),
            lead(&["run", "status", "R-9"]),
            lead(&["run", "stop", "R-9"]),
            lead(&["run", "logs", "R-9"]),
            lead(&["run", "start", "ghost"]),
            lead(&["run", "start", "ghost", "--agent", "bob"]),
            lead(&["ping", "the build is ready"]),
            lead(&["ping", "info only", "--level", "info", "--title", "Quiet"]),
            overlord(&["ping", "the Overlord needs you", "--level", "urgent"]),
        ],
    );
    server.stop(false).await;
}

/// The cards of two engines differ in the folders of their temp homes, so their hashes cannot
/// compare across homes: here both engines tick the same home, Node first, and the Argus tick
/// must find every card hash unchanged.
#[test]
fn the_argus_portfolio_tick_hashes_a_card_as_node_does() {
    let Some(d) = differ("portfolio_tick_hashes") else {
        return;
    };
    let engines = Engines::locate().expect("engines");
    let home = &d.twin().rust;
    let port = d.twin().closed_port;
    let tick = lead(&["portfolio", "tick"]);
    let setup = [
        lead(&["portfolio", "init"]),
        lead(&["portfolio", "add", "beta", "--name", "Beta"]),
    ];
    for step in setup.iter().chain([&tick]) {
        run_step(&engines, Which::Node, home, port, step).expect("node step");
    }
    let board = || fs::read_to_string(home.board("_portfolio")).expect("portfolio board");
    let after_node = board();
    let hashes: serde_json::Value = serde_json::from_str(&after_node).expect("json");
    assert!(hashes["pushedHash"]["alpha"].is_string(), "{after_node}");
    let seen = run_step(&engines, Which::Argus, home, port, &tick).expect("argus tick");
    assert_eq!(
        seen.code,
        Some(0),
        "{}",
        String::from_utf8_lossy(&seen.stderr)
    );
    let after_argus: serde_json::Value = serde_json::from_str(&board()).expect("json");
    assert_eq!(after_argus["pushedHash"], hashes["pushedHash"]);
    assert_eq!(after_argus["cards"], hashes["cards"]);
    let quiet: serde_json::Value = serde_json::from_slice(&seen.stdout).expect("tick json");
    assert_eq!(
        quiet["attention"],
        json!([]),
        "nothing is new since the Node tick"
    );
}
