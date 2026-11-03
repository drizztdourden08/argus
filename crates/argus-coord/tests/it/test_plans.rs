//! I-70 P3b: test plans and their runs from the CLI, notes on items and tests, to-dos that name
//! an assignee and may be about a test.

// test-tag: critical

use crate::work_board::Work;
use serde_json::Value;

const CONTEXT: &str = "The plan needs a decision before the next run, and it is the owner's call.";

/// A board with the tests T-1 to T-3 on I-1 and the plan I-2 holding T-1 and T-2.
fn planned() -> Work {
    let work = Work::new();
    for title in ["Lobby opens", "Join is approved", "Seat is freed"] {
        work.worker_ok(&["test", "add", "I-1", title]);
    }
    let said = work.worker_ok(&[
        "test", "plan", "add", "Smoke", "--case", "T-1", "--case", "T-2",
    ]);
    assert!(
        said.contains("I-2: Smoke (test plan, worker, 2 case(s))"),
        "{said}"
    );
    work
}

fn cases(work: &Work) -> Value {
    work.board()["items"]["I-2"]["fields"]["cases"].clone()
}

#[test]
fn a_plan_with_three_cases_runs_end_to_end() {
    let work = planned();
    let said = work.worker_ok(&["test", "plan", "cases", "I-2", "--add", "T-3"]);
    assert!(said.contains("I-2: 3 case(s): T-1, T-2, T-3."), "{said}");
    let started = work.worker_ok(&["test", "plan", "start", "I-2"]);
    assert!(started.contains("I-2 run 1 started."), "{started}");
    assert_eq!(work.board()["items"]["I-2"]["stage"], "running");
    let pass = work.worker_ok(&["test", "result", "T-1", "pass", "it opens", "--run", "I-2"]);
    assert!(
        pass.contains("T-1: pass in run 1 of I-2 (1 of 3 cases)"),
        "{pass}"
    );
    let skip = work.worker_ok(&["test", "result", "T-2", "skip", "--run", "I-2"]);
    assert!(
        skip.contains("T-2: skip in run 1 of I-2 (2 of 3 cases)"),
        "{skip}"
    );
    let fail = work.worker_ok(&[
        "test",
        "result",
        "T-3",
        "fail",
        "no seat freed",
        "--run",
        "I-2",
    ]);
    assert!(
        fail.contains("T-3: fail in run 1 of I-2 (3 of 3 cases)"),
        "{fail}"
    );
    let shown = work.worker_ok(&["test", "plan", "show", "I-2"]);
    assert!(shown.contains("T-1  pass"), "{shown}");
    assert!(shown.contains("T-2  skip"), "{shown}");
    assert!(shown.contains("T-3  fail"), "{shown}");
    assert!(shown.contains("run 1: open, 3 result(s)"), "{shown}");
    let ended = work.worker_ok(&["test", "plan", "end", "I-2"]);
    assert!(
        ended.contains("1 pass, 1 fail, 0 partial, 0 blocked, 1 skip, 0 not run"),
        "{ended}"
    );
    let board = work.board();
    assert_eq!(board["items"]["I-2"]["stage"], "reported");
    let run = &board["items"]["I-2"]["fields"]["runs"][0];
    assert_eq!(run["summary"]["total"], 3);
    assert_eq!(run["results"][2]["note"], "no seat freed");
    let again = work.worker(&["test", "plan", "end", "I-2"]);
    assert_eq!(again.code, 1);
    assert!(again.err.contains("has no open run"), "{}", again.err);
}

#[test]
fn a_plan_run_writes_the_result_to_the_test_and_a_skip_writes_nothing() {
    let work = planned();
    work.worker_ok(&["test", "plan", "start", "I-2"]);
    work.worker_ok(&["test", "result", "T-1", "fail", "it hangs", "--run", "I-2"]);
    work.worker_ok(&["test", "result", "T-2", "skip", "--run", "I-2"]);
    let board = work.board();
    assert_eq!(board["tests"]["T-1"]["status"], "fail");
    assert_eq!(board["tests"]["T-1"]["result"]["text"], "it hangs");
    assert_eq!(board["tests"]["T-1"]["result"]["by"], "worker");
    assert_eq!(
        board["tests"]["T-1"]["runs"].as_array().map_or(0, Vec::len),
        1
    );
    assert_eq!(board["tests"]["T-2"]["status"], "pending");
    assert_eq!(board["tests"]["T-2"]["result"], Value::Null);
    assert_eq!(
        board["items"]["I-2"]["fields"]["runs"][0]["results"][1]["status"],
        "skip"
    );
}

