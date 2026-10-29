//! Is the dashboard answering? One `GET /api/health` with a short limit.

use std::time::Duration;

use argus_core::Paths;
use serde_json::Value;

use super::config::{DashboardConfig, dashboard_config, pid_port};
use super::http::Client;

const HEALTH_TIMEOUT: Duration = Duration::from_secs(2);
/// A hook text waits this long at most for the health answer that picks its links.
const LINK_HEALTH_TIMEOUT: Duration = Duration::from_secs(1);

/// Whether the server answers its health check with a success status.
#[must_use]
pub fn healthy(client: &Client) -> bool {
    client
        .request_within(HEALTH_TIMEOUT, "GET", "/api/health", None)
        .is_ok()
}

/// The port and the health answer of the server that wrote `server.pid`; `Null` for the answer
/// when it does not give one quickly. `None` without a pid file: no server is asked then.
#[must_use]
pub fn running_health(paths: &Paths) -> Option<(u16, Value)> {
    let port = pid_port(paths)?;
    let config = dashboard_config(paths).unwrap_or(DashboardConfig {
        port,
        token: None,
        install_dir: None,
    });
    let client = Client::new(&DashboardConfig { port, ..config });
    let health = client
        .request_within(LINK_HEALTH_TIMEOUT, "GET", "/api/health", None)
        .unwrap_or(Value::Null);
    Some((port, health))
}
