//! I-70 P3a: `coord item migrate-blockers`, the table of the design (4.4) on copies of the
//! blocker texts seen on the boards, and a dry run that writes nothing.

// test-tag: critical

use crate::work_board::Work;
use serde_json::{Value, json};

const OWNER_TEXT: &str = "external: owner, run the login test on the 0.4 build";
const OTHER_TEXT: &str = "external: brock lead publishes tessera 0.95";

/// A board with I-1 to I-6 (I-5 done) whose blockers are the texts of the table.
fn blocked_board() -> Work {
    let work = Work::new();
    for title in ["Two", "Three", "Four", "Five", "Six"] {
        work.seed_item(title);
    }
    work.worker_ok(&["item", "status", "I-5", "done"]);
    let mut board = work.board();
    let texts = [
        ("I-1", OWNER_TEXT),
        ("I-2", "I-3"),
        ("I-3", "D-77"),
        ("I-4", "D-5"),
        ("I-5", "external: owner, a go"),
        ("I-6", OTHER_TEXT),
    ];
    for (id, text) in texts {
        board["items"][id]["blocker"] = json!(text);
    }
    board["entries"]["D-5"] = json!({
        "id": "D-5", "kind": "decision", "item": null, "thread": "worker", "author": "worker",
        "text": "Use JSON", "status": "done", "rev": 1,
    });
    work.save(&board);
    work
}

fn blocker(work: &Work, id: &str) -> Value {
    work.board()["items"][id]["blocker"].clone()
}

fn todos(work: &Work) -> Vec<Value> {
    let board = work.board();
    let all = board["actions"].as_object().cloned().unwrap_or_default();
    all.values().cloned().collect()
}

#[test]
fn a_dry_run_prints_every_change_and_writes_nothing() {
    let work = blocked_board();
    let before = work.board();
    let said = work.lead(&["item", "migrate-blockers", "--dry-run"]);
    assert_eq!(said.code, 0, "{}", said.err);
    assert_eq!(work.board(), before);
    assert_eq!(said.out.matches("[dry run] I-").count(), 6, "{}", said.out);
    assert!(
        said.out
            .contains("dry run: 6 blocker(s) would migrate, 0 would stay; nothing written."),
        "{}",
        said.out
    );
}

#[test]
fn each_text_becomes_what_the_table_says() {
    let work = blocked_board();
    let ran = work.lead(&["item", "migrate-blockers"]);
    assert_eq!(ran.code, 0, "{}", ran.err);
    assert!(
        ran.out.contains("migrated 6 blocker(s), kept 0."),
        "{}",
        ran.out
    );
    let board = work.board();
    let items = &board["items"];
    assert_eq!(items["I-2"]["blockedBy"], json!(["I-3"]));
    assert_eq!(items["I-4"]["blockedBy"], json!(["D-5"]));
    assert!(items["I-3"].get("blockedBy").is_none());
    assert!(items["I-5"].get("blockedBy").is_none());
    for id in ["I-1", "I-2", "I-3", "I-4", "I-5", "I-6"] {
        assert_eq!(blocker(&work, id), Value::Null, "{id}");
    }
    let made = todos(&work);
    assert_eq!(made.len(), 2, "{made:?}");
    let owner = made.iter().find(|t| t["assignee"] == "owner").unwrap();
    assert_eq!(owner["text"], "owner, run the login test on the 0.4 build");
    assert_eq!(owner["item"], "I-1");
    let thread = made.iter().find(|t| t["assignee"] == "worker").unwrap();
    assert_eq!(thread["text"], "brock lead publishes tessera 0.95");
    assert_eq!(thread["item"], "I-6");
    assert_eq!(items["I-1"]["blockedBy"].as_array().map(Vec::len), Some(1));
    assert_eq!(items["I-6"]["blockedBy"].as_array().map(Vec::len), Some(1));
}

#[test]
fn a_second_run_finds_nothing_and_a_clean_board_says_so() {
    let work = blocked_board();
    work.lead(&["item", "migrate-blockers"]);
    let after = work.board();
    let again = work.lead(&["item", "migrate-blockers"]);
    assert_eq!(again.out, "no free-text blockers.\n");
    assert_eq!(todos(&work).len(), 2);
    assert_eq!(work.board()["items"]["I-2"], after["items"]["I-2"]);
    let clean = Work::new().lead(&["item", "migrate-blockers", "--dry-run"]);
    assert_eq!(clean.out, "no free-text blockers.\n");
}

#[test]
fn an_id_that_would_close_a_cycle_becomes_a_to_do_for_the_thread() {
    let work = Work::new();
    work.seed_item("Two");
    work.worker_ok(&["item", "block", "I-1", "--on", "I-2"]);
    let mut board = work.board();
    board["items"]["I-2"]["blocker"] = json!("I-1");
    work.save(&board);
    let ran = work.lead(&["item", "migrate-blockers"]);
    assert_eq!(ran.code, 0, "{}", ran.err);
    let board = work.board();
    assert_eq!(board["items"]["I-2"]["blocker"], Value::Null);
    let made = todos(&work);
    assert_eq!(made.len(), 1);
    assert_eq!(made[0]["text"], "I-1");
    assert_eq!(made[0]["assignee"], "worker");
    assert_eq!(
        board["items"]["I-2"]["blockedBy"].as_array().map(Vec::len),
        Some(1)
    );
}
