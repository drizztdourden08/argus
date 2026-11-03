//! K1, S3: the buffer limit of one page stream client, with no server: a client a megabyte behind
//! is dropped and forgotten, and the opening snapshot frames do not count toward the limit.
// test-tag: critical

use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::Duration;

use argus_server::sse::{SSE_BUFFER_LIMIT, frame, open_stream};
use http_body_util::BodyExt;
use serde_json::json;

#[tokio::test(flavor = "multi_thread")]
async fn a_client_that_falls_a_megabyte_behind_is_dropped() {
    let open = Arc::new(AtomicUsize::new(0));
    let (sender, res) = open_stream(Duration::from_secs(3600), &open, None);
    assert_eq!(open.load(Ordering::SeqCst), 1);
    assert!(sender.event("a", &json!({ "n": 1 }), Some(5)));
    let first = frame("a", &json!({ "n": 1 }), Some(5));
    assert_eq!(first, "id: 5\nevent: a\ndata: {\"n\":1}\n\n");
    let chunk = json!({ "pad": "x".repeat(64 * 1024) });
    let mut sent = 0;
    while sender.event("a", &chunk, None) {
        sent += 1;
        assert!(
            sent < 32,
            "the limit of {SSE_BUFFER_LIMIT} bytes must stop it"
        );
    }
    assert!((10..=16).contains(&sent), "{sent} chunks fit");
    assert!(
        !sender.event("a", &json!({ "n": 3 }), None),
        "nothing after close"
    );
    let end = tokio::time::timeout(Duration::from_secs(2), res.into_body().collect());
    let bytes = end.await.expect("the body ends").expect("body").to_bytes();
    assert!(bytes.starts_with(first.as_bytes()));
    assert_eq!(open.load(Ordering::SeqCst), 0, "the stream is forgotten");
}

#[tokio::test(flavor = "multi_thread")]
async fn opening_frames_do_not_count_but_a_live_backlog_over_a_megabyte_still_drops() {
    let open = Arc::new(AtomicUsize::new(0));
    let (sender, res) = open_stream(Duration::from_secs(3600), &open, None);
    let chunk = json!({ "pad": "x".repeat(64 * 1024) });
    for _ in 0..32 {
        assert!(
            sender.opening_event("snapshot", &chunk, None),
            "2 MB of opening frames"
        );
    }
    assert!(
        sender.event("changes", &json!({ "n": 1 }), None),
        "a live frame after the snapshot"
    );
    let mut sent = 0;
    while sender.event("changes", &chunk, None) {
        sent += 1;
        assert!(
            sent < 32,
            "the live backlog must hit the limit of {SSE_BUFFER_LIMIT} bytes"
        );
    }
    assert!((10..=16).contains(&sent), "{sent} live chunks fit");
    let end = tokio::time::timeout(Duration::from_secs(2), res.into_body().collect());
    let bytes = end.await.expect("the body ends").expect("body").to_bytes();
    assert!(
        bytes.len() > 2 * 1024 * 1024,
        "the opening frames were all sent"
    );
}
