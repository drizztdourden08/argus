//! K2, K4: `coord wake-fault` against an in-process server whose wake engine the test ticks by
//! hand. The list works for any session, the clear works only for the lead of project
//! `portfolio` and needs a note, and a dashboard that is down is an error, not a silent success.

// test-tag: standard

use crate::common::{Ran, coord};
use crate::remote::{ALPHA_LEAD, HELPER, MAIN, Rig, UI, env_plus, ok, refused, seed_standard};
use serde_json::{Value, json};

pub const NOTE: &str = "the ui session had crashed, restarted its wake-stream";

async fn faults(rig: &Rig, all: bool) -> Vec<Value> {
    let path = if all {
        "/api/wake-faults?all=1"
    } else {
        "/api/wake-faults"
    };
    rig.t.get(path).await.json()["faults"]
        .as_array()
        .cloned()
        .unwrap_or_default()
}

/// A server where two wakes for threads nobody listens for have become faults: the window closes
/// after 100 ms of the manual clock and the 300 ms without a listener pass after 400 more.
async fn faulted() -> Rig {
    faulted_with(seed_standard).await
}

/// [`faulted`] on a home that `seed(coord_home, folder)` filled.
pub async fn faulted_with(seed: impl FnOnce(&std::path::Path, &std::path::Path)) -> Rig {
    let config = json!({ "wake": { "windowMs": 30, "noListenerMs": 300, "reactionMs": 60000, "tickMs": 50, "aliveMs": 1 }, "wakePolicy": { "debounceMs": 0 } });
    let rig = Rig::start_manual_wakes(config, seed).await;
    for thread in ["ui", "api"] {
        let body = json!({ "thread": thread, "reason": "thread-message", "text": format!("lead: look at {thread}") });
        let queued = rig
            .t
            .write("POST", "/api/p/alpha/wakes", &body)
            .await
            .json();
        assert_eq!(queued["queued"], true);
    }
    rig.t.advance_wakes(100);
    rig.t.advance_wakes(400);
    assert_eq!(
        faults(&rig, false).await.len(),
        2,
        "both wakes became faults"
    );
    rig
}

pub fn as_session(rig: &Rig, argv: &[&str], session: Option<&str>, folder: &str) -> Ran {
    let cwd = rig.dir.path().join(folder);
    rig.run_in(argv, session, &cwd)
}

#[tokio::test(flavor = "multi_thread")]
async fn list_works_for_any_session_and_the_cleared_one_is_not_listed() {
    let rig = faulted().await;
    let cleared = json!({ "project": "alpha", "thread": "ui", "by": "lead", "note": NOTE });
    assert_eq!(
        rig.t
            .write("POST", "/api/wake-faults/clear", &cleared)
            .await
            .status,
        200
    );
    let listed = as_session(&rig, &["wake-fault", "list"], Some(UI), "checkout-alpha");
    let out = ok(&listed);
    assert!(out.contains("alpha/api") && out.contains("open"), "{out}");
    assert!(
        !out.contains("alpha/ui"),
        "the cleared one is not listed: {out}"
    );
    assert!(
        out.contains("coord wake-fault clear <project>/<thread> --note"),
        "{out}"
    );
    let all = rig.run(&["wake-fault", "list", "--all", "--json"]);
    let json: Value = serde_json::from_str(ok(&all)).expect("json");
    assert_eq!(json.as_array().map(Vec::len), Some(2));
    let ui = json
        .as_array()
        .and_then(|l| l.iter().find(|f| f["thread"] == "ui"))
        .cloned();
    assert_eq!(ui.map(|f| f["clearedBy"].clone()), Some(json!("lead")));
    let table = rig.run(&["wake-fault", "list", "--all"]);
    assert!(
        ok(&table).contains("cleared by lead: the ui session had crashed"),
        "{}",
        table.out
    );
    rig.stop().await;
}

/// `coord wake-fault clear alpha/api <extra>` as a session, from a folder of the rig.
pub fn clear(rig: &Rig, extra: &[&str], session: Option<&str>, folder: &str) -> Ran {
    let mut argv = vec!["wake-fault", "clear", "alpha/api"];
    argv.extend(extra);
    as_session(rig, &argv, session, folder)
}

