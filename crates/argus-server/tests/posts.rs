//! Owner posts over HTTP: every reply kind and the op outcomes (applied, conflict, refused,
//! duplicate), idempotent ids, the result-note rule, a post waiting on a held board.lock,
//! proofs, portfolio posts, recovery of a post left `received`, and shadow mode.
mod posting;

use std::time::{Duration, Instant};

use argus_store::rusqlite::params;
use argus_testkit::{Request, TestOptions, TestServer};
use posting::{cli, cli_to, doc, entry, entry_count, id, inbox, iso, page_put, post, seeded, with};
use serde_json::{Value, json};

const NOTE: &str = "a note is required for fail, partial and blocked: say what you saw";
const PNG_HEX: &str = "89504e470d0a1a0a0000000d49484452000000010000000108060000001f15c4890000000d49444154789c6360000002000154a24f5d0000000049454e44ae426082";

fn png() -> Vec<u8> {
    (0..PNG_HEX.len())
        .step_by(2)
        .map(|i| u8::from_str_radix(&PNG_HEX[i..i + 2], 16).expect("hex"))
        .collect()
}

fn text_of(v: &Value, field: &str) -> String {
    v[field].as_str().unwrap_or_default().to_owned()
}

async fn status_of(t: &TestServer, key: &str, body: &Value) -> u16 {
    cli_to(t, key, body).await.status
}

fn seq_of(v: &Value) -> i64 {
    v["seq"].as_i64().unwrap_or_default()
}

#[tokio::test(flavor = "multi_thread")]
async fn a_followup_answers_the_question_and_the_reply_carries_the_export_seq() {
    let t = seeded().await;
    let r = post(
        &t,
        &json!({ "kind": "followup", "replyTo": "Q-1", "text": "Yes, remember it." }),
    )
    .await;
    assert_eq!(r["status"], "applied");
    assert_eq!(r["entry"]["author"], "human");
    assert_eq!(r["entry"]["via"], "cli");
    assert!(seq_of(&r) > 0);
    let q = t.get("/api/p/alpha/db/entries/Q-1").await.json();
    assert_eq!(q["data"]["status"], "answered");
    assert!(seq_of(&q) >= seq_of(&r));
    let id = text_of(&r, "id");
    assert_eq!(doc(&t, "alpha", "entries", &id).await["thread"], "ui");
    assert!(inbox(&t, "alpha", "ui").iter().any(|m| m["ref"] == id));
    assert_eq!(t.board("alpha")["dirty"], json!({}));
}

#[tokio::test(flavor = "multi_thread")]
async fn the_same_id_twice_applies_once_and_returns_the_stored_result() {
    let t = seeded().await;
    let body = entry(&json!({ "kind": "followup", "replyTo": "N-1", "text": "Thanks" }));
    let a = cli_to(&t, "alpha", &body).await.json();
    let before = inbox(&t, "alpha", "ui").len();
    let b = cli_to(&t, "alpha", &body).await.json();
    assert_eq!(a["status"], "applied");
    assert_eq!(b["status"], "applied");
    assert_eq!(b["id"], a["id"]);
    assert_eq!(inbox(&t, "alpha", "ui").len(), before);
}

#[tokio::test(flavor = "multi_thread")]
async fn a_decision_is_pending_ack_dismisses_and_done_closes_the_action() {
    let t = seeded().await;
    let d = post(
        &t,
        &json!({ "kind": "decision", "text": "Ship b1 on Friday", "item": "I-1" }),
    )
    .await;
    assert_eq!(d["status"], "pending");
    let a = post(&t, &json!({ "kind": "ack", "replyTo": "N-1" })).await;
    assert_eq!(a["status"], "applied");
    assert_eq!(
        doc(&t, "alpha", "entries", "N-1").await["status"],
        "dismissed"
    );
    let done = post(&t, &json!({ "kind": "done", "replyTo": "H-1" })).await;
    assert_eq!(done["status"], "applied");
    assert_eq!(doc(&t, "alpha", "actions", "H-1").await["status"], "done");
}

#[tokio::test(flavor = "multi_thread")]
async fn pause_and_resume_a_thread() {
    let t = seeded().await;
    post(&t, &json!({ "kind": "pause", "replyTo": "ui" })).await;
    assert_eq!(doc(&t, "alpha", "threads", "ui").await["status"], "paused");
    post(&t, &json!({ "kind": "resume", "replyTo": "ui" })).await;
    assert_ne!(doc(&t, "alpha", "threads", "ui").await["status"], "paused");
}

