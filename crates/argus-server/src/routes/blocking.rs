//! Work that blocks (a lock, the database, a board file) runs off the runtime's workers.

use argus_core::ApiError;

/// Runs `f` on a blocking thread and answers what it returns.
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
