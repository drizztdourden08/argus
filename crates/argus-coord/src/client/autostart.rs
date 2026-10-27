//! The client of a dashboard that may be down: it answers at once when the server is healthy,
//! else it starts the dashboard (hidden, detached) and waits up to 8 seconds
//! (`COORD_DASHBOARD_WAIT_MS`) for it. A server that will not come up is
//! [`ClientError::Down`], the error the `run` verbs turn into exit 3.
//!
//! The dashboard is started through the tray on Windows, as the hooks do, so the icon is there
//! whenever the server runs; what exactly is run is decided in [`super::launch`].

use std::thread::sleep;
use std::time::{Duration, Instant};

use argus_core::Paths;

use super::config::dashboard_config;
use super::health::healthy;
use super::http::{Client, ClientError};
use super::launch::{plan, spawn};
use crate::cli::env::Env;

const WAIT_DEFAULT_MS: u64 = 8000;
const POLL: Duration = Duration::from_millis(200);

fn ready(paths: &Paths) -> Option<Client> {
    let client = Client::new(&dashboard_config(paths)?);
    healthy(&client).then_some(client)
}

fn wait_ms(env: &Env) -> u64 {
    env.get("COORD_DASHBOARD_WAIT_MS")
        .and_then(|v| v.trim().parse::<u64>().ok())
        .filter(|ms| *ms > 0)
        .unwrap_or(WAIT_DEFAULT_MS)
}

/// A client of the running dashboard. With `autostart`, a server that does not answer is
/// started and waited for.
///
/// # Errors
/// [`ClientError::Down`] when it is not running and may not be started (`autostart` off, no
/// install found) or does not come up in time.
pub fn dashboard_client(env: &Env, paths: &Paths, autostart: bool) -> Result<Client, ClientError> {
    if let Some(client) = ready(paths) {
        return Ok(client);
    }
    if !autostart {
        return Err(ClientError::down(None));
    }
    let Some(plan) = plan(env, paths) else {
        return Err(ClientError::down(Some("no dashboard install found")));
    };
    spawn(env, &plan);
    let deadline = Instant::now() + Duration::from_millis(wait_ms(env));
    while Instant::now() < deadline {
        sleep(POLL);
        if let Some(client) = ready(paths) {
            return Ok(client);
        }
    }
    Err(ClientError::down(None))
}
