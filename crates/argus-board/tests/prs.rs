//! The pull request ops of the local remote: open, update, merge, close and the check list.

mod common;

use argus_board::Board;
use argus_board::ops::{Status, Who, apply_op};
use common::alpha;
use serde_json::{Value, json};

const HEAD: &str = "0123456789abcdef0123456789abcdef01234567";
const MERGE: &str = "fedcba9876543210fedcba9876543210fedcba98";

fn run(board: &mut Board, op: Value) -> (Status, String) {
    let res = apply_op(board, &op, &Who::human(), None);
    (res.status, res.outcome)
}

fn open_op(extra: Value) -> Value {
    let mut set = json!({"title": "Add x", "body": "why", "head": "feat/x", "base": "main", "headCommit": HEAD});
    for (k, v) in extra.as_object().cloned().unwrap_or_default() {
        set[k.as_str()] = v;
    }
    json!({"kind": "pr-open", "set": set})
}

fn target(kind: &str, set: Value) -> Value {
    json!({"kind": kind, "target": {"c": "prs", "id": "PR-1"}, "set": set})
}

fn opened() -> Board {
    let mut b = alpha();
    let (status, _) = run(&mut b, open_op(json!({})));
    assert_eq!(status, Status::Applied);
    b
}

#[test]
fn open_makes_the_next_pr_with_a_pending_replay_and_an_audit_line() {
    let mut b = alpha();
    let res = apply_op(
        &mut b,
        &open_op(json!({"item": "I-1"})),
        &Who::human(),
        None,
    );
    assert_eq!(res.status, Status::Applied, "{}", res.outcome);
    assert_eq!(res.extra["created"], json!({"c": "prs", "id": "PR-1"}));
    let pr = &b["prs"]["PR-1"];
    assert_eq!(pr["status"], "open");
    assert_eq!(pr["head"], "feat/x");
    assert_eq!(pr["headCommit"], HEAD);
    assert_eq!(pr["item"], "I-1");
    assert_eq!(
        pr["replay"],
        json!({"state": "pending", "at": null, "url": null})
    );
    assert_eq!(pr["checks"], json!([]));
    assert_eq!(pr["rev"], 1);
    let audit = b["log"]["audit"]["entries"]
        .as_array()
        .cloned()
        .unwrap_or_default();
    assert!(audit.iter().any(|l| {
        l["text"]
            .as_str()
            .is_some_and(|t| t.contains("opened PR-1"))
    }));
    let (_, second) = run(&mut b, open_op(json!({"head": "feat/y"})));
    assert!(second.contains("PR-2"), "{second}");
}

#[test]
fn open_defaults_the_base_and_refuses_bad_fields() {
    let mut b = alpha();
    let mut op = open_op(json!({}));
    op["set"].as_object_mut().map(|m| m.remove("base"));
    assert_eq!(run(&mut b, op).0, Status::Applied);
    assert_eq!(b["prs"]["PR-1"]["base"], "main");
    let refused = [
        open_op(json!({"head": "main"})),
        open_op(json!({"head": "feat/x"})),
        open_op(json!({"head": "bad branch; rm"})),
        open_op(json!({"title": "  "})),
        open_op(json!({"headCommit": "zzzzzzz"})),
        open_op(json!({"item": "I-99", "head": "feat/z"})),
    ];
    for op in refused {
        let (status, why) = run(&mut b, op);
        assert_eq!(status, Status::Refused, "{why}");
    }
    assert_eq!(b["prs"].as_object().map(|m| m.len()), Some(1));
}

#[test]
fn update_moves_the_head_commit_and_stamps_the_record() {
    let mut b = opened();
    let fresh = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";
    let (status, why) = run(
        &mut b,
        target(
            "pr-update",
            json!({"headCommit": fresh, "title": "Add x again"}),
        ),
    );
    assert_eq!(status, Status::Applied, "{why}");
    assert_eq!(b["prs"]["PR-1"]["headCommit"], fresh);
    assert_eq!(b["prs"]["PR-1"]["title"], "Add x again");
    assert_eq!(b["prs"]["PR-1"]["rev"], 2);
    let (status, _) = run(&mut b, target("pr-update", json!({})));
    assert_eq!(status, Status::Refused);
}

#[test]
fn merge_records_the_commit_and_ends_the_pr() {
    let mut b = opened();
    let (status, why) = run(
        &mut b,
        target(
            "pr-merge",
            json!({"mergeCommit": MERGE, "mergedBy": "brock"}),
        ),
    );
    assert_eq!(status, Status::Applied, "{why}");
    let pr = &b["prs"]["PR-1"];
    assert_eq!(pr["status"], "merged");
    assert_eq!(pr["mergeCommit"], MERGE);
    assert_eq!(pr["mergedBy"], "brock");
    assert!(pr["mergedAt"].is_string());
    assert_eq!(pr["replay"]["state"], "pending");
    let (again, why) = run(&mut b, target("pr-merge", json!({"mergeCommit": MERGE})));
    assert_eq!(again, Status::Refused);
    assert!(why.contains("merged"), "{why}");
}

#[test]
fn merge_needs_a_commit_and_close_refuses_a_merged_pr() {
    let mut b = opened();
    let (status, why) = run(&mut b, target("pr-merge", json!({})));
    assert_eq!(status, Status::Refused, "{why}");
    assert_eq!(
        run(&mut b, target("pr-close", json!({}))).0,
        Status::Applied
    );
    assert_eq!(b["prs"]["PR-1"]["status"], "closed");
    assert!(b["prs"]["PR-1"]["closedAt"].is_string());
    assert_eq!(
        run(&mut b, target("pr-close", json!({}))).0,
        Status::Refused
    );
}

#[test]
fn checks_are_kept_in_order_and_unchanged_lists_write_nothing() {
    let mut b = opened();
    let (status, _) = run(
        &mut b,
        target("pr-checks", json!({"checks": ["C-2", "C-3"]})),
    );
    assert_eq!(status, Status::Applied);
    assert_eq!(b["prs"]["PR-1"]["checks"], json!(["C-2", "C-3"]));
    let rev = b["prs"]["PR-1"]["rev"].clone();
    run(
        &mut b,
        target("pr-checks", json!({"checks": ["C-2", "C-3"]})),
    );
    assert_eq!(b["prs"]["PR-1"]["rev"], rev);
    assert_eq!(
        run(&mut b, target("pr-checks", json!({"checks": ["x"]}))).0,
        Status::Refused
    );
    run(&mut b, target("pr-close", json!({})));
    assert_eq!(
        run(&mut b, target("pr-checks", json!({"checks": ["C-4"]}))).0,
        Status::Applied
    );
}

#[test]
fn an_unknown_pr_is_refused() {
    let mut b = alpha();
    let op = json!({"kind": "pr-merge", "target": {"c": "prs", "id": "PR-9"}, "set": {"mergeCommit": MERGE}});
    let (status, why) = run(&mut b, op);
    assert_eq!(status, Status::Refused);
    assert!(why.contains("PR-9"), "{why}");
}
