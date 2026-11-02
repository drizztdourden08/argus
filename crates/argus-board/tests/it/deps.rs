// test-tag: critical
//! Cross-board dependencies: ask in four steps, close, reopen and cancel, the rules and their
//! refusals, reconcile after any writer, the loop walk, the history, and the export.

use std::fs;
use std::path::Path;

use crate::common::{alpha, post};
use argus_board::Board;
use argus_board::deps::{
    AskRequest, BoardRoster, CancelRequest, CloseRequest, DepError, DepState, Done, ListFilter,
    ReopenRequest, Transition, ask, by_item, cancel, close, list, reconcile, reconcile_board,
    reopen, show, sweep,
};
use argus_board::portfolio::shape;
use argus_board::store::{board_path, empty_board, read_board, save_board, with_board};
use argus_board::sync::export_docs;
use argus_core::Coll;
use argus_core::clock::{iso_from_ms, now_iso, now_ms};
use argus_core::fixture::{by_id, doc, thread};
use serde_json::{Value, json};
use tempfile::TempDir;

fn item(id: &str, status: &str) -> Value {
    doc(
        id,
        "dev",
        json!({
            "title": format!("Item {id}"), "description": null, "subject": "dev", "thread": "dev",
            "status": status, "origin": "thread", "ac": [],
        }),
        1024,
    )
}

fn project_board(key: &str) -> Board {
    let mut b = empty_board(key);
    b["threads"] = json!({
        "coordinator": thread("coordinator", "coordinator", "lead", key, 1024),
        "dev": thread("dev", "thread", "dev", key, 2048),
    });
    b["items"] = by_id(vec![item("I-1", "doing"), item("I-2", "done")]);
    b["counters"] = json!({"I": 2});
    b
}

/// A coord home with a board for each key and the portfolio that registers them.
fn world(keys: &[&str]) -> TempDir {
    let home = tempfile::tempdir().expect("temp dir");
    let mut p = json!({"project": "_portfolio", "entries": {}, "projects": {}});
    for key in keys {
        p["projects"][*key] = json!({"key": key, "name": key, "root": null});
        save_board(home.path(), &project_board(key)).expect("save a board");
    }
    shape(&mut p);
    save_board(home.path(), &p).expect("save the portfolio");
    home
}

fn read(home: &Path, key: &str) -> Board {
    read_board(home, key).expect("read").expect("a board")
}

fn edit(home: &Path, key: &str, change: impl FnOnce(&mut Board)) {
    with_board(home, key, false, |b| {
        change(b);
        Ok(())
    })
    .expect("edit the board");
}

fn add_item(home: &Path, key: &str, id: &str, status: &str) {
    edit(home, key, |b| b["items"][id] = item(id, status));
}

fn set_status(home: &Path, key: &str, id: &str, status: &str) {
    edit(home, key, |b| b["items"][id]["status"] = json!(status));
}

fn asking(project: &str, item: &str, target: &str) -> AskRequest {
    serde_json::from_value(json!({
        "project": project, "item": item, "actor": format!("{project}/dev"),
        "target": target, "title": "Expose the save hook",
    }))
    .expect("a request")
}

fn onto(mut req: AskRequest, item: &str) -> AskRequest {
    req.onto = Some(item.to_owned());
    req
}

fn ask_in(home: &Path, req: &AskRequest) -> Result<Done, DepError> {
    ask(home, req, &BoardRoster::new(home))
}

fn asked(home: &Path, project: &str, item: &str, target: &str) -> Done {
    ask_in(home, &asking(project, item, target)).expect("asked")
}

fn refused(home: &Path, req: &AskRequest) -> String {
    ask_in(home, req).expect_err("refused").to_string()
}

fn close_as(home: &Path, project: &str, how: &str, reason: Option<&str>) -> Result<Done, DepError> {
    let req = CloseRequest {
        how: how.to_owned(),
        reason: reason.map(str::to_owned),
        actor: format!("{project}/dev"),
    };
    close(home, "D-1", &req)
}

fn reopen_as(home: &Path, project: &str) -> Result<Done, DepError> {
    let req = ReopenRequest {
        actor: format!("{project}/dev"),
    };
    reopen(home, "D-1", &req)
}

