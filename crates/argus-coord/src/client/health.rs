//! Is the dashboard answering? One `GET /api/health` with a short limit.

use std::time::Duration;

use super::http::Client;

const HEALTH_TIMEOUT: Duration = Duration::from_secs(2);

/// Whether the server answers its health check with a success status.
#[must_use]
pub fn healthy(client: &Client) -> bool {
    client
        .request_within(HEALTH_TIMEOUT, "GET", "/api/health", None)
        .is_ok()
}
