//! Shared by the Overlord route specs: the owner's page asks, the reply must be 200, the body comes back.

use argus_testkit::{Request, TestServer};
use serde_json::Value;

/// The JSON body of a request made from the owner's page; fails the spec unless the status is 200.
pub async fn page_body(t: &TestServer, request: Request) -> Value {
    let reply = t.send(t.page(request)).await;
    assert_eq!(reply.status, 200, "{}", reply.text());
    reply.json()
}
