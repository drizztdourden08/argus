// test-tag: critical
//! `GET/POST /api/notify/auto`: a partial patch changes only what it sends and leaves the toast and
//! sound settings alone; only the owner's page may change them.

use argus_testkit::{Reply, Request, TestOptions, TestServer};
use serde_json::{Value, json};

async fn as_page(t: &TestServer, body: &Value) -> Reply {
    t.send(t.page(Request::new("POST", "/api/notify/auto").json(body)))
        .await
}

#[tokio::test(flavor = "multi_thread")]
async fn a_partial_patch_changes_only_what_it_sends_and_needs_the_page_token() {
    let t = TestServer::start(TestOptions::default()).await;
    let before = t.get("/api/notify/settings").await.json();
    let first = t.get("/api/notify/auto").await.json();
    assert_eq!(first["os"], true);
    assert_eq!(first["auto"]["ask"], true);

    let denied = t
        .write("POST", "/api/notify/auto", &json!({ "os": false }))
        .await;
    assert_eq!(denied.status, 403);

    let ok = as_page(&t, &json!({ "auto": { "ask": false } })).await;
    assert_eq!(ok.status, 200);
    let body = ok.json();
    assert_eq!(body["auto"]["ask"], false);
    assert_eq!(body["auto"]["handed"], true);
    assert_eq!(body["os"], true);
    assert_eq!(t.get("/api/notify/auto").await.json(), body);

    let after = t.get("/api/notify/settings").await.json();
    assert_eq!(after["toast"], before["toast"]);
    assert_eq!(after["sound"], before["sound"]);

    for bad in [json!({ "loud": true }), json!({ "auto": { "x": true } })] {
        assert_eq!(as_page(&t, &bad).await.status, 400, "{bad}");
    }
}
