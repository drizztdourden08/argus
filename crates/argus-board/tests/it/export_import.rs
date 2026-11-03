//! The page export and the owner's posts on a board: the context of a question reaches the page
//! and survives the owner's answer, every reply kind, and the op outcomes (applied, conflict,
//! refused, duplicate).
//! K2, K3: the page export and the owner posts.

// test-tag: critical

use crate::common::{alpha, doc, post};
use argus_board::sync::{export_docs, import_entries};
use argus_core::Coll;
use argus_core::clock::now_iso;
use serde_json::{Value, json};

const CTX: &str = "The login form can remember the email. It changes what is stored on the machine, so it is your call. I will store nothing if unanswered.";

fn exported(board: &Value, coll: Coll, id: &str) -> Value {
    export_docs(board, &|_| String::new())
        .into_iter()
        .find(|d| d.coll == coll && d.id == id)
        .and_then(|d| d.data)
        .unwrap_or(Value::Null)
}

#[test]
fn the_context_reaches_the_page_docs_and_survives_importing_the_owner_answer() {
    let mut b = alpha();
    b["entries"]["Q-2"] = doc(
        "Q-2",
        "ui",
        json!({
            "kind": "question", "item": "I-1", "thread": "ui", "author": "ui", "text": "Remember the email?",
            "needs": "decision", "options": ["a", "b", "c", "d", "e"], "context": CTX, "status": "open",
            "replyTo": null, "deliveredAt": null,
        }),
        3072,
    );
    b["entries"]["N-2"] = doc(
        "N-2",
        "ui",
        json!({
            "kind": "notice", "item": null, "thread": "ui", "author": "ui", "text": "Build b2 is ready",
            "context": "It has the new login form.", "status": "open", "replyTo": null,
        }),
        4096,
    );
    assert_eq!(exported(&b, Coll::Entries, "Q-2")["context"], CTX);
    assert_eq!(
        exported(&b, Coll::Entries, "N-2")["context"],
        "It has the new login form."
    );
    let answer = json!({
        "id": "O-1", "kind": "decision", "author": "human", "replyTo": "Q-2", "item": "I-1",
        "thread": "ui", "text": "a", "status": "new", "createdAt": now_iso(),
    });
    let res = import_entries(&mut b, vec![answer], &|_| false);
    assert_eq!(res.imported, 1);
    assert_eq!(b["entries"]["Q-2"]["status"], "answered");
    assert_eq!(b["entries"]["Q-2"]["context"], CTX);
    assert_eq!(
        b["entries"]["Q-2"]["options"].as_array().map(Vec::len),
        Some(5)
    );
}

#[test]
fn the_export_has_every_page_collection_the_meta_and_the_cards() {
    let mut b = alpha();
    b["cards"] = json!({"beta": {"key": "beta"}});
    b["runDefs"] = json!({"hello": {"name": "hello", "cmd": "echo", "env": {"SECRET": "x"}}});
    b["tests"]["T-1"]["proofs"] =
        json!([{"n": 1, "file": "C:/p/T-1-1.png", "caption": "green", "at": null}]);
    let docs = export_docs(&b, &|p| format!("/files/{}", p["n"]));
    let has = |coll: Coll, id: &str| docs.iter().any(|d| d.coll == coll && d.id == id);
    assert!(has(Coll::Threads, "ui") && has(Coll::Items, "I-1") && has(Coll::Locks, "dev-server"));
    assert!(has(Coll::Log, "ui") && has(Coll::Cards, "beta") && has(Coll::Meta, "board"));
    let launch = exported(&b, Coll::Launches, "hello");
    assert_eq!(launch["env"], json!({"SECRET": ""}));
    let test = docs.iter().find(|d| d.coll == Coll::Tests && d.id == "T-1");
    let proof = &test.and_then(|d| d.data.clone()).unwrap_or_default()["proofs"][0];
    assert_eq!(proof["url"], "/files/1");
    let meta = exported(&b, Coll::Meta, "board");
    assert_eq!(meta["name"], "Alpha");
    assert_eq!(meta["handedBuild"], "b1");
}

#[test]
fn a_followup_answers_the_question_and_queues_the_thread_inbox() {
    let mut b = alpha();
    let (id, res) = post(
        &mut b,
        json!({"kind": "followup", "replyTo": "Q-1", "text": "Yes, remember it."}),
    );
    assert_eq!(res.imported, 1);
    assert_eq!(b["entries"]["Q-1"]["status"], "answered");
    assert_eq!(b["entries"][id.as_str()]["thread"], "ui");
    assert_eq!(b["entries"][id.as_str()]["createdBy"], "human");
    let inbox = b["inbox"]["ui"].as_array().cloned().unwrap_or_default();
    assert!(inbox.iter().any(|m| m["ref"] == id.as_str()));
    assert_eq!(res.deliveries["ui"].as_array().map(Vec::len), Some(1));
}