#[tokio::test(flavor = "multi_thread")]
async fn clear_is_refused_for_everyone_but_the_overlord() {
    let rig = faulted().await;
    let with_note = ["--note", NOTE];
    let overlord =
        "only the Overlord (the lead thread of project portfolio) can clear a wake fault; ";
    let refusals = [
        (
            clear(&rig, &with_note, Some(UI), "checkout-alpha"),
            "this session belongs to project alpha",
        ),
        (
            clear(&rig, &with_note, Some(HELPER), "checkout-portfolio"),
            "thread helper is not the lead",
        ),
        (
            clear(&rig, &with_note, Some(ALPHA_LEAD), "checkout-alpha"),
            "this session belongs to project alpha",
        ),
        (
            clear(
                &rig,
                &["--note", NOTE, "--as", "lead"],
                Some(ALPHA_LEAD),
                "checkout-alpha",
            ),
            "this session belongs to project alpha",
        ),
        (
            clear(&rig, &with_note, None, "."),
            "this session belongs to no project",
        ),
    ];
    for (ran, who) in &refusals {
        let said = format!(
            "{overlord}{who}. A fault is cleared after the Overlord found out what is going on."
        );
        refused(ran, 1, &said);
    }
    assert_eq!(faults(&rig, false).await.len(), 2, "still open");
    rig.stop().await;
}

#[tokio::test(flavor = "multi_thread")]
async fn clear_needs_a_note_a_target_and_an_open_fault() {
    let rig = faulted().await;
    let main = |extra: &[&str]| clear(&rig, extra, Some(MAIN), "checkout-portfolio");
    refused(&main(&[]), 1, "--note is required, 10 to 300 characters");
    let short = main(&["--note", "short"]);
    refused(
        &short,
        1,
        "--note is required, 10 to 300 characters: say what was wrong and what you did (it is 5).",
    );
    let named = as_session(
        &rig,
        &["wake-fault", "clear", "api", "--note", NOTE],
        Some(MAIN),
        "checkout-portfolio",
    );
    refused(
        &named,
        1,
        "name the fault: coord wake-fault clear <project>/<thread>",
    );
    let ghost = as_session(
        &rig,
        &["wake-fault", "clear", "alpha/ghost", "--note", NOTE],
        Some(MAIN),
        "checkout-portfolio",
    );
    refused(&ghost, 1, "no open wake fault for alpha/ghost");
    let cleared = main(&["--note", NOTE]);
    assert!(
        ok(&cleared).starts_with("cleared the wake fault of alpha/api (since "),
        "{}",
        cleared.out
    );
    assert_eq!(
        faults(&rig, false).await.len(),
        1,
        "the ui fault is still open"
    );
    rig.stop().await;
}

#[tokio::test(flavor = "multi_thread")]
async fn with_both_faults_cleared_the_list_no_longer_shows_them() {
    let rig = faulted().await;
    for thread in ["ui", "api"] {
        let target = format!("alpha/{thread}");
        let cleared = as_session(
            &rig,
            &["wake-fault", "clear", &target, "--note", NOTE],
            Some(MAIN),
            "checkout-portfolio",
        );
        ok(&cleared);
    }
    let listed = rig.run(&["wake-fault", "list"]);
    let out = ok(&listed);
    assert!(
        !out.contains("alpha/ui") && !out.contains("alpha/api"),
        "{out}"
    );
    rig.stop().await;
}

#[test]
fn a_dashboard_that_is_down_is_an_error_for_list_and_clear_not_a_silent_success() {
    let home = crate::common::Home::new();
    let env = env_plus(&home.env(), &[("CLAUDE_CODE_SESSION_ID", "")]);
    let listed = coord(&["wake-fault", "list"], env, home.path());
    refused(&listed, 1, "the dashboard is not running");
    let unknown = coord(&["wake-fault", "frobnicate"], home.env(), home.path());
    refused(
        &unknown,
        1,
        "unknown wake-fault command \"frobnicate\" (list, clear).",
    );
}