fn cancel_as(home: &Path, project: &str, reason: Option<&str>) -> Result<Done, DepError> {
    let req = CancelRequest {
        reason: reason.map(str::to_owned),
        actor: format!("{project}/dev"),
    };
    cancel(home, "D-1", &req)
}

fn ledger(home: &Path, id: &str) -> Value {
    read(home, "_portfolio")["deps"][id].clone()
}

fn inbox_texts(home: &Path, project: &str, thread: &str) -> Vec<String> {
    read(home, project)["inbox"][thread]
        .as_array()
        .into_iter()
        .flatten()
        .filter_map(|m| m["text"].as_str().map(str::to_owned))
        .collect()
}

fn moves(moved: &[Transition]) -> Vec<(i64, DepState, DepState)> {
    moved.iter().map(|t| (t.seq, t.from, t.to)).collect()
}

#[test]
fn an_ask_opens_the_request_and_writes_the_ledger_and_both_mirrors() {
    let home = world(&["alpha", "beta"]);
    let h = home.path();
    let done = asked(h, "alpha", "I-1", "beta");
    assert_eq!(done.dep.id, "D-1");
    assert_eq!(done.dep.state, DepState::Open);
    assert_eq!(
        moves(&done.transitions),
        [(1, DepState::Opening, DepState::Open)]
    );
    let stored = ledger(h, "D-1");
    assert_eq!(stored["state"], "open");
    assert_eq!(stored["createdBy"], "alpha/dev");
    assert_eq!(
        stored["from"],
        json!({"project": "alpha", "item": "I-1", "thread": "dev"})
    );
    assert_eq!(
        stored["to"],
        json!({"project": "beta", "item": "I-3", "thread": "coordinator"})
    );
    assert_eq!(stored["history"][0]["from"], "opening");
    assert_eq!(stored["history"][0]["to"], "open");
    assert_eq!(done.dep.to_value()["title"], "Expose the save hook");
    let waits = read(h, "alpha")["items"]["I-1"]["waitsOn"].clone();
    let want = json!({
        "dep": "D-1", "project": "beta", "item": "I-3", "title": "Expose the save hook",
        "state": "open", "since": done.dep.opened_at, "reason": null,
    });
    assert_eq!(waits, want);
    let beta = read(h, "beta");
    let made = &beta["items"]["I-3"];
    assert_eq!(made["status"], "todo");
    assert_eq!(made["thread"], "coordinator");
    assert_eq!(made["origin"], "coordinator");
    assert_eq!(made["title"], "Expose the save hook");
    let by = json!({
        "dep": "D-1", "project": "alpha", "item": "I-1", "thread": "dev", "title": "Item I-1",
        "state": "open",
    });
    assert_eq!(made["requestedBy"], by);
    assert_eq!(
        inbox_texts(h, "beta", "coordinator"),
        ["alpha/I-1 asks you for I-3 \"Expose the save hook\" (D-1)"]
    );
}

#[test]
fn an_ask_that_breaks_a_rule_is_refused_with_the_contract_text() {
    let home = world(&["alpha", "beta"]);
    let h = home.path();
    assert_eq!(
        refused(h, &asking("alpha", "I-1", "alpha")),
        "a dependency stays inside one project: use coord item add --for"
    );
    assert_eq!(
        refused(h, &asking("alpha", "I-1", "nowhere")),
        "nowhere is not a board here: known beta"
    );
    assert_eq!(
        refused(h, &asking("alpha", "I-2", "beta")),
        "alpha/I-2 is done: only an open item can wait on another board"
    );
    let mut named = asking("alpha", "I-1", "beta");
    named.to = Some("T-9".to_owned());
    assert_eq!(
        refused(h, &named),
        "thread T-9 is not live on beta: live threads coordinator, dev"
    );
    edit(h, "beta", |b| {
        b["threads"]["coordinator"]["status"] = json!("done")
    });
    assert_eq!(
        refused(h, &asking("alpha", "I-1", "beta")),
        "beta has no live lead; name a thread with --to"
    );
    assert!(
        read(h, "_portfolio")["deps"]
            .as_object()
            .is_some_and(|d| d.is_empty())
    );
    let err = ask_in(h, &named).expect_err("refused");
    assert_eq!(err.status(), 409);
}

