//! Where the server of this dashboard home is and how to talk to it: the pid file knows the real
//! port (an ephemeral one in tests), `config.json` the token and the configured port. The token
//! is sent only when the config asks for it. Nothing here starts a server.

use std::time::Duration;

use argus_core::Paths;
use argus_core::config::{env_port, peek_config};
use argus_server::instance::{PidRecord, call_with, pid_alive, probe, read_pid_file};
use serde_json::Value;
use tokio::time::timeout;

const CALL_LIMIT: Duration = Duration::from_secs(20);

/// The address of a server and what to send it.
pub struct Live {
    /// The loopback port.
    pub port: u16,
    /// The bearer, when the config requires one.
    pub token: Option<String>,
    /// The record of the pid file, when it is readable.
    pub record: Option<PidRecord>,
}

/// A server that answered its health call.
pub struct Up {
    /// Where it is.
    pub at: Live,
    /// Its health answer.
    pub health: Value,
}

impl Live {
    /// `http://127.0.0.1:<port>`.
    #[must_use]
    pub fn base(&self) -> String {
        format!("http://127.0.0.1:{}", self.port)
    }

    /// The pid of the pid file, as the lines print it.
    #[must_use]
    pub fn pid_text(&self) -> String {
        self.record
            .as_ref()
            .map_or_else(|| "?".to_owned(), |r| r.pid.to_string())
    }
}

/// The server of `paths` as far as the files say. `None` without a config or a usable port.
#[must_use]
pub fn locate(paths: &Paths) -> Option<Live> {
    let config = peek_config(paths, env_port().ok().flatten())?;
    config["token"].as_str().filter(|t| !t.is_empty())?;
    let record = read_pid_file(&paths.pid_file);
    let live = record
        .as_ref()
        .filter(|r| r.port > 0 && pid_alive(r.pid))
        .map(|r| r.port);
    let configured = config["port"].as_u64().and_then(|p| u16::try_from(p).ok());
    let port = live.or(configured).filter(|p| *p > 0)?;
    let token = (config["requireToken"] == true)
        .then(|| config["token"].as_str().map(str::to_owned))
        .flatten();
    Some(Live {
        port,
        token,
        record,
    })
}

/// The server when it answers its health call.
pub async fn up(paths: &Paths) -> Option<Up> {
    let at = locate(paths)?;
    let health = probe(at.port, at.token.as_deref()).await?;
    (health["ok"] == true).then_some(Up { at, health })
}

/// One call to a running server: the answer's JSON, or the server's own sentence for a refusal.
///
/// # Errors
///
/// Nothing answered, or the answer was not a success.
pub async fn api(at: &Live, method: &str, path: &str, body: &Value) -> Result<Value, String> {
    let text = body.to_string();
    let asked = call_with(at.port, method, path, at.token.as_deref(), &text);
    let (status, reply) = timeout(CALL_LIMIT, asked)
        .await
        .ok()
        .flatten()
        .ok_or_else(|| format!("no answer from {path}"))?;
    let data: Value = serde_json::from_str(&reply).unwrap_or(Value::Null);
    if (200..300).contains(&status) {
        return Ok(data);
    }
    Err(data["error"]["message"]
        .as_str()
        .map_or_else(|| format!("HTTP {status} from {path}"), str::to_owned))
}
