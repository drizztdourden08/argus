//! Waiting for what a server does on its own, and reading what it woke.

use std::future::Future;
use std::time::Duration;

use argus_testkit::TestServer;
use serde_json::Value;

/// Polls until `f` answers something (up to twenty seconds).
pub async fn until<T, Fut: Future<Output = Option<T>>>(
    what: &str,
    mut f: impl FnMut() -> Fut,
) -> T {
    for _ in 0..400 {
        if let Some(found) = f().await {
            return found;
        }
        tokio::time::sleep(Duration::from_millis(50)).await;
    }
    panic!("timed out waiting for {what}");
}

/// The wake rows of a project that carry `reason`.
pub async fn wakes_of(t: &TestServer, key: &str, reason: &str) -> Vec<Value> {
    let body = t.get(&format!("/api/p/{key}/wakes")).await.json();
    let rows = body["wakes"].as_array().cloned().unwrap_or_default();
    rows.into_iter().filter(|w| w["reason"] == reason).collect()
}