#[test]
fn a_link_has_one_request_and_an_item_is_requested_once() {
    let home = world(&["alpha", "beta"]);
    let h = home.path();
    asked(h, "alpha", "I-1", "beta");
    assert_eq!(
        refused(h, &asking("alpha", "I-1", "beta")),
        "alpha/I-1 already waits on beta/I-3 (D-1): cancel it first"
    );
    add_item(h, "alpha", "I-4", "todo");
    assert_eq!(
        refused(h, &onto(asking("alpha", "I-4", "beta"), "I-3")),
        "beta/I-3 is already requested by alpha/I-1 (D-1)"
    );
    add_item(h, "beta", "I-4", "doing");
    let second = ask_in(h, &onto(asking("alpha", "I-4", "beta"), "I-4")).expect("onto");
    assert_eq!(second.dep.id, "D-2");
    assert_eq!(second.dep.to.item.as_deref(), Some("I-4"));
    assert_eq!(second.dep.to.thread.as_deref(), Some("dev"));
    let beta = read(h, "beta");
    assert_eq!(beta["items"]["I-4"]["requestedBy"]["dep"], "D-2");
    assert_eq!(beta["items"]["I-4"]["status"], "doing");
}

#[test]
fn a_request_that_would_close_a_loop_is_refused_with_the_path() {
    let home = world(&["alpha", "beta", "gamma"]);
    let h = home.path();
    asked(h, "alpha", "I-1", "beta");
    asked(h, "beta", "I-3", "gamma");
    let closing = onto(asking("gamma", "I-3", "alpha"), "I-1");
    assert_eq!(
        refused(h, &closing),
        "this would close a loop: gamma/I-3 -> alpha/I-1 -> beta/I-3 -> gamma/I-3"
    );
    let deps = read(h, "_portfolio")["deps"].clone();
    assert_eq!(deps.as_object().map(serde_json::Map::len), Some(2));
}

#[test]
fn closing_as_done_sets_the_item_and_resolves_the_requester() {
    let home = world(&["alpha", "beta"]);
    let h = home.path();
    asked(h, "alpha", "I-1", "beta");
    let done = close_as(h, "beta", "done", None).expect("closed");
    assert_eq!(done.dep.state, DepState::Done);
    assert_eq!(
        moves(&done.transitions),
        [(2, DepState::Open, DepState::Done)]
    );
    assert_eq!(read(h, "beta")["items"]["I-3"]["status"], "done");
    let alpha = read(h, "alpha");
    assert_eq!(alpha["items"]["I-1"]["waitsOn"]["state"], "done");
    assert_eq!(
        inbox_texts(h, "alpha", "dev"),
        ["beta/I-3 is done (D-1): resume I-1"]
    );
    assert!(ledger(h, "D-1")["resolvedAt"].is_string());
    let dropped = world(&["alpha", "beta"]);
    asked(dropped.path(), "alpha", "I-1", "beta");
    let gone = close_as(dropped.path(), "beta", "dropped", None).expect("dropped");
    assert_eq!(gone.dep.state, DepState::Dropped);
}

#[test]
fn blocking_needs_a_reason_wakes_the_requester_and_a_move_to_doing_reopens() {
    let home = world(&["alpha", "beta"]);
    let h = home.path();
    asked(h, "alpha", "I-1", "beta");
    let no_reason = close_as(h, "beta", "blocked", None).expect_err("refused");
    assert_eq!(no_reason.to_string(), "--as blocked needs --reason");
    assert_eq!(no_reason.status(), 400);
    set_status(h, "beta", "I-3", "doing");
    let blocked = close_as(h, "beta", "blocked", Some("need the API key")).expect("blocked");
    assert_eq!(blocked.dep.state, DepState::Blocked);
    assert_eq!(
        moves(&blocked.transitions),
        [(2, DepState::Open, DepState::Blocked)]
    );
    let beta = read(h, "beta");
    assert_eq!(beta["items"]["I-3"]["status"], "todo");
    assert_eq!(beta["items"]["I-3"]["requestedBy"]["state"], "blocked");
    let waits = &read(h, "alpha")["items"]["I-1"]["waitsOn"];
    assert_eq!(waits["state"], "blocked");
    assert_eq!(waits["reason"], "need the API key");
    assert_eq!(
        inbox_texts(h, "alpha", "dev"),
        ["beta/I-3 is blocked on you (D-1): need the API key"]
    );
    set_status(h, "beta", "I-3", "doing");
    let moved = reconcile(h, "D-1").expect("reconciled");
    assert_eq!(moves(&moved), [(3, DepState::Blocked, DepState::Open)]);
    set_status(h, "beta", "I-3", "todo");
    close_as(h, "beta", "blocked", Some("again")).expect("blocked again");
    let back = reopen_as(h, "beta").expect("reopened");
    assert_eq!(back.dep.state, DepState::Open);
    let texts = inbox_texts(h, "alpha", "dev");
    assert_eq!(
        texts.last().map(String::as_str),
        Some("beta/I-3 is open again (D-1)")
    );
}

