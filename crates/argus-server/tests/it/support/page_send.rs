//! Shared by the portfolio send specs: a send the way the Overlord page makes it, and a long brief to send.

use std::sync::atomic::{AtomicU32, Ordering};

use argus_testkit::{Request, TestServer};
use serde_json::{Value, json};

static COUNTER: AtomicU32 = AtomicU32::new(0);

pub fn post_id() -> String {
    let n = COUNTER.fetch_add(1, Ordering::SeqCst) + 1;
    format!("O-{}-{n:04}", argus_core::clock::now_ms())
}

/// A long text with lines, a list and markdown, the way the owner writes a brief.
pub fn brief(len: usize) -> String {
    let unit = "Plan:\n- first **bold** step\n- second step\n1. numbered\n";
    unit.repeat(len.div_ceil(50))[..len].trim().to_owned()
}

pub fn page_request(id: &str, data: &Value) -> Request {
    Request::new("PUT", &format!("/api/p/_portfolio/db/entries/{id}"))
        .json(&json!({ "data": data }))
}

/// A send the way the page makes it: a write of the `_portfolio` entries collection.
pub async fn from_page(t: &TestServer, extra: Value) -> Value {
    let mut data = json!({ "author": "human", "createdAt": argus_core::clock::now_iso() });
    data.as_object_mut()
        .expect("an object")
        .extend(extra.as_object().expect("an object").clone());
    let reply = t.send(t.page(page_request(&post_id(), &data))).await;
    assert_eq!(reply.status, 200, "{}", reply.text());
    reply.json()
}
