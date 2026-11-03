//! The tools that act as the owner.
//! K2, K4: the tools that act as the owner and their refusals.
// test-tag: critical

use argus_mcp::{CallContext, RESULT_NOTE_REQUIRED};
use regex::Regex;
use serde_json::{Value, json};

use crate::support::{Rig, ctx};

fn client(name: &str) -> CallContext {
    CallContext {
        client: name.to_owned(),
        ..ctx()
    }
}

fn body(rig: &Rig, n: usize) -> Value {
    rig.backend.calls().post[n]["body"].clone()
}

fn assert_post_id(id: &str) {
    let shape = Regex::new(r"^O-[a-z0-9]+-[a-z0-9]{4}$").expect("a pattern");
    assert!(shape.is_match(id), "{id}");
    let stamp = id.split('-').nth(1).expect("a time part");
    let made = i64::from_str_radix(stamp, 36).expect("base 36 milliseconds");
    assert!((argus_core::clock::now_ms() - made).abs() < 60_000, "{id}");
}

#[tokio::test]
async fn post_entry_stamps_ids_author_and_via_and_validates_by_kind() {
    let rig = &Rig::new();
    let args =
        json!({ "project": "alpha", "kind": "followup", "text": "ship it", "replyTo": "Q-1" });
    let r = rig.ok_as("post_entry", args, &client("claude-code")).await;
    assert_eq!(r["status"], "applied");
    let call = rig.backend.calls().post[0].clone();
    assert_eq!(call["scope"], "alpha");
    assert_eq!(call["source"], "mcp");
    assert_eq!(call["via"], "mcp:claude-code");
    assert_post_id(call["body"]["id"].as_str().expect("an id"));
    let keys: Vec<&str> = call["body"]
        .as_object()
        .expect("a body")
        .keys()
        .map(String::as_str)
        .collect();
    assert_eq!(keys, ["id", "author", "kind", "text", "replyTo"]);
    assert_eq!(call["body"]["author"], "human");
    for kind in ["followup", "ack", "pause"] {
        rig.err(
            "post_entry",
            json!({ "project": "alpha", "kind": kind }),
            "bad_request",
        )
        .await;
    }
    rig.err(
        "post_entry",
        json!({ "project": "alpha", "kind": "create", "text": "x" }),
        "bad_request",
    )
    .await;
    let refused = rig
        .call(
            "post_entry",
            json!({ "project": "alpha", "kind": "ack", "replyTo": "NOPE" }),
        )
        .await;
    assert!(refused.is_error);
    assert_eq!(refused.result["status"], "refused");
    let weird = rig
        .ok_as(
            "post_entry",
            json!({ "project": "alpha", "kind": "followup", "text": "x" }),
            &client("we<ird>/name"),
        )
        .await;
    assert_eq!(weird["status"], "applied");
    assert_eq!(
        rig.backend.calls().post.last().expect("a post")["via"],
        "mcp:weirdname"
    );
}

#[tokio::test]
async fn post_entry_messages_say_what_is_missing() {
    let rig = &Rig::new();
    let message = |kind: &'static str| async move {
        rig.err(
            "post_entry",
            json!({ "project": "alpha", "kind": kind }),
            "bad_request",
        )
        .await["message"]
            .as_str()
            .unwrap_or("")
            .to_owned()
    };
    assert_eq!(message("followup").await, "a followup needs text");
    assert_eq!(message("decision").await, "a decision needs text");
    assert_eq!(
        message("ack").await,
        "ack needs replyTo (the id it answers)"
    );
    assert_eq!(
        message("pause").await,
        "pause needs replyTo (the id it answers: the thread)"
    );
}

fn pick(rig: &Rig, n: usize, keys: &[&str]) -> Vec<Value> {
    keys.iter().map(|k| body(rig, n)[*k].clone()).collect()
}

#[tokio::test]
async fn answer_and_decision_build_the_right_entries() {
    let rig = &Rig::new();
    let answer = json!({ "project": "alpha", "entry": "Q-1", "text": "yes glow" });
    rig.ok("answer", answer).await;
    let as_decision =
        json!({ "project": "alpha", "entry": "Q-1", "text": "glow", "decision": true });
    rig.ok("answer", as_decision).await;
    let missing = json!({ "project": "alpha", "entry": "Q-77", "text": "x" });
    rig.err("answer", missing, "not_found").await;
    let decision = json!({ "project": "alpha", "text": "use the glow", "item": "I-1" });
    rig.ok("decision", decision).await;
    let want = |kind: &str, reply: Value| [json!(kind), reply];
    assert_eq!(
        pick(rig, 0, &["kind", "replyTo"]),
        want("followup", json!("Q-1"))
    );
    assert_eq!(
        pick(rig, 1, &["kind", "replyTo"]),
        want("decision", json!("Q-1"))
    );
    let standalone = pick(rig, 2, &["kind", "item", "replyTo"]);
    assert_eq!(standalone, [json!("decision"), json!("I-1"), Value::Null]);
}

