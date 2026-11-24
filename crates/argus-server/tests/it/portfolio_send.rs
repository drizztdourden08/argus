//! The Overlord page's send-down box has no working limit for the owner. A send from the page
//! (source page, author human) is held to the 50000-character safety net only, stored whole in
//! the portfolio, delivered whole to the lead's inbox and its board entry, and `coord inbox`
//! prints it whole; the wake line stays short and says where the full text is. The same text
//! from the CLI side (`coord portfolio send`, the API without a page) keeps the short limits.
//! Ported from `portfolio.longsend.test.mjs`; the two cases that run the Node coord CLI need
//! `ARGUS_COORD_CLI`.
//! K3, K4: the owner send of the Overlord page, whole to the lead, with the 50000 net and the short limits of the API.
// test-tag: standard

use crate::page_send::{brief, from_page, page_request, post_id};
use crate::wait::until;
use crate::wake_rig::{rig, wakes};
use argus_board::limits::OWNER_TEXT_MAX;
use argus_testkit::{Request, TestServer};
use serde_json::{Value, json};

fn lead_entry(t: &TestServer, key: &str, reference: &str) -> Value {
    let board = t.board(key);
    let entries = board["entries"].as_object().cloned().unwrap_or_default();
    entries
        .values()
        .find(|e| e["ref"] == reference)
        .cloned()
        .unwrap_or(Value::Null)
}

fn inbox_message(t: &TestServer, key: &str, reference: &str) -> Value {
    let inbox = t.board(key)["inbox"]["lead"]
        .as_array()
        .cloned()
        .unwrap_or_default();
    inbox
        .into_iter()
        .find(|m| m["ref"] == reference)
        .unwrap_or(Value::Null)
}

fn text_of(v: &Value) -> &str {
    v["text"].as_str().unwrap_or_default()
}

fn count(t: &TestServer, key: &str) -> usize {
    t.board(key)["entries"]
        .as_object()
        .map_or(0, serde_json::Map::len)
}

#[tokio::test(flavor = "multi_thread")]
async fn a_20000_character_decision_is_stored_whole_handed_to_the_lead_whole_and_the_wake_is_short()
{
    let r = rig().await;
    let t = &r.t;
    let text = brief(20000);
    let sent = from_page(
        t,
        json!({ "kind": "decision", "project": "alpha", "text": text }),
    )
    .await;
    assert_eq!(sent["status"], "delivered", "{sent}");
    assert_eq!(sent["entry"]["text"], text);
    let id = sent["id"].as_str().expect("an id");
    assert_eq!(
        text_of(&t.board("_portfolio")["entries"][id]).len(),
        text.len(),
        "the portfolio keeps the whole send"
    );
    let lead = lead_entry(t, "alpha", id);
    assert_eq!(
        text_of(&lead),
        text,
        "the lead board carries the whole text"
    );
    assert_eq!(lead["kind"], "decision");
    let message = inbox_message(t, "alpha", lead["id"].as_str().unwrap_or_default());
    assert!(
        text_of(&message).contains(&text),
        "the lead inbox message carries the whole text"
    );
    let wake = until("the wake row", || async {
        let rows = wakes(t, "alpha").await;
        rows.into_iter()
            .find(|w| w["reason"] == "portfolio-decision")
    })
    .await;
    let line = text_of(&wake);
    assert!(
        line.chars().count() < 300,
        "the wake line is {}",
        line.len()
    );
    assert!(
        line.ends_with(&format!("[full text in {id}: coord inbox]")),
        "{line}"
    );
    assert!(!line.contains('\n'));
}

#[tokio::test(flavor = "multi_thread")]
async fn a_note_to_all_projects_at_20000_characters_reaches_every_lead_whole() {
    let r = rig().await;
    let t = &r.t;
    let text = brief(20000);
    let sent = from_page(
        t,
        json!({ "kind": "followup", "project": "all", "text": text }),
    )
    .await;
    assert_eq!(sent["status"], "delivered", "{sent}");
    let mut to: Vec<String> = sent["entry"]["deliveredTo"]
        .as_array()
        .expect("deliveredTo")
        .iter()
        .filter_map(|k| k.as_str().map(str::to_owned))
        .collect();
    to.sort();
    assert_eq!(to, ["alpha", "beta"]);
    let id = sent["id"].as_str().expect("an id");
    for key in ["alpha", "beta"] {
        let lead = lead_entry(t, key, id);
        assert_eq!(text_of(&lead), text, "{key}");
        let message = inbox_message(t, key, lead["id"].as_str().unwrap_or_default());
        assert!(text_of(&message).contains(&text), "{key}");
    }
}

#[tokio::test(flavor = "multi_thread")]
async fn exactly_50000_characters_are_fine_one_more_gets_a_plain_sentence_and_nothing_is_kept() {
    let r = rig().await;
    let t = &r.t;
    let ok = from_page(
        t,
        json!({ "kind": "followup", "project": "alpha", "text": "y".repeat(OWNER_TEXT_MAX) }),
    )
    .await;
    assert_eq!(ok["status"], "delivered", "{ok}");
    let id = ok["id"].as_str().expect("an id");
    assert_eq!(
        text_of(&t.board("_portfolio")["entries"][id]).len(),
        OWNER_TEXT_MAX
    );
    let portfolio_before = count(t, "_portfolio");
    let lead_before = count(t, "alpha");
    let over = json!({
        "kind": "followup", "project": "alpha", "author": "human",
        "text": "x".repeat(OWNER_TEXT_MAX + 1),
    });
    let res = t.send(t.page(page_request(&post_id(), &over))).await;
    assert_eq!(res.status, 400, "{}", res.text());
    assert!(
        res.text().contains(
            "this text is 50001 characters and one post takes 50000 at most: shorten it or split it into two posts"
        ),
        "{}",
        res.text()
    );
    assert_eq!(count(t, "_portfolio"), portfolio_before);
    assert_eq!(count(t, "alpha"), lead_before);
}

#[tokio::test(flavor = "multi_thread")]
async fn the_api_without_a_page_keeps_its_limit_and_the_same_call_from_the_page_takes_it_whole() {
    let r = rig().await;
    let t = &r.t;
    let before = count(t, "_portfolio");
    let api = t
        .write(
            "POST",
            "/api/portfolio/send",
            &json!({ "project": "alpha", "text": "a".repeat(241) }),
        )
        .await;
    assert_eq!(api.status, 400, "{}", api.text());
    assert!(
        api.text().contains("chars, the limit is 240"),
        "{}",
        api.text()
    );
    assert_eq!(count(t, "_portfolio"), before);
    let request = Request::new("POST", "/api/portfolio/send")
        .json(&json!({ "project": "alpha", "text": "a".repeat(5000) }));
    let owner = t.send(t.page(request)).await;
    assert_eq!(owner.status, 200, "{}", owner.text());
}