#[test]
fn skip_needs_a_run_and_a_result_in_a_run_needs_a_case_of_the_plan() {
    let work = planned();
    let bare = work.worker(&["test", "result", "T-1", "skip", "later"]);
    assert_eq!(bare.code, 1);
    assert!(
        bare.err
            .contains("result must be one of pass, fail, partial, blocked, moved."),
        "{}",
        bare.err
    );
    let closed = work.worker(&["test", "result", "T-1", "pass", "ok", "--run", "I-2"]);
    assert_eq!(closed.code, 1);
    assert!(closed.err.contains("has no open run"), "{}", closed.err);
    work.worker_ok(&["test", "plan", "start", "I-2"]);
    let foreign = work.worker(&["test", "result", "T-3", "pass", "ok", "--run", "I-2"]);
    assert_eq!(foreign.code, 1);
    assert!(
        foreign.err.contains("T-3 is not a case of I-2"),
        "{}",
        foreign.err
    );
    let twice = work.worker(&["test", "plan", "start", "I-2"]);
    assert!(
        twice.err.contains("already has run 1 open"),
        "{}",
        twice.err
    );
    assert_eq!(work.board()["tests"]["T-3"]["status"], "pending");
}

#[test]
fn the_cases_of_a_plan_are_ordered_added_and_removed() {
    let work = planned();
    let said = work.worker_ok(&[
        "test", "plan", "cases", "I-2", "--remove", "T-2", "--add", "T-2", "--add", "T-3",
        "--before", "T-1",
    ]);
    assert!(said.contains("T-2, T-3, T-1"), "{said}");
    assert_eq!(cases(&work), serde_json::json!(["T-2", "T-3", "T-1"]));
    work.worker_ok(&["test", "plan", "cases", "I-2", "--remove", "T-3"]);
    assert_eq!(cases(&work), serde_json::json!(["T-2", "T-1"]));
    let dup = work.worker(&["test", "plan", "cases", "I-2", "--add", "T-1"]);
    assert!(
        dup.err.contains("T-1 is already a case of I-2"),
        "{}",
        dup.err
    );
    let gone = work.worker(&["test", "plan", "cases", "I-2", "--remove", "T-3"]);
    assert!(
        gone.err.contains("T-3 is not a case of I-2"),
        "{}",
        gone.err
    );
    let none = work.worker(&["test", "plan", "cases", "I-2"]);
    assert!(
        none.err.contains("usage: coord test plan cases"),
        "{}",
        none.err
    );
    let task = work.worker(&["test", "plan", "cases", "I-1", "--add", "T-3"]);
    assert!(task.err.contains("I-1 is not a test plan."), "{}", task.err);
    let verb = work.worker(&["test", "plan", "nope"]);
    assert!(
        verb.err.contains("plan verb must be one of"),
        "{}",
        verb.err
    );
}

#[test]
fn a_new_case_lands_in_the_plan_it_names() {
    let work = planned();
    let said = work.worker_ok(&["test", "add", "I-1", "Fourth", "--plan", "I-2"]);
    assert!(said.contains("T-4 is a case of I-2."), "{said}");
    assert_eq!(cases(&work), serde_json::json!(["T-1", "T-2", "T-4"]));
    let bad = work.worker(&["test", "add", "I-1", "Fifth", "--plan", "I-1"]);
    assert_eq!(bad.code, 1);
    assert!(bad.err.contains("I-1 is not a test plan."), "{}", bad.err);
    assert!(work.board()["tests"].get("T-5").is_none());
}

#[test]
fn a_note_is_an_entry_on_an_item_or_a_test() {
    let work = planned();
    let said = work.worker_ok(&["note", "I-1", "The cache needs a restart first"]);
    assert!(said.contains("N-1 noted on I-1."), "{said}");
    work.worker_ok(&["note", "T-1", "Flaky on the first run"]);
    let board = work.board();
    let first = &board["entries"]["N-1"];
    assert_eq!(first["kind"], "note");
    assert_eq!(first["item"], "I-1");
    assert_eq!(first["thread"], "worker");
    assert_eq!(first["status"], "done");
    assert_eq!(first["text"], "The cache needs a restart first");
    assert_eq!(board["entries"]["N-2"]["item"], "T-1");
    let listed = work.worker_ok(&["entry", "list", "--kind", "note"]);
    assert!(listed.contains("N-1") && listed.contains("N-2"), "{listed}");
    let missing = work.worker(&["note", "I-9", "x"]);
    assert!(missing.err.contains("no item \"I-9\"."), "{}", missing.err);
    let usage = work.worker(&["note"]);
    assert!(usage.err.contains("usage: coord note"), "{}", usage.err);
    let blank = work.worker(&["note", "I-1", "  "]);
    assert_eq!(blank.code, 1);
    assert_eq!(
        work.board()["entries"]
            .as_object()
            .map_or(0, serde_json::Map::len),
        2
    );
}

