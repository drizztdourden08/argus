//! Ports `lead.verbs.test.mjs`: `coord lead <project> "<text>"` against temp boards: delivery to
//! the target lead's inbox, the sender label, the log line, the refusals (own project, unknown
//! project, no board, not in a checkout, kind, text limit), no thread needed on the other board,
//! a dashboard that is down never failing the verb, and a running dashboard waking the target
//! lead (a `*` wake-stream listener hears it).

mod common;
mod remote;

use std::path::{Path, PathBuf};

use common::{Home, coord};
use remote::{Rig, lead_board, make_checkout, ok, read_board_at, refused};
use serde_json::{Value, json};

/// Two projects with a lead each, a checkout each, and a registry that also names `ghost`.
fn seed(coord_home: &Path, base: &Path) {
    for key in ["pixelarttool", "brock"] {
        let checkout = make_checkout(base, key);
        argus_testkit::write_board(coord_home, key, &lead_board(key, &checkout));
    }
    std::fs::create_dir_all(base.join("bare")).expect("bare folder");
}

fn world() -> (Home, PathBuf) {
    let home = Home::new();
    seed(&home.coord_home(), home.path());
    let from = home.path().to_path_buf();
    for argv in [
        &["portfolio", "init"][..],
        &["portfolio", "add", "ghost", "--name", "Ghost"],
    ] {
        ok(&coord(argv, home.env(), &from));
    }
    (home, from)
}

fn board(home: &Home, key: &str) -> Value {
    read_board_at(&home.coord_home(), key)
}

fn lead(home: &Home, from: &str, argv: &[&str]) -> common::Ran {
    let cwd = home.path().join(from);
    coord(argv, home.env(), &cwd)
}

#[test]
fn it_delivers_to_the_target_lead_from_a_checkout_with_no_thread_on_the_other_board() {
    let (home, _) = world();
    let text = "rule no-magic-numbers fires on tile sizes, I think it is wrong";
    let ran = lead(&home, "checkout-pixelarttool", &["lead", "brock", text]);
    assert_eq!(ok(&ran), format!("queued for brock lead: {text}\n"));
    let b = board(&home, "brock");
    let inbox = b["inbox"]["lead"].as_array().expect("the lead inbox");
    assert_eq!(inbox.len(), 1);
    let m = &inbox[0];
    assert_eq!(
        (m["from"].as_str(), m["kind"].as_str()),
        (Some("pixelarttool lead"), Some("followup"))
    );
    let said = m["text"].as_str().expect("text");
    assert!(said.starts_with("P-"), "{said}");
    assert!(
        said.contains(" followup from pixelarttool lead: rule no-magic-numbers fires"),
        "{said}"
    );
    assert_eq!(m["deliveredAt"], Value::Null);
    let entry = &b["entries"][m["ref"].as_str().expect("ref")];
    let seen = json!([
        entry["kind"],
        entry["author"],
        entry["thread"],
        entry["status"]
    ]);
    assert_eq!(
        seen,
        json!(["followup", "pixelarttool lead", "lead", "info"])
    );
    let log = b["log"]["leads"]["entries"].as_array().expect("log");
    assert_eq!(log.len(), 1);
    assert!(
        log[0]["text"]
            .as_str()
            .is_some_and(|t| t.contains("note from pixelarttool lead: rule no-magic"))
    );
    assert!(
        board(&home, "pixelarttool")["inbox"]["lead"].is_null(),
        "the sender board is untouched"
    );
}

#[test]
fn a_decision_carries_the_pick_hint_and_the_other_direction_works_too() {
    let (home, _) = world();
    let text = "please freeze the lint config until Friday";
    ok(&lead(
        &home,
        "checkout-brock",
        &["lead", "pixelarttool", text, "--kind", "decision"],
    ));
    let m = board(&home, "pixelarttool")["inbox"]["lead"][0].clone();
    assert_eq!(
        (m["from"].as_str(), m["kind"].as_str()),
        (Some("brock lead"), Some("decision"))
    );
    let said = m["text"].as_str().expect("text");
    assert!(
        said.contains("decision from brock lead: please freeze"),
        "{said}"
    );
    assert!(said.contains(" Run coord entry pick P-"), "{said}");
    assert!(said.ends_with(" when you start on it."), "{said}");
    let shout = lead(
        &home,
        "checkout-pixelarttool",
        &["lead", "brock", "x", "--kind", "shout"],
    );
    refused(&shout, 1, "--kind must be note or decision");
}

