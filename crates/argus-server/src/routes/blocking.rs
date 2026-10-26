//! Calls into a service that blocks (a database, the platform) run off the async workers.

use argus_core::ApiError;

/// Runs `f` on a blocking thread and answers what it returned.
///
/// # Errors
/// 500 when the thread panicked.
pub async fn blocking<T: Send + 'static>(
    f: impl FnOnce() -> T + Send + 'static,
) -> Result<T, ApiError> {
    tokio::task::spawn_blocking(f)
        .await
        .map_err(|_| ApiError::internal())
}
