// test-tag: critical
//! The inbox side of a dependency request: the one `dep` message the assignee thread reads (K2)
//! and the acknowledgement that read leaves on the ledger, written once and only for an open
//! request of that project (K3).

use argus_board::deps::ack;
use serde_json::{Value, json};

use crate::deps::{ask_in, asked, asking, cancel_as, inbox_texts, ledger, read, world};

fn messages(home: &std::path::Path, project: &str, thread: &str) -> Vec<Value> {
    read(home, project)["inbox"][thread]
        .as_array()
        .cloned()
        .unwrap_or_default()
}

#[test]
fn asking_queues_one_dep_message_for_the_thread_the_wake_reaches() {
    let home = world(&["alpha", "beta"]);
    let h = home.path();
    let mut req = asking("alpha", "I-1", "beta");
    req.to = Some("dev".to_owned());
    req.ac = vec!["Hook is public".to_owned(), "Docs updated".to_owned()];
    ask_in(h, &req).expect("asked");
    assert!(messages(h, "beta", "coordinator").is_empty());
    let sent = messages(h, "beta", "dev");
    assert_eq!(sent.len(), 1);
    assert_eq!(sent[0]["kind"], "dep");
    assert_eq!(sent[0]["ref"], "D-1");
    assert_eq!(sent[0]["from"], "alpha/dev");
    assert_eq!(
        sent[0]["text"],
        json!(
            "dep D-1: alpha/I-1 needs your I-3 \"Expose the save hook\" \
             | acceptance: Hook is public; Docs updated \
             | start it: coord item status I-3 doing, or answer: \
             coord dep close D-1 --as blocked --reason \"...\""
        )
    );
}

#[test]
fn ack_sets_acked_at_once_only_for_an_open_dep_of_that_project() {
    let home = world(&["alpha", "beta"]);
    let h = home.path();
    asked(h, "alpha", "I-1", "beta");
    assert!(!ack(h, "D-1", "alpha", "alpha lead").expect("ack"));
    assert!(!ack(h, "D-9", "beta", "beta lead").expect("ack"));
    assert!(ledger(h, "D-1")["ackedAt"].is_null());
    assert!(ack(h, "D-1", "beta", "beta lead").expect("ack"));
    let first = ledger(h, "D-1")["ackedAt"].clone();
    assert!(first.is_string());
    assert_eq!(ledger(h, "D-1")["ackedBy"], "beta lead");
    assert!(!ack(h, "D-1", "beta", "beta/dev").expect("ack"));
    assert_eq!(ledger(h, "D-1")["ackedAt"], first);
    assert_eq!(ledger(h, "D-1")["ackedBy"], "beta lead");
    assert_eq!(inbox_texts(h, "beta", "coordinator").len(), 1);

    let closed = world(&["alpha", "beta"]);
    let c = closed.path();
    asked(c, "alpha", "I-1", "beta");
    cancel_as(c, "alpha", None).expect("cancelled");
    assert!(!ack(c, "D-1", "beta", "beta lead").expect("ack"));
    assert!(ledger(c, "D-1")["ackedAt"].is_null());
}
