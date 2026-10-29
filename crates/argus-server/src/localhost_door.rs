//! The localhost door: a second listener on `127.0.0.1` (port 80 when run by `argus serve`) that
//! serves the same app as the main listener, so `http://<key>.localhost/` works with no port.
//! Browsers resolve every `*.localhost` name to loopback without DNS, and Claude's built-in
//! browser pane loads the page assets of those names where it blocks the ones of `.argus`.
//! It does not depend on the local names. `"localhostDoor": false` in `config.json` turns it off;
//! when the port cannot be bound (taken, or owned by `http.sys`) the log says so once and
//! nothing else changes.

use std::net::Ipv4Addr;

use argus_core::Config;
use serde_json::{Value, json};
use tokio::net::TcpListener;

use crate::Services;
use crate::security::Doors;

/// The key of `config.json`.
const KEY: &str = "localhostDoor";

fn is_on(config: &Config) -> bool {
    config
        .extra
        .get(KEY)
        .and_then(Value::as_bool)
        .unwrap_or(true)
}

/// Binds the door and records its port; `None` when it is off, not asked for, or not bindable.
pub(crate) async fn open(svc: &Services, port: Option<u16>) -> Option<TcpListener> {
    let port = port.filter(|_| is_on(&svc.config()))?;
    let bound = TcpListener::bind((Ipv4Addr::LOCALHOST, port)).await;
    let listener = match bound {
        Ok(listener) => listener,
        Err(error) => {
            let line = format!("localhost door: cannot bind 127.0.0.1:{port}, it stays off");
            svc.log.warn(&line, json!({ "err": error.to_string() }));
            return None;
        }
    };
    let at = listener.local_addr().ok()?.port();
    let _set = svc.localhost_door.set(at);
    Some(listener)
}

impl Services {
    /// The port of the localhost door, when it is bound.
    #[must_use]
    pub fn localhost_door(&self) -> Option<u16> {
        self.localhost_door.get().copied()
    }

    /// The doors a `Host` may name besides the main listener.
    #[must_use]
    pub fn doors(&self) -> Doors<'_> {
        Doors {
            names: self.door(),
            localhost: self.localhost_door(),
        }
    }
}