#[test]
fn a_verb_in_the_wrong_state_or_from_the_wrong_project_is_refused() {
    let home = world(&["alpha", "beta"]);
    let h = home.path();
    asked(h, "alpha", "I-1", "beta");
    let stranger = close_as(h, "zeta", "done", None).expect_err("refused");
    assert_eq!(stranger.to_string(), "D-1 is not a dependency of zeta");
    let wrong_side = cancel_as(h, "beta", None).expect_err("refused");
    assert_eq!(wrong_side.to_string(), "D-1 is not a dependency of beta");
    let open = reopen_as(h, "beta").expect_err("refused");
    assert_eq!(
        open.to_string(),
        "D-1 is open: reopen needs it blocked or done"
    );
    cancel_as(h, "alpha", Some("not needed")).expect("cancelled");
    let late = cancel_as(h, "alpha", None).expect_err("refused");
    assert_eq!(
        late.to_string(),
        "D-1 is cancelled: cancel needs it open or blocked"
    );
    let closed = close_as(h, "beta", "done", None).expect_err("refused");
    assert_eq!(
        closed.to_string(),
        "D-1 is cancelled: close needs it open or blocked"
    );
    let unknown = close(
        h,
        "D-9",
        &CloseRequest {
            how: "done".to_owned(),
            reason: None,
            actor: "beta/dev".to_owned(),
        },
    )
    .expect_err("refused");
    assert_eq!(unknown.status(), 404);
}

#[test]
fn a_cancel_drops_an_item_nobody_started_and_keeps_one_that_was() {
    let home = world(&["alpha", "beta"]);
    let h = home.path();
    asked(h, "alpha", "I-1", "beta");
    let cancelled = cancel_as(h, "alpha", Some("not needed")).expect("cancelled");
    assert_eq!(cancelled.dep.state, DepState::Cancelled);
    assert_eq!(cancelled.dep.reason.as_deref(), Some("not needed"));
    let beta = read(h, "beta");
    assert_eq!(beta["items"]["I-3"]["status"], "dropped");
    assert_eq!(beta["items"]["I-3"]["requestedBy"]["state"], "cancelled");
    assert_eq!(
        read(h, "alpha")["items"]["I-1"]["waitsOn"]["state"],
        "cancelled"
    );
    assert_eq!(
        inbox_texts(h, "beta", "coordinator")
            .last()
            .map(String::as_str),
        Some("alpha/I-1 cancelled its request I-3 (D-1)")
    );
    let started = world(&["alpha", "beta"]);
    asked(started.path(), "alpha", "I-1", "beta");
    set_status(started.path(), "beta", "I-3", "doing");
    cancel_as(started.path(), "alpha", None).expect("cancelled");
    assert_eq!(
        read(started.path(), "beta")["items"]["I-3"]["status"],
        "doing"
    );
    let again = asked(started.path(), "alpha", "I-1", "beta");
    assert_eq!(again.dep.id, "D-2");
    let waits = &read(started.path(), "alpha")["items"]["I-1"]["waitsOn"];
    assert_eq!(waits["dep"], "D-2");
}

fn after_beta_change(change: impl FnOnce(&Path)) -> (TempDir, Vec<Transition>) {
    let home = world(&["alpha", "beta"]);
    asked(home.path(), "alpha", "I-1", "beta");
    change(home.path());
    let moved = reconcile(home.path(), "D-1").expect("reconciled");
    (home, moved)
}