#[test]
fn a_question_and_a_decision_may_be_about_a_test() {
    let work = planned();
    let asked = work.worker_ok(&[
        "ask",
        "T-1",
        "Is the lobby flaky or broken?",
        "--context",
        CONTEXT,
    ]);
    assert!(asked.contains("asked on T-1"), "{asked}");
    let decided = work.worker_ok(&[
        "decide",
        "T-2",
        "Keep the join test manual",
        "--context",
        CONTEXT,
    ]);
    assert!(decided.contains("recorded on T-2"), "{decided}");
    let board = work.board();
    assert_eq!(board["entries"]["Q-1"]["item"], "T-1");
    assert_eq!(board["entries"]["D-1"]["item"], "T-2");
    let gone = work.worker(&["ask", "T-9", "Which?", "--context", CONTEXT]);
    assert!(gone.err.contains("no test \"T-9\"."), "{}", gone.err);
}

#[test]
fn a_to_do_names_an_assignee_and_may_be_about_a_test() {
    let work = planned();
    work.worker_ok(&["action", "add", "Plug in the device", "--item", "T-1"]);
    let for_lead = work.worker_ok(&[
        "action",
        "add",
        "Review the plan",
        "--item",
        "I-2",
        "--who",
        "coordinator",
    ]);
    assert!(
        for_lead.contains("H-2 is for coordinator: Review the plan"),
        "{for_lead}"
    );
    work.worker_ok(&["action", "add", "Ask brock", "--who", "lead:brock"]);
    let board = work.board();
    assert_eq!(board["actions"]["H-1"]["item"], "T-1");
    assert_eq!(board["actions"]["H-1"].get("assignee"), None);
    assert_eq!(board["actions"]["H-2"]["assignee"], "coordinator");
    assert_eq!(board["actions"]["H-3"]["assignee"], "lead:brock");
    let who = work.worker(&["action", "add", "Nobody", "--who", "nobody"]);
    assert_eq!(who.code, 1);
    let lead = work.worker(&["action", "add", "Nobody", "--who", "lead:nobody"]);
    assert_eq!(lead.code, 1);
    let test = work.worker(&["action", "add", "Missing", "--item", "T-99"]);
    assert!(test.err.contains("no test \"T-99\"."), "{}", test.err);
    let listed = work.worker_ok(&["action", "list", "--who", "coordinator"]);
    assert!(listed.contains("[coordinator] Review the plan"), "{listed}");
    assert!(!listed.contains("Plug in the device"), "{listed}");
    let moved = work.worker_ok(&["action", "set", "H-2", "--who", "owner"]);
    assert!(moved.contains("H-2: assignee updated"), "{moved}");
    assert_eq!(work.board()["actions"]["H-2"]["assignee"], "owner");
}

#[test]
fn only_a_to_do_for_the_owner_makes_a_thread_need_the_human() {
    let work = Work::new();
    work.worker_ok(&["action", "add", "Review it", "--who", "coordinator"]);
    assert_eq!(
        work.board()["threads"]["worker"]["flags"]["needsHuman"],
        false
    );
    work.worker_ok(&["action", "add", "Sign in"]);
    assert_eq!(
        work.board()["threads"]["worker"]["flags"]["needsHuman"],
        true
    );
}

#[test]
fn the_assignee_ticks_a_to_do_that_belongs_to_another_thread() {
    let work = Work::new();
    let made = work.lead(&[
        "action",
        "add",
        "Smoke the build",
        "--for",
        "coordinator",
        "--who",
        "worker",
    ]);
    assert_eq!(made.code, 0, "{}", made.err);
    let plain = work.lead(&["action", "add", "Mine", "--for", "coordinator"]);
    assert_eq!(plain.code, 0, "{}", plain.err);
    let denied = work.worker(&["action", "done", "H-2"]);
    assert_eq!(denied.code, 1);
    assert!(
        denied.err.contains("belongs to coordinator"),
        "{}",
        denied.err
    );
    let ticked = work.worker_ok(&["action", "done", "H-1"]);
    assert!(ticked.contains("H-1: done (by worker)"), "{ticked}");
    assert_eq!(work.board()["actions"]["H-1"]["status"], "done");
}
