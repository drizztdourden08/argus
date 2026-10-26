//! Waiting for what a server does on its own.

use std::future::Future;
use std::time::Duration;

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
