//! Where the server of this dashboard home is and how to talk to it: the pid file knows the real
//! port (an ephemeral one in tests), `config.json` the token and the configured port. The token
//! is sent only when the config asks for it. Nothing here starts a server.

use std::time::Duration;

use argus_core::Paths;
use argus_core::config::{env_port, peek_config};
use argus_server::instance::{PidRecord, call_with, pid_alive, probe, read_pid_file};
use serde_json::Value;
use tokio::time::{sleep, timeout};

const CALL_LIMIT: Duration = Duration::from_secs(20);
/// How often, and how many times, a server that answers is waited for to show its pid file.
const RECORD_STEP: Duration = Duration::from_millis(50);
const RECORD_TRIES: u32 = 40;

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

impl Up {
    /// Whether this is an argus server. The Node dashboard's health has no `impl` field.
    #[must_use]
    pub fn is_argus(&self) -> bool {
        self.health["impl"] == "argus"
    }

    /// When this is the Node dashboard: says so on stderr and answers the exit code 1. The verbs
    /// that change a server never act on it, whatever home it was found through.
    #[must_use]
    pub fn refuse_node(&self, verb: &str) -> Option<u8> {
        if self.is_argus() {
            return None;
        }
        eprintln!(
            "dashboard: the server at {} is the Node dashboard, not argus; `argus {verb}` does not touch it",
            self.at.base()
        );
        Some(1)
    }
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

/// The pid file of a server that answers: read again for a moment while it cannot be read, since
/// the lines print the pid and a file that is being replaced is briefly unreadable on Windows.
async fn settled_record(file: &std::path::Path, tries: u32, step: Duration) -> Option<PidRecord> {
    for _ in 0..tries {
        if let Some(record) = read_pid_file(file) {
            return Some(record);
        }
        sleep(step).await;
    }
    read_pid_file(file)
}

/// The server when it answers its health call.
pub async fn up(paths: &Paths) -> Option<Up> {
    let mut at = locate(paths)?;
    let health = probe(at.port, at.token.as_deref()).await?;
    if health["ok"] != true {
        return None;
    }
    if at.record.is_none() {
        at.record = settled_record(&paths.pid_file, RECORD_TRIES, RECORD_STEP).await;
    }
    Some(Up { at, health })
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

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use super::settled_record;

    #[tokio::test]
    async fn a_pid_file_that_shows_up_late_is_still_read() {
        let dir = tempfile::tempdir().expect("a temp dir");
        let file = dir.path().join("server.pid");
        let writer = file.clone();
        let late = tokio::spawn(async move {
            tokio::time::sleep(Duration::from_millis(120)).await;
            std::fs::write(&writer, r#"{"pid":4242,"port":1,"instance":"i"}"#).expect("pid file");
        });
        let record = settled_record(&file, 40, Duration::from_millis(20)).await;
        late.await.expect("the writer ends");
        assert_eq!(record.map(|r| r.pid), Some(4242));
    }

    #[tokio::test]
    async fn no_pid_file_gives_up_after_the_tries() {
        let dir = tempfile::tempdir().expect("a temp dir");
        let file = dir.path().join("server.pid");
        let none = settled_record(&file, 3, Duration::from_millis(5)).await;
        assert!(none.is_none());
    }
}
