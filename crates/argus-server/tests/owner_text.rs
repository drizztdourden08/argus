//! The owner's free text has no working limit; the AI side keeps its short ones. A post from
//! the page (source page, author human) is held to the 50000-character safety net only, stored
//! whole and delivered whole to the thread's inbox. The same text from the CLI (what a model
//! writes) is still refused at the old limits.
mod posting;

use argus_board::limits::OWNER_TEXT_MAX;
use argus_testkit::TestServer;
use posting::{cli, doc, entry_count, inbox, page, page_reply, seeded};
use serde_json::{Value, json};

/// A long text with lines, a list and markdown, the way an owner writes feedback.
fn feedback(len: usize) -> String {
    let unit = "Steps: open the page.\n- first **bold** item\n- second item\n1. numbered\n";
    unit.repeat(len.div_ceil(40))[..len].trim().to_owned()
}

fn text_of(v: &Value, field: &str) -> String {
    v[field].as_str().unwrap_or_default().to_owned()
}

fn inbox_text(t: &TestServer, entry_id: &str) -> String {
    let message = inbox(t, "alpha", "ui")
        .into_iter()
        .find(|m| m["ref"] == entry_id)
        .expect("a message for the thread");
    text_of(&message, "text")
}

fn is_limit_message(body: &str) -> bool {
    body.split_once("chars, the limit is ")
        .is_some_and(|(_, rest)| rest.starts_with(|c: char| c.is_ascii_digit()))
}

#[test]
fn the_safety_net_is_50000_characters() {
    assert_eq!(OWNER_TEXT_MAX, 50000);
}

#[tokio::test(flavor = "multi_thread")]
async fn a_20000_character_result_note_is_stored_whole_and_reaches_the_inbox_whole() {
    let t = seeded().await;
    let text = feedback(20000);
    let r = page(
        &t,
        &json!({ "kind": "result", "outcome": "fail", "replyTo": "T-1", "text": text }),
    )
    .await;
    assert_eq!(r["status"], "applied", "{r}");
    assert_eq!(r["entry"]["text"], text);
    assert_eq!(r["entry"]["via"], "page");
    let id = text_of(&r, "id");
    let b = t.board("alpha");
    assert_eq!(text_of(&b["entries"][&id], "text").len(), text.len());
    assert_eq!(b["tests"]["T-1"]["result"]["text"], text);
    let last = b["tests"]["T-1"]["runs"].as_array().and_then(|r| r.last());
    assert_eq!(last.map(|run| run["text"].clone()), Some(json!(text)));
    assert!(
        inbox_text(&t, &id).contains(&text),
        "the inbox carries the whole note"
    );
    assert_eq!(doc(&t, "alpha", "entries", &id).await["text"], text);
}

#[tokio::test(flavor = "multi_thread")]
async fn answers_and_decisions_from_the_page_are_not_cut_at_240_or_200() {
    let t = seeded().await;
    let long = feedback(5000);
    for kind in ["followup", "decision"] {
        let r = page(&t, &json!({ "kind": kind, "replyTo": "Q-1", "text": long })).await;
        let expected = if kind == "decision" {
            "pending"
        } else {
            "applied"
        };
        assert_eq!(r["status"], expected, "{r}");
        let id = text_of(&r, "id");
        assert_eq!(t.board("alpha")["entries"][&id]["text"], long, "{kind}");
        assert!(inbox_text(&t, &id).contains(&long), "{kind}");
    }
}

#[tokio::test(flavor = "multi_thread")]
async fn the_owner_may_lengthen_his_own_decision_but_never_edits_a_threads_text() {
    let t = seeded().await;
    let d = page(
        &t,
        &json!({ "kind": "decision", "text": "short", "item": "I-1" }),
    )
    .await;
    let id = text_of(&d, "id");
    let longer = feedback(3000);
    let rev = t.board("alpha")["entries"][&id]["rev"].clone();
    let target = json!({ "c": "entries", "id": id });
    let e = page(
        &t,
        &json!({ "kind": "edit", "target": target, "set": { "text": longer }, "prev": { "text": "short" }, "baseRev": rev }),
    )
    .await;
    assert_eq!(e["status"], "applied", "{e}");
    assert_eq!(t.board("alpha")["entries"][&id]["text"], longer);
    let own = json!({ "c": "entries", "id": "Q-1" });
    let q = page(
        &t,
        &json!({ "kind": "edit", "target": own, "set": { "text": "x" } }),
    )
    .await;
    assert_ne!(
        q["status"], "applied",
        "the owner never edits a thread's own wording"
    );
}

#[tokio::test(flavor = "multi_thread")]
async fn over_50000_characters_the_page_gets_a_plain_sentence_and_nothing_is_stored() {
    let t = seeded().await;
    let entries = entry_count(&t, "alpha");
    let huge = "x".repeat(OWNER_TEXT_MAX + 1);
    let res = page_reply(
        &t,
        &json!({ "kind": "result", "outcome": "fail", "replyTo": "T-1", "text": huge }),
    )
    .await;
    assert_eq!(res.status, 400);
    let message = text_of(&res.json()["error"], "message");
    assert!(
        message.contains("this text is 50001 characters and one post takes 50000 at most: shorten it or split it into two posts"),
        "{message}"
    );
    assert_eq!(
        entry_count(&t, "alpha"),
        entries,
        "nothing reached the board"
    );
}

#[tokio::test(flavor = "multi_thread")]
async fn exactly_50000_characters_is_accepted_whole() {
    let t = seeded().await;
    let ok = page(
        &t,
        &json!({ "kind": "followup", "replyTo": "N-1", "text": "y".repeat(OWNER_TEXT_MAX) }),
    )
    .await;
    assert_eq!(
        ok["status"],
        "applied",
        "{}",
        ok.to_string().chars().take(200).collect::<String>()
    );
    let id = text_of(&ok, "id");
    assert_eq!(
        text_of(&t.board("alpha")["entries"][&id], "text").len(),
        OWNER_TEXT_MAX
    );
}

#[tokio::test(flavor = "multi_thread")]
async fn the_ai_side_keeps_its_limits_from_the_cli_source() {
    let t = seeded().await;
    let entries = entry_count(&t, "alpha");
    for data in [
        json!({ "kind": "followup", "replyTo": "Q-1", "text": "a".repeat(241) }),
        json!({ "kind": "decision", "replyTo": "Q-1", "text": "a".repeat(201) }),
        json!({ "kind": "result", "outcome": "fail", "replyTo": "T-1", "text": "a".repeat(301) }),
    ] {
        let res = cli(&t, &data).await;
        assert_eq!(res.status, 400, "{}", data["kind"]);
        assert!(is_limit_message(&res.text()), "{}", res.text());
    }
    assert_eq!(entry_count(&t, "alpha"), entries);
    let under = json!({ "kind": "followup", "replyTo": "Q-1", "text": "a".repeat(240) });
    assert_eq!(cli(&t, &under).await.status, 200);
}