#[tokio::test(flavor = "multi_thread")]
async fn the_one_click_result_records_the_run_and_ticks_the_covered_ac() {
    let t = seeded().await;
    let r = t
        .write(
            "POST",
            "/api/p/alpha/tests/T-2/result",
            &json!({ "outcome": "pass" }),
        )
        .await
        .json();
    assert_eq!(r["status"], "applied");
    assert_eq!(r["entry"]["kind"], "result");
    let test2 = doc(&t, "alpha", "tests", "T-2").await;
    assert_eq!(test2["status"], "pass");
    assert_eq!(test2["result"]["by"], "human");
    assert_eq!(
        doc(&t, "alpha", "items", "I-1").await["ac"][1]["done"],
        true
    );
    let bad = t
        .write(
            "POST",
            "/api/p/alpha/tests/T-2/result",
            &json!({ "outcome": "great" }),
        )
        .await;
    assert_eq!(bad.status, 400);
}

#[tokio::test(flavor = "multi_thread")]
async fn a_fail_partial_or_blocked_result_without_a_note_is_refused_and_nothing_is_stored() {
    let t = seeded().await;
    let entries = entry_count(&t, "alpha");
    let t1 = doc(&t, "alpha", "tests", "T-1").await["status"].clone();
    let woken = inbox(&t, "alpha", "ui").len();
    for outcome in ["fail", "partial", "blocked"] {
        let plain = json!({ "kind": "result", "outcome": outcome, "replyTo": "T-1" });
        let mut bodies = vec![plain.clone()];
        bodies.extend(["", "  \n "].map(|text| with(plain.clone(), &json!({ "text": text }))));
        for body in bodies {
            let r = post(&t, &body).await;
            assert_eq!(r["status"], "refused", "{body}");
            assert_eq!(r["outcome"], NOTE);
            assert!(r["entry"].is_null());
        }
    }
    let route = json!({ "outcome": "fail", "text": " " });
    let via = t
        .write("POST", "/api/p/alpha/tests/T-1/result", &route)
        .await
        .json();
    assert_eq!(
        json!([via["status"], via["outcome"]]),
        json!(["refused", NOTE])
    );
    assert_eq!(doc(&t, "alpha", "tests", "T-1").await["status"], t1);
    assert_eq!(
        entry_count(&t, "alpha"),
        entries,
        "no entry reached the board"
    );
    assert_eq!(inbox(&t, "alpha", "ui").len(), woken, "nobody was woken");
}

#[tokio::test(flavor = "multi_thread")]
async fn the_same_id_of_a_refused_result_gives_the_stored_refusal() {
    let t = seeded().await;
    let body =
        entry(&json!({ "kind": "result", "outcome": "partial", "replyTo": "T-1", "text": "" }));
    let a = cli_to(&t, "alpha", &body).await.json();
    let b = cli_to(&t, "alpha", &body).await.json();
    let seen = json!([a["status"], b["status"], b["outcome"]]);
    assert_eq!(seen, json!(["refused", "refused", NOTE]));
}

#[tokio::test(flavor = "multi_thread")]
async fn with_a_note_the_result_is_applied_and_a_pass_needs_none() {
    let t = seeded().await;
    let ok = post(
        &t,
        &json!({ "kind": "result", "outcome": "fail", "replyTo": "T-1", "text": "The button does nothing" }),
    )
    .await;
    assert_eq!(ok["status"], "applied");
    let t1 = doc(&t, "alpha", "tests", "T-1").await;
    let seen = json!([t1["status"], t1["result"]["text"]]);
    assert_eq!(seen, json!(["fail", "The button does nothing"]));
    let said = inbox(&t, "alpha", "ui");
    assert!(
        said.iter()
            .any(|m| text_of(m, "text").contains("The button does nothing"))
    );
    let pass = post(
        &t,
        &json!({ "kind": "result", "outcome": "pass", "replyTo": "T-1" }),
    )
    .await;
    assert_eq!(pass["status"], "applied");
    assert_eq!(doc(&t, "alpha", "tests", "T-1").await["status"], "pass");
}

#[tokio::test(flavor = "multi_thread")]
async fn an_edit_is_applied_and_a_stale_revision_is_a_conflict() {
    let t = seeded().await;
    let item = doc(&t, "alpha", "items", "I-2").await;
    let edit = |title: &str| json!({ "kind": "edit", "target": { "c": "items", "id": "I-2" }, "set": { "title": title }, "baseRev": item["rev"], "prev": { "title": item["title"] } });
    let ok = post(&t, &edit("Settings v2")).await;
    assert_eq!(ok["status"], "applied", "{}", ok["outcome"]);
    assert_eq!(
        doc(&t, "alpha", "items", "I-2").await["title"],
        "Settings v2"
    );
    let clash = post(&t, &edit("Mine")).await;
    assert_eq!(clash["status"], "conflict");
    assert_eq!(clash["entry"]["current"]["title"], "Settings v2");
}

