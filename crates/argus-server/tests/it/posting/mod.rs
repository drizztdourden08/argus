//! Helpers shared by the owner post tests: unique post ids, a seeded server, and the calls a
//! page (a `PUT` of the entries collection) and the CLI (`POST /api/p/:key/entries`) make.
use argus_core::clock::now_iso;
pub use argus_sync::new_post_id as id;
use argus_testkit::{Reply, Request, TestOptions, TestServer};
use serde_json::{Value, json};

/// The current time as an ISO string.
pub fn iso() -> String {
    now_iso()
}

/// A server on the fixture boards.
pub async fn seeded() -> TestServer {
    TestServer::start(TestOptions {
        seed: true,
        ..TestOptions::default()
    })
    .await
}

/// `base` with the fields of `extra` set over it.
pub fn with(mut base: Value, extra: &Value) -> Value {
    if let (Some(b), Some(e)) = (base.as_object_mut(), extra.as_object()) {
        b.extend(e.clone());
    }
    base
}

/// A post body: a fresh id and the time, then `extra`.
pub fn entry(extra: &Value) -> Value {
    with(json!({ "id": id(), "createdAt": iso() }), extra)
}

/// `POST /api/p/<key>/entries` with the CLI's headers.
pub async fn cli_to(t: &TestServer, key: &str, body: &Value) -> Reply {
    t.write("POST", &format!("/api/p/{key}/entries"), body)
        .await
}

/// A CLI post to alpha with a fresh id; the answer's JSON.
pub async fn post(t: &TestServer, extra: &Value) -> Value {
    cli_to(t, "alpha", &entry(extra)).await.json()
}

/// A CLI post to alpha with a fresh id; the whole reply.
pub async fn cli(t: &TestServer, extra: &Value) -> Reply {
    cli_to(t, "alpha", &entry(extra)).await
}

/// The `data` of a doc, read through the page's own collection.
pub async fn doc(t: &TestServer, key: &str, coll: &str, id: &str) -> Value {
    t.get(&format!("/api/p/{key}/db/{coll}/{id}")).await.json()["data"].clone()
}

/// A write of one entries doc the way the browser page makes it (source `page`).
pub async fn page_put(t: &TestServer, scope: &str, id: &str, data: &Value) -> Reply {
    let req = Request::new("PUT", &format!("/api/p/{scope}/db/entries/{id}"))
        .json(&json!({ "data": data }));
    t.send(t.page(req)).await
}

/// A page post to alpha: new id, owner author, `new` status, then `extra`; the reply.
pub async fn page_reply(t: &TestServer, extra: &Value) -> Reply {
    let data = with(
        json!({ "status": "new", "author": "human", "createdAt": iso() }),
        extra,
    );
    page_put(t, "alpha", &id(), &data).await
}

/// A page post to alpha; the answer's JSON.
pub async fn page(t: &TestServer, extra: &Value) -> Value {
    page_reply(t, extra).await.json()
}

/// The messages in a thread's inbox on a board.
pub fn inbox(t: &TestServer, key: &str, thread: &str) -> Vec<Value> {
    t.board(key)["inbox"][thread]
        .as_array()
        .cloned()
        .unwrap_or_default()
}

/// How many entries a board holds.
pub fn entry_count(t: &TestServer, key: &str) -> usize {
    t.board(key)["entries"]
        .as_object()
        .map_or(0, serde_json::Map::len)
}
