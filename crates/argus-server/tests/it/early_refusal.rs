//! A refusal sent before the body is read still reaches the client: the bad Host, the foreign
//! Origin and the wrong content type of the gate, and a handler that refuses a browser before it
//! reads anything. The body arrives with the head, after a pause, or from many clients at once;
//! every client reads its refusal, never a reset connection (the Node server's behaviour).
//! K4, K5: a refusal sent before the body is read reaches the client.
// test-tag: standard

use std::sync::Arc;
use std::time::Duration;

use argus_testkit::{Request, TestOptions, TestServer};
use serde_json::json;

const ROUNDS: usize = 10;
const CLIENTS: usize = 8;
const BATCH: &str = "/api/p/alpha/db/batch";
const CLEAR: &str = "/api/wake-faults/clear";

fn body() -> String {
    json!({ "ops": [], "pad": "x".repeat(64 * 1024) }).to_string()
}

/// The early refusals, each with the status it must answer.
fn refusals(t: &TestServer) -> Vec<(&'static str, Request, u16)> {
    let write = |path: &str| {
        Request::new("POST", path)
            .header("Content-Type", "application/json")
            .body(body())
    };
    let wrong_type = Request::new("POST", BATCH)
        .header("Content-Type", "text/plain")
        .body(body());
    vec![
        ("bad host", write(BATCH).header("Host", "evil.test"), 421),
        (
            "foreign origin",
            write(BATCH).header("Origin", "http://evil.test"),
            403,
        ),
        ("wrong type", wrong_type, 415),
        ("page clear", t.page(write(CLEAR)), 403),
    ]
}

async fn client(t: Arc<TestServer>, n: usize) -> Vec<String> {
    let mut misses = Vec::new();
    for round in 0..ROUNDS {
        for (name, req, status) in refusals(&t) {
            let gap = Duration::from_millis(((round + n) % 3 * 5) as u64);
            let got = t.send(req.paced(gap)).await.status;
            if got != status {
                misses.push(format!("{name} (gap {gap:?}): {got}, not {status}"));
            }
        }
    }
    misses
}

#[tokio::test(flavor = "multi_thread")]
async fn every_early_refusal_reaches_the_client() {
    let t = Arc::new(TestServer::start(TestOptions::default()).await);
    let runs: Vec<_> = (0..CLIENTS)
        .map(|n| tokio::spawn(client(Arc::clone(&t), n)))
        .collect();
    let mut misses = Vec::new();
    for run in runs {
        misses.extend(run.await.expect("client task"));
    }
    let sent = ROUNDS * CLIENTS * 4;
    let first: Vec<_> = misses.iter().take(4).collect();
    assert!(
        misses.is_empty(),
        "{} of {sent} refusals lost, first: {first:?}",
        misses.len()
    );
    if let Ok(t) = Arc::try_unwrap(t) {
        t.stop(false).await;
    }
}