#[tokio::test(flavor = "multi_thread")]
async fn archiving_a_missing_doc_is_refused_as_gone() {
    let t = seeded().await;
    let gone = post(
        &t,
        &json!({ "kind": "archive", "target": { "c": "items", "id": "I-99" } }),
    )
    .await;
    assert_eq!(gone["status"], "refused");
    assert_eq!(gone["outcome"], "gone");
}

#[tokio::test(flavor = "multi_thread")]
async fn the_same_post_under_a_new_id_is_a_duplicate() {
    let t = seeded().await;
    let same =
        json!({ "kind": "followup", "replyTo": "Q-1", "text": "Double click", "createdAt": iso() });
    let first = post(&t, &same).await;
    let second = post(&t, &same).await;
    assert_eq!(first["status"], "applied");
    assert_eq!(second["status"], "duplicate");
}

#[tokio::test(flavor = "multi_thread")]
async fn the_page_path_put_of_entries_is_a_post() {
    let t = seeded().await;
    let data = json!({ "kind": "followup", "replyTo": "Q-1", "text": "From the page", "status": "new", "author": "human", "createdAt": iso() });
    let r = page_put(&t, "alpha", &id(), &data).await.json();
    assert_eq!(r["status"], "applied");
    assert_eq!(r["entry"]["via"], "page");
}

#[tokio::test(flavor = "multi_thread")]
async fn bad_posts_and_other_page_writes_are_refused() {
    let t = seeded().await;
    let ok = json!({ "kind": "followup", "text": "x" });
    assert_eq!(
        status_of(&t, "alpha", &with(json!({ "id": "X-1" }), &ok)).await,
        400
    );
    assert_eq!(
        status_of(&t, "alpha", &json!({ "kind": "shout", "text": "x" })).await,
        400
    );
    let long = json!({ "kind": "followup", "text": "x".repeat(300) });
    assert_eq!(status_of(&t, "alpha", &long).await, 400);
    assert_eq!(status_of(&t, "nope", &ok).await, 404);
    let put = json!({ "data": { "title": "x" } });
    let other = t.write("PUT", "/api/p/alpha/db/items/I-1", &put).await;
    assert_eq!((other.status, other.code().as_str()), (403, "read_only"));
}

#[tokio::test(flavor = "multi_thread")]
async fn a_post_waits_for_a_held_board_lock_and_applies_once_it_is_released() {
    let t = seeded().await;
    let lock = t.coord_home.join("alpha").join("board.lock");
    std::fs::write(&lock, "").expect("lock file");
    tokio::spawn(async move {
        tokio::time::sleep(Duration::from_millis(400)).await;
        let _ = std::fs::remove_file(lock);
    });
    let started = Instant::now();
    let r = post(
        &t,
        &json!({ "kind": "followup", "replyTo": "Q-1", "text": "After the lock" }),
    )
    .await;
    assert_eq!(r["status"], "applied");
    assert!(started.elapsed() >= Duration::from_millis(350));
}

#[tokio::test(flavor = "multi_thread")]
async fn a_proof_upload_is_kept_and_served() {
    let t = seeded().await;
    let upload = Request::new("POST", "/api/p/alpha/tests/T-1/proofs?caption=green")
        .header("Content-Type", "image/png")
        .body(png());
    let r = t.send(upload).await.json();
    let proof = json!({ "n": 1, "url": "/files/alpha/proofs/T-1-1.png", "caption": "green" });
    assert_eq!(r, json!({ "test": "T-1", "proof": proof }));
    let stored = doc(&t, "alpha", "tests", "T-1").await;
    assert_eq!(stored["proofs"][0]["url"], "/files/alpha/proofs/T-1-1.png");
    let img = t.get("/files/alpha/proofs/T-1-1.png").await;
    assert_eq!(img.status, 200);
    assert_eq!(img.header("content-type"), Some("image/png"));
    assert_eq!(img.body, png());
}

#[tokio::test(flavor = "multi_thread")]
async fn a_proof_that_is_not_an_image_type_is_refused() {
    let t = seeded().await;
    let text = Request::new("POST", "/api/p/alpha/tests/T-1/proofs")
        .header("Content-Type", "text/plain")
        .body("hello");
    assert_eq!(t.send(text).await.status, 415);
}

