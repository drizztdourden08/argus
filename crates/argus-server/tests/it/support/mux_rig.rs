//! The rig of the mux tests: a server with two boards of docs, an open mux stream and its POSTs.

use argus_core::{Coll, DocChange};
use argus_testkit::{Event, SseReader, TestOptions, TestServer};
use serde_json::{Value, json};

pub const WAIT: u64 = 2000;

/// A server whose `alpha` scope holds two items and a meta doc, and `beta` one item and a meta
/// doc.
pub async fn server() -> TestServer {
    let t = TestServer::start(TestOptions {
        sync: false,
        ..TestOptions::default()
    })
    .await;
    let item = |id: &str| DocChange::set(Coll::Items, id, json!({ "id": id }));
    let meta = |scope: &str| DocChange::set(Coll::Meta, "board", json!({ "project": scope }));
    apply(&t, "alpha", &[item("I-1"), item("I-2"), meta("alpha")]);
    apply(&t, "beta", &[item("B-1"), meta("beta")]);
    t
}

pub fn apply(t: &TestServer, scope: &str, changes: &[DocChange]) -> i64 {
    t.svc.db.docs().apply(scope, changes).expect("apply").seq
}

pub fn seq(t: &TestServer) -> i64 {
    t.svc.db.docs().seq().expect("seq")
}

pub fn set_item(id: &str) -> DocChange {
    DocChange::set(Coll::Items, id, json!({ "id": id }))
}

pub fn names(got: &[Event]) -> Vec<&str> {
    got.iter().map(|e| e.event.as_str()).collect()
}

pub fn sub(scope: &str, colls: &[&str]) -> Value {
    json!({ "op": "sub", "scope": scope, "colls": colls })
}

pub fn sub_since(scope: &str, colls: &[&str], since: i64) -> Value {
    json!({ "op": "sub", "scope": scope, "colls": colls, "since": since })
}

/// An open mux stream and the id it was given.
pub struct Mux {
    pub s: SseReader,
    pub conn: String,
}

impl Mux {
    /// Opens the stream and reads its `hello`.
    pub async fn open(t: &TestServer) -> Mux {
        let s = t.sse("/api/mux").await;
        assert_eq!(s.status, 200);
        let hello = s.next(Some("hello"), WAIT).await.expect("hello");
        let conn = hello.data["conn"].as_str().expect("a conn id").to_owned();
        Mux { s, conn }
    }

    pub fn path(&self) -> String {
        format!("/api/mux/{}", self.conn)
    }

    /// Posts the ops and answers the results of a 200.
    pub async fn ops(&self, t: &TestServer, ops: Value) -> Vec<Value> {
        let r = t.write("POST", &self.path(), &json!({ "ops": ops })).await;
        assert_eq!(r.status, 200, "{}", r.text());
        r.json()["results"].as_array().expect("results").clone()
    }

    /// Posts one `sub` and reads up to its `ready`.
    pub async fn sub(&self, t: &TestServer, op: Value) -> Vec<Event> {
        let results = self.ops(t, json!([op])).await;
        assert_eq!(results, [json!({ "ok": true })]);
        self.until_ready().await
    }

    /// The events up to and including the next `ready`.
    pub async fn until_ready(&self) -> Vec<Event> {
        let mut got = Vec::new();
        loop {
            let e = self
                .s
                .next(None, WAIT)
                .await
                .expect("a ready event arrives");
            let done = e.event == "ready";
            got.push(e);
            if done {
                return got;
            }
        }
    }
}
