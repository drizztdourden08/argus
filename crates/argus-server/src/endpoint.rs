//! Where the running server of a dashboard home is, for the commands that talk to it: the live
//! pid file knows the real port, the config the configured one.

use argus_core::Paths;
use argus_core::config::{env_port, peek_config};

use crate::instance::{pid_alive, read_pid_file};

/// A server to talk to.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Endpoint {
    /// The loopback port.
    pub port: u16,
    /// The bearer, when the config asks for a token.
    pub token: Option<String>,
}

/// The server of a home: the port of its live pid file, else the config's, and the token when
/// the config requires one. `None` when no port is known.
#[must_use]
pub fn endpoint(paths: &Paths) -> Option<Endpoint> {
    let config = peek_config(paths, env_port().ok().flatten());
    let configured = config
        .as_ref()
        .and_then(|c| c["port"].as_u64())
        .and_then(|p| u16::try_from(p).ok());
    let live = read_pid_file(&paths.pid_file).filter(|r| r.port > 0 && pid_alive(r.pid));
    let port = live.map(|r| r.port).or(configured).filter(|p| *p > 0)?;
    let token = config
        .filter(|c| c["requireToken"] == true)
        .and_then(|c| c["token"].as_str().map(str::to_owned))
        .filter(|t| !t.is_empty());
    Some(Endpoint { port, token })
}