#[tokio::test(flavor = "multi_thread")]
async fn a_portfolio_send_is_delivered_to_the_project_board() {
    let t = seeded().await;
    let body = json!({ "project": "beta", "text": "Please review the API", "decision": true });
    let sent = t.write("POST", "/api/portfolio/send", &body).await.json();
    assert_eq!(sent["status"], "delivered", "{sent}");
    let sent_id = text_of(&sent, "id");
    let beta = t.board("beta");
    let delivered = beta["entries"]
        .as_object()
        .and_then(|e| e.values().find(|e| e["ref"] == sent_id))
        .expect("the entry on beta");
    assert_eq!(delivered["kind"], "decision");
    assert_eq!(delivered["status"], "pending");
    let stored = doc(&t, "_portfolio", "entries", &sent_id).await;
    assert_eq!(stored["status"], "delivered");
}

#[tokio::test(flavor = "multi_thread")]
async fn a_page_post_to_all_projects_is_delivered_to_each() {
    let t = seeded().await;
    let data = json!({ "kind": "followup", "author": "human", "project": "all", "text": "Hello leads", "createdAt": iso() });
    let page = page_put(&t, "_portfolio", &id(), &data).await.json();
    assert_eq!(page["status"], "delivered");
    let mut to: Vec<String> = page["entry"]["deliveredTo"]
        .as_array()
        .map(|a| {
            a.iter()
                .filter_map(|v| v.as_str().map(str::to_owned))
                .collect()
        })
        .unwrap_or_default();
    to.sort();
    assert_eq!(to, ["alpha", "beta"]);
}

#[tokio::test(flavor = "multi_thread")]
async fn a_legacy_portfolio_id_is_still_taken() {
    let t = seeded().await;
    let data =
        json!({ "kind": "followup", "author": "human", "project": "beta", "text": "Old page id" });
    let legacy = page_put(&t, "_portfolio", "O-7", &data).await.json();
    assert_eq!(legacy["status"], "delivered");
}

#[tokio::test(flavor = "multi_thread")]
async fn a_post_left_received_is_applied_at_the_next_boot() {
    let t = seeded().await;
    let rid = id();
    let body = json!({ "id": rid, "kind": "followup", "replyTo": "Q-1", "text": "Recovered", "author": "human", "status": "new", "via": "page", "createdAt": iso() });
    t.svc
        .db
        .with_conn(|c| {
            c.execute(
                "INSERT INTO posts(id, scope, kind, source, body, status, received_at) VALUES(?, 'beta', 'followup', 'page', ?, 'received', ?)",
                params![rid, body.to_string(), iso()],
            )
        })
        .expect("the received post");
    let (coord_home, dashboard_home) = t.stop(true).await;
    let t = TestServer::start(TestOptions {
        coord_home: Some(coord_home.clone()),
        dashboard_home: Some(dashboard_home.clone()),
        ..TestOptions::default()
    })
    .await;
    let (status, outcome): (String, Option<String>) = t
        .svc
        .db
        .with_conn(|c| {
            c.query_row(
                "SELECT status, outcome FROM posts WHERE id = ?",
                [rid.as_str()],
                |r| Ok((r.get(0)?, r.get(1)?)),
            )
        })
        .expect("the post row");
    assert_eq!(status, "applied", "{outcome:?}");
    let beta = t.board("beta");
    assert_eq!(beta["entries"][&rid]["text"], "Recovered");
    assert_eq!(beta["entries"]["Q-1"]["status"], "answered");
    t.stop(false).await;
    let _ = std::fs::remove_dir_all(coord_home);
    let _ = std::fs::remove_dir_all(dashboard_home);
}

fn board_files(t: &TestServer) -> Vec<Vec<u8>> {
    ["alpha", "beta"]
        .map(|k| std::fs::read(t.coord_home.join(k).join("board.json")).expect("board file"))
        .to_vec()
}

#[tokio::test(flavor = "multi_thread")]
async fn in_shadow_mode_every_post_is_read_only_and_no_board_changes() {
    let t = TestServer::start(TestOptions {
        shadow: true,
        seed: true,
        ..TestOptions::default()
    })
    .await;
    let before = board_files(&t);
    let followup = json!({ "kind": "followup", "replyTo": "Q-1", "text": "Shadow" });
    let entries = cli(&t, &followup).await;
    let data = with(
        json!({ "status": "new", "author": "human", "createdAt": iso() }),
        &followup,
    );
    let put = page_put(&t, "alpha", &id(), &data).await;
    let note = json!({ "project": "beta", "text": "Shadow" });
    let sent = t.write("POST", "/api/portfolio/send", &note).await;
    for reply in [entries, put, sent] {
        assert_eq!((reply.status, reply.code().as_str()), (403, "read_only"));
    }
    assert_eq!(board_files(&t), before);
}
