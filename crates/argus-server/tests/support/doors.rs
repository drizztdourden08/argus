//! What the door tests share: the alias builder and the write check of a host-named door.

use std::net::SocketAddr;

use argus_server::security::Alias;
use argus_testkit::{Request, request_at};
use serde_json::json;

/// An alias as the host rules parse it.
pub fn alias(key: &str, thread: Option<&str>, reserved: bool) -> Alias {
    Alias {
        key: key.to_owned(),
        thread: thread.map(str::to_owned),
        reserved,
    }
}

/// A write through the door at `addr` for `host` is read-only with that host as its origin and
/// forbidden with any other.
pub async fn assert_write_needs_own_origin(addr: SocketAddr, host: &str) {
    let put = |origin: String| {
        Request::new("PUT", "/api/p/alpha/db/items/I-1")
            .header("Host", host)
            .header("Origin", &origin)
            .json(&json!({ "data": {} }))
    };
    let own = request_at(addr, &put(format!("http://{host}"))).await;
    assert_eq!((own.status, own.code().as_str()), (403, "read_only"));
    let cross = request_at(addr, &put("http://evil.test".to_owned())).await;
    assert_eq!((cross.status, cross.code().as_str()), (403, "forbidden"));
}