#[test]
fn reconcile_follows_the_assignee_item_whoever_wrote_it() {
    let (home, moved) = after_beta_change(|h| set_status(h, "beta", "I-3", "done"));
    assert_eq!(moves(&moved), [(2, DepState::Open, DepState::Done)]);
    assert_eq!(
        read(home.path(), "alpha")["items"]["I-1"]["waitsOn"]["state"],
        "done"
    );
    let (_, moved) = after_beta_change(|h| set_status(h, "beta", "I-3", "dropped"));
    assert_eq!(moves(&moved), [(2, DepState::Open, DepState::Dropped)]);
    let (home, moved) = after_beta_change(|h| {
        edit(h, "beta", |b| {
            b["items"]["I-3"]["deletedAt"] = json!(now_iso())
        });
    });
    assert_eq!(moves(&moved), [(2, DepState::Open, DepState::Gone)]);
    assert_eq!(
        inbox_texts(home.path(), "alpha", "dev"),
        ["beta/I-3 is gone (D-1): resume I-1"]
    );
    let (_, moved) = after_beta_change(|h| {
        edit(h, "beta", |b| {
            b["items"]["I-3"]["archivedAt"] = json!(now_iso())
        });
    });
    assert_eq!(moves(&moved), [(2, DepState::Open, DepState::Gone)]);
    let (_, moved) = after_beta_change(|h| {
        edit(h, "beta", |b| {
            if let Some(items) = b["items"].as_object_mut() {
                items.shift_remove("I-3");
            }
        });
    });
    assert_eq!(moves(&moved), [(2, DepState::Open, DepState::Gone)]);
}

#[test]
fn a_gone_board_or_a_retired_project_ends_the_request() {
    let (home, moved) = after_beta_change(|h| fs::remove_file(board_path(h, "beta")).expect("rm"));
    assert_eq!(moves(&moved), [(2, DepState::Open, DepState::Gone)]);
    assert_eq!(ledger(home.path(), "D-1")["reason"], "board gone");
    let (_, moved) = after_beta_change(|h| {
        edit(h, "_portfolio", |p| {
            p["projects"]["beta"]["retired"] = json!(true)
        });
    });
    assert_eq!(moves(&moved), [(2, DepState::Open, DepState::Gone)]);
    let (_, moved) = after_beta_change(|h| {
        edit(h, "_portfolio", |p| {
            p["projects"]["alpha"]["retired"] = json!(true)
        });
    });
    assert_eq!(moves(&moved), [(2, DepState::Open, DepState::Cancelled)]);
}

#[test]
fn a_requester_item_that_ends_cancels_the_request_and_tells_the_assignee() {
    let (home, moved) = after_beta_change(|h| set_status(h, "alpha", "I-1", "done"));
    assert_eq!(moves(&moved), [(2, DepState::Open, DepState::Cancelled)]);
    let beta = read(home.path(), "beta");
    assert_eq!(beta["items"]["I-3"]["status"], "dropped");
    assert_eq!(
        inbox_texts(home.path(), "beta", "coordinator")
            .last()
            .map(String::as_str),
        Some("alpha/I-1 cancelled its request I-3 (D-1)")
    );
    let (_, moved) = after_beta_change(|h| {
        edit(h, "alpha", |b| {
            b["items"]["I-1"]["deletedAt"] = json!(now_iso())
        });
    });
    assert_eq!(moves(&moved), [(2, DepState::Open, DepState::Cancelled)]);
}

#[test]
fn a_done_request_comes_back_when_the_item_is_reopened_and_the_link_is_still_open() {
    let (home, _) = after_beta_change(|h| set_status(h, "beta", "I-3", "done"));
    let h = home.path();
    set_status(h, "beta", "I-3", "doing");
    let moved = reconcile(h, "D-1").expect("reconciled");
    assert_eq!(moves(&moved), [(3, DepState::Done, DepState::Open)]);
    assert_eq!(read(h, "alpha")["items"]["I-1"]["waitsOn"]["state"], "open");
    assert_eq!(
        inbox_texts(h, "alpha", "dev").last().map(String::as_str),
        Some("beta/I-3 is open again (D-1)")
    );
    let (closed, _) = after_beta_change(|h| set_status(h, "beta", "I-3", "done"));
    set_status(closed.path(), "alpha", "I-1", "done");
    set_status(closed.path(), "beta", "I-3", "doing");
    assert!(
        reconcile(closed.path(), "D-1")
            .expect("reconciled")
            .is_empty()
    );
    assert_eq!(ledger(closed.path(), "D-1")["state"], "done");
}