async fn refused_without_a_note(rig: &Rig, outcome: &str) {
    for extra in [json!({}), json!({ "text": "" }), json!({ "text": "   " })] {
        let mut args = json!({ "project": "alpha", "test": "T-1", "outcome": outcome });
        args.as_object_mut()
            .expect("args")
            .extend(extra.as_object().expect("extra").clone());
        let e = rig.err("record_test_result", args, "bad_request").await;
        assert_eq!(e["message"], RESULT_NOTE_REQUIRED);
    }
}

#[tokio::test]
async fn record_test_result_needs_a_note_for_fail_partial_and_blocked() {
    let rig = &Rig::new();
    let fail = json!({ "project": "alpha", "test": "T-1", "outcome": "fail", "text": "no glow" });
    rig.ok("record_test_result", fail).await;
    let moved = json!({ "project": "alpha", "test": "T-1", "outcome": "moved" });
    rig.err("record_test_result", moved, "bad_request").await;
    let unknown = json!({ "project": "alpha", "test": "T-99", "outcome": "pass" });
    rig.err("record_test_result", unknown, "not_found").await;
    for outcome in ["fail", "partial", "blocked"] {
        refused_without_a_note(rig, outcome).await;
    }
    assert_eq!(
        RESULT_NOTE_REQUIRED,
        "a note is required for fail, partial and blocked: say what you saw"
    );
    let pass = json!({ "project": "alpha", "test": "T-1", "outcome": "pass" });
    rig.ok("record_test_result", pass).await;
    let keys = ["kind", "outcome", "replyTo", "text"];
    let result = [
        json!("result"),
        json!("fail"),
        json!("T-1"),
        json!("no glow"),
    ];
    assert_eq!(pick(rig, 0, &keys), result);
    let posts = rig.backend.calls().post.len();
    assert_eq!(
        posts, 2,
        "the refused notes never reached the board; the pass without a note did"
    );
    assert_eq!(
        pick(rig, 1, &["outcome", "text"]),
        [json!("pass"), Value::Null]
    );
}

#[tokio::test]
async fn run_start_is_hidden_and_by_name_unknown_names_error_and_run_stop_stops() {
    let rig = &Rig::new();
    let r = rig
        .ok_as(
            "run_start",
            json!({ "project": "alpha", "name": "hello", "agent": "A-1", "reason": "gate" }),
            &client("desktop"),
        )
        .await;
    assert_eq!(r["already"], false);
    assert_eq!(r["run"]["visible"], false);
    let want = json!({ "key": "alpha", "name": "hello", "origin": "mcp", "by": "mcp:desktop", "visible": false, "agent": "A-1", "reason": "gate" });
    assert_eq!(rig.backend.calls().run_start[0], want);
    rig.err(
        "run_start",
        json!({ "project": "alpha", "name": "nope" }),
        "unknown_run",
    )
    .await;
    rig.err(
        "run_start",
        json!({ "project": "alpha", "name": "x;rm -rf" }),
        "bad_request",
    )
    .await;
    rig.err(
        "run_start",
        json!({ "project": "alpha", "name": "hello", "cmd": "calc.exe" }),
        "bad_request",
    )
    .await;
    rig.err(
        "run_start",
        json!({ "project": "alpha", "name": "hello", "visible": true }),
        "bad_request",
    )
    .await;
    let stopped = rig
        .ok("run_stop", json!({ "project": "alpha", "run": "R-2" }))
        .await;
    assert_eq!(stopped["run"]["status"], "stopped");
    assert_eq!(rig.backend.calls().run_stop[0]["by"], "mcp:tester");
}

#[tokio::test]
async fn send_to_lead_takes_a_project_all_or_the_default() {
    let rig = &Rig::new();
    rig.ok(
        "send_to_lead",
        json!({ "project": "beta", "text": "hello lead", "decision": true }),
    )
    .await;
    rig.ok(
        "send_to_lead",
        json!({ "project": "all", "text": "everyone" }),
    )
    .await;
    let e = rig
        .err("send_to_lead", json!({ "text": "x" }), "bad_request")
        .await;
    assert_eq!(
        e["message"],
        "project is required: a project key, or \"all\""
    );
    rig.err(
        "send_to_lead",
        json!({ "project": "zzz", "text": "x" }),
        "not_found",
    )
    .await;
    let sent: Vec<Value> = rig
        .backend
        .calls()
        .send
        .iter()
        .map(|s| json!([s["project"], s["decision"], s["via"]]))
        .collect();
    assert_eq!(
        sent,
        [
            json!(["beta", true, "mcp:tester"]),
            json!(["all", false, "mcp:tester"])
        ]
    );
    let alpha = CallContext {
        default_project: Some("alpha".into()),
        ..ctx()
    };
    rig.ok_as("send_to_lead", json!({ "text": "via default" }), &alpha)
        .await;
    assert_eq!(
        rig.backend.calls().send.last().expect("a note")["project"],
        "alpha"
    );
}