#[test]
fn a_decision_is_pending_ack_dismisses_and_done_closes_the_action() {
    let mut b = alpha();
    let (id, _) = post(
        &mut b,
        json!({"kind": "decision", "text": "Ship b1 on Friday", "item": "I-1"}),
    );
    assert_eq!(b["entries"][id.as_str()]["status"], "pending");
    post(&mut b, json!({"kind": "ack", "replyTo": "N-1"}));
    assert_eq!(b["entries"]["N-1"]["status"], "dismissed");
    post(&mut b, json!({"kind": "done", "replyTo": "H-1"}));
    assert_eq!(b["actions"]["H-1"]["status"], "done");
    assert_eq!(b["threads"]["ui"]["flags"]["needsHuman"], false);
}

#[test]
fn a_result_records_the_run_and_ticks_the_covered_ac() {
    let mut b = alpha();
    let (_, res) = post(
        &mut b,
        json!({"kind": "result", "outcome": "pass", "replyTo": "T-2"}),
    );
    assert_eq!(res.imported, 1);
    assert_eq!(b["tests"]["T-2"]["status"], "pass");
    assert_eq!(b["tests"]["T-2"]["result"]["by"], "human");
    assert_eq!(b["tests"]["T-2"]["runs"].as_array().map(Vec::len), Some(1));
    assert_eq!(b["items"]["I-1"]["ac"][1]["done"], true);
    let inbox = b["inbox"]["ui"].as_array().cloned().unwrap_or_default();
    assert!(
        inbox
            .iter()
            .any(|m| m["text"].as_str().is_some_and(|t| t.contains("T-2 pass")))
    );
}

#[test]
fn an_edit_is_applied_then_a_stale_revision_conflicts_with_the_current_value() {
    let mut b = alpha();
    let rev = b["items"]["I-2"]["rev"].clone();
    let edit = |title: &str| {
        json!({
            "kind": "edit", "target": {"c": "items", "id": "I-2"}, "set": {"title": title},
            "baseRev": rev, "prev": {"title": "Settings page"},
        })
    };
    let (ok_id, ok) = post(&mut b, edit("Settings v2"));
    assert_eq!(ok.ops.applied, 1);
    assert_eq!(b["entries"][ok_id.as_str()]["status"], "applied");
    assert_eq!(b["items"]["I-2"]["title"], "Settings v2");
    let (clash_id, clash) = post(&mut b, edit("Mine"));
    assert_eq!(clash.ops.conflict, 1);
    let entry = &b["entries"][clash_id.as_str()];
    assert_eq!(entry["status"], "conflict");
    assert_eq!(entry["current"]["title"], "Settings v2");
    assert_eq!(entry["by"], "human");
    assert_eq!(b["items"]["I-2"]["title"], "Settings v2");
}

#[test]
fn an_archive_of_a_missing_target_is_refused_as_gone() {
    let mut b = alpha();
    let (id, res) = post(
        &mut b,
        json!({"kind": "archive", "target": {"c": "items", "id": "I-99"}}),
    );
    assert_eq!(res.ops.refused, 1);
    assert_eq!(b["entries"][id.as_str()]["status"], "refused");
    assert_eq!(b["entries"][id.as_str()]["outcome"], "gone");
}

#[test]
fn the_same_post_twice_within_two_minutes_is_a_duplicate() {
    let mut b = alpha();
    let at = now_iso();
    let body =
        |text: &str| json!({"kind": "followup", "replyTo": "Q-1", "text": text, "createdAt": at});
    let (_, first) = post(&mut b, body("Double click"));
    let (second_id, second) = post(&mut b, body("Double click"));
    assert_eq!(first.imported, 1);
    assert_eq!(second.imported, 0);
    assert_eq!(b["entries"][second_id.as_str()]["status"], "duplicate");
}

#[test]
fn the_owner_edits_his_own_decision_past_200_characters_but_never_a_threads_question() {
    let mut b = alpha();
    let (id, _) = post(
        &mut b,
        json!({"kind": "decision", "text": "short", "item": "I-1"}),
    );
    let longer = "Steps: open the page.\n- first **bold** item\n"
        .repeat(70)
        .trim()
        .to_owned();
    let rev = b["entries"][id.as_str()]["rev"].clone();
    let (_, edited) = post(
        &mut b,
        json!({
            "kind": "edit", "target": {"c": "entries", "id": id}, "set": {"text": longer},
            "prev": {"text": "short"}, "baseRev": rev,
        }),
    );
    assert_eq!(edited.ops.applied, 1);
    assert_eq!(b["entries"][id.as_str()]["text"], longer.as_str());
    let (qid, refused) = post(
        &mut b,
        json!({"kind": "edit", "target": {"c": "entries", "id": "Q-1"}, "set": {"text": "x"}}),
    );
    assert_eq!(refused.ops.refused, 1);
    assert_eq!(b["entries"][qid.as_str()]["status"], "refused");
    assert_eq!(
        b["entries"]["Q-1"]["text"],
        "Should the form remember the user?"
    );
}