#[test]
fn a_reopen_that_would_close_a_loop_stays_done() {
    let home = world(&["alpha", "beta"]);
    let h = home.path();
    add_item(h, "beta", "I-4", "doing");
    ask_in(h, &onto(asking("alpha", "I-1", "beta"), "I-4")).expect("asked");
    set_status(h, "beta", "I-4", "done");
    let moved = reconcile(h, "D-1").expect("reconciled");
    assert_eq!(moves(&moved), [(2, DepState::Open, DepState::Done)]);
    set_status(h, "beta", "I-4", "doing");
    ask_in(h, &onto(asking("beta", "I-4", "alpha"), "I-1")).expect("asked back");
    assert!(reconcile(h, "D-1").expect("reconciled").is_empty());
    assert_eq!(ledger(h, "D-1")["state"], "done");
}

fn crashed_ask(home: &Path, created: &str) {
    edit(home, "_portfolio", |p| {
        p["deps"]["D-1"] = json!({
            "id": "D-1", "rev": 1, "createdAt": created, "createdBy": "alpha/dev",
            "updatedAt": created, "state": "opening",
            "from": {"project": "alpha", "item": "I-1", "thread": "dev"},
            "to": {"project": "beta", "item": null, "thread": "coordinator"},
            "title": "Expose the save hook", "reason": null, "openedAt": null,
            "resolvedAt": null, "history": [],
        });
        p["counters"]["D"] = json!(1);
    });
}

#[test]
fn an_opening_request_is_finished_when_its_item_exists_and_cancelled_after_a_minute_without() {
    let home = world(&["alpha", "beta"]);
    let h = home.path();
    crashed_ask(h, &now_iso());
    assert!(reconcile(h, "D-1").expect("reconciled").is_empty());
    edit(h, "beta", |b| {
        b["items"]["I-3"] = item("I-3", "todo");
        b["items"]["I-3"]["requestedBy"] = json!({"dep": "D-1"});
    });
    let moved = reconcile(h, "D-1").expect("reconciled");
    assert_eq!(moves(&moved), [(1, DepState::Opening, DepState::Open)]);
    assert_eq!(ledger(h, "D-1")["to"]["item"], "I-3");
    assert_eq!(read(h, "alpha")["items"]["I-1"]["waitsOn"]["dep"], "D-1");
    assert_eq!(
        read(h, "beta")["items"]["I-3"]["requestedBy"]["project"],
        "alpha"
    );

    let stuck = world(&["alpha", "beta"]);
    crashed_ask(stuck.path(), &iso_from_ms(now_ms() - 120_000));
    let moved = reconcile(stuck.path(), "D-1").expect("reconciled");
    assert_eq!(moves(&moved), [(1, DepState::Opening, DepState::Cancelled)]);
}

#[test]
fn a_thread_moved_on_the_board_is_copied_into_the_ledger() {
    let (home, moved) = after_beta_change(|h| {
        edit(h, "beta", |b| b["items"]["I-3"]["thread"] = json!("dev"));
    });
    assert!(moved.is_empty());
    assert_eq!(ledger(home.path(), "D-1")["to"]["thread"], "dev");
}

#[test]
fn the_history_keeps_the_last_twenty_rows_and_the_numbers_keep_growing() {
    let home = world(&["alpha", "beta"]);
    let h = home.path();
    asked(h, "alpha", "I-1", "beta");
    for _ in 0..12 {
        close_as(h, "beta", "blocked", Some("waiting")).expect("blocked");
        reopen_as(h, "beta").expect("reopened");
    }
    let stored = ledger(h, "D-1");
    let rows = stored["history"].as_array().expect("history");
    assert_eq!(rows.len(), 20);
    assert_eq!(rows[0]["seq"], 6);
    assert_eq!(rows[19]["seq"], 25);
    assert_eq!(stored["rev"].as_i64().map(|r| r > 25), Some(true));
}