#[test]
fn it_refuses_the_own_project_an_unknown_project_one_with_no_board_and_a_call_outside_a_checkout() {
    let (home, _) = world();
    let px = "checkout-pixelarttool";
    let cases = [
        (
            px,
            vec!["lead", "pixelarttool", "to myself"],
            "pixelarttool is your own project: use coord inbox send",
        ),
        (
            px,
            vec!["lead", "nowhere", "hello"],
            "unknown project \"nowhere\"",
        ),
        (
            px,
            vec!["lead", "ghost", "hello"],
            "project \"ghost\" has no board yet",
        ),
        (
            "bare",
            vec!["lead", "brock", "hello"],
            "must run inside a project checkout",
        ),
    ];
    for (from, argv, text) in cases {
        refused(&lead(&home, from, &argv), 1, text);
    }
    assert!(
        board(&home, "brock")["inbox"]["lead"].is_null(),
        "nothing was delivered"
    );
}

#[test]
fn the_text_is_required_and_limited_to_240_characters() {
    let (home, _) = world();
    let px = "checkout-pixelarttool";
    refused(
        &lead(&home, px, &["lead", "brock"]),
        1,
        "lead text is required",
    );
    let long = "x".repeat(241);
    refused(
        &lead(&home, px, &["lead", "brock", &long]),
        1,
        "lead is 241 chars, the limit is 240",
    );
    ok(&lead(&home, px, &["lead", "brock", &"y".repeat(240)]));
    assert_eq!(
        board(&home, "brock")["inbox"]["lead"]
            .as_array()
            .map(Vec::len),
        Some(1)
    );
}

#[test]
fn project_names_the_sender_when_the_cwd_is_not_a_checkout() {
    let (home, _) = world();
    let ran = lead(
        &home,
        "bare",
        &[
            "lead",
            "brock",
            "from elsewhere",
            "--project",
            "pixelarttool",
        ],
    );
    assert!(ok(&ran).starts_with("queued for brock lead: from elsewhere"));
    assert_eq!(
        board(&home, "brock")["inbox"]["lead"][0]["from"],
        "pixelarttool lead"
    );
}

#[tokio::test(flavor = "multi_thread")]
async fn with_the_dashboard_running_the_target_lead_is_woken_with_the_sender_and_the_first_100_characters()
 {
    let rig = Rig::start(json!({ "wake": { "windowMs": 100 } }), seed).await;
    for argv in [
        &["portfolio", "init"][..],
        &["portfolio", "add", "ghost", "--name", "Ghost"],
    ] {
        ok(&rig.run(argv));
    }
    let star = rig.t.sse("/api/p/brock/wakes/stream?thread=*").await;
    star.next(Some("ready"), 5000).await.expect("ready");
    let long = "z".repeat(120);
    let cwd = rig.dir.path().join("checkout-pixelarttool");
    let ran = rig.run_in(&["lead", "brock", &long, "--kind", "decision"], None, &cwd);
    assert_eq!(ok(&ran), format!("queued for brock lead: {long}\n"));
    let heard = star
        .next(Some("wake"), 5000)
        .await
        .expect("the lead was woken");
    assert_eq!(heard.data["thread"], "lead");
    assert_eq!(
        heard.data["lines"],
        json!([format!("pixelarttool lead: {}", "z".repeat(100))])
    );
    let rows = rig.t.get("/api/p/brock/wakes").await.json();
    let woke: Vec<&Value> = rows["wakes"]
        .as_array()
        .expect("wakes")
        .iter()
        .filter(|w| w["reason"] == "lead-message")
        .collect();
    assert_eq!(woke.len(), 1);
    let note = rig.run_in(&["lead", "brock", "fyi: released 1.2"], None, &cwd);
    assert_eq!(ok(&note), "queued for brock lead: fyi: released 1.2\n");
    let rows = rig.t.get("/api/p/brock/wakes").await.json();
    let info: Vec<&Value> = rows["wakes"]
        .as_array()
        .expect("wakes")
        .iter()
        .filter(|w| w["reason"] == "lead-info")
        .collect();
    assert_eq!(info.len(), 1, "a note is information only");
    assert_eq!(info[0]["status"], "passive", "kept off the listener");
    star.close();
    rig.stop().await;
}