#[test]
fn a_sweep_and_a_board_reconcile_follow_every_open_request() {
    let home = world(&["alpha", "beta"]);
    let h = home.path();
    asked(h, "alpha", "I-1", "beta");
    set_status(h, "beta", "I-3", "done");
    let swept = sweep(h).expect("swept");
    assert!(swept.failed.is_empty());
    assert_eq!(
        moves(&swept.transitions),
        [(2, DepState::Open, DepState::Done)]
    );
    assert!(sweep(h).expect("swept").transitions.is_empty());
    set_status(h, "beta", "I-3", "doing");
    let board = reconcile_board(h, "beta").expect("reconciled");
    assert_eq!(
        moves(&board.transitions),
        [(3, DepState::Done, DepState::Open)]
    );
    assert!(
        reconcile_board(h, "gamma")
            .expect("reconciled")
            .transitions
            .is_empty()
    );
}

#[test]
fn the_list_the_item_lookup_and_the_show_answer_from_the_ledger() {
    let home = world(&["alpha", "beta"]);
    let h = home.path();
    asked(h, "alpha", "I-1", "beta");
    let count = |project: Option<&str>, view: &str, state: Option<&str>| {
        let filter = ListFilter::parse(project, Some(view), state).expect("a filter");
        list(h, &filter).expect("listed").len()
    };
    assert_eq!(count(None, "all", None), 1);
    assert_eq!(count(Some("alpha"), "waiting", None), 1);
    assert_eq!(count(Some("beta"), "waiting", None), 0);
    assert_eq!(count(Some("beta"), "for-me", Some("open")), 1);
    assert_eq!(count(Some("beta"), "for-me", Some("done")), 0);
    assert!(ListFilter::parse(None, Some("sideways"), None).is_err());
    assert!(ListFilter::parse(None, None, Some("lost")).is_err());
    assert_eq!(by_item(h, "alpha", "I-1").expect("requester").id, "D-1");
    assert_eq!(by_item(h, "beta", "I-3").expect("assignee").id, "D-1");
    let none = by_item(h, "alpha", "I-2").expect_err("none");
    assert_eq!(none.to_string(), "no dependency on alpha/I-2");
    let shown = show(h, "D-1", Some("beta")).expect("shown");
    assert_eq!(shown.requester["title"], "Item I-1");
    assert_eq!(shown.assignee["requestedBy"]["dep"], "D-1");
    let stranger = show(h, "D-1", Some("gamma")).expect_err("refused");
    assert_eq!(stranger.to_string(), "D-1 is not a dependency of gamma");
}

#[test]
fn the_page_and_the_ops_cannot_edit_the_mirrors() {
    let mut b = alpha();
    for field in ["waitsOn", "requestedBy"] {
        let (_, res) = post(
            &mut b,
            json!({"kind": "edit", "target": {"c": "items", "id": "I-2"}, "set": {field: {"dep": "D-1"}}}),
        );
        assert_eq!(res.ops.refused, 1, "{field}");
        assert!(b["items"]["I-2"].get(field).is_none(), "{field}");
    }
}

#[test]
fn the_ledger_is_exported_to_the_portfolio_scope_and_the_mirrors_survive_a_save() {
    let home = world(&["alpha", "beta"]);
    let h = home.path();
    assert_eq!(read(h, "_portfolio")["deps"], json!({}));
    assert!(read(h, "alpha").get("deps").is_none());
    asked(h, "alpha", "I-1", "beta");
    let has = |board: &Board, coll: Coll, id: &str| {
        export_docs(board, &|_| String::new())
            .iter()
            .any(|d| d.coll == coll && d.id == id)
    };
    assert!(has(&read(h, "_portfolio"), Coll::Deps, "D-1"));
    assert!(!has(&read(h, "alpha"), Coll::Deps, "D-1"));
    let alpha = read(h, "alpha");
    save_board(h, &alpha).expect("saved");
    assert_eq!(read(h, "alpha")["items"]["I-1"]["waitsOn"]["dep"], "D-1");
    assert_eq!("deps".parse::<Coll>(), Ok(Coll::Deps));
    assert!(Coll::ALL.contains(&Coll::Deps));
}
