//! One server per dashboard home: `server.pid` names the running one, and it only counts when a
//! server answers on its port with the recorded instance (a recycled pid is someone else).

use std::path::Path;
use std::time::Duration;

use serde_json::{Value, json};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;

const PROBE: Duration = Duration::from_secs(2);

/// What `server.pid` records.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PidRecord {
    /// The process id.
    pub pid: u32,
    /// The port.
    pub port: u16,
    /// The instance id.
    pub instance: String,
}

/// The pid file, when it is readable.
#[must_use]
pub fn read_pid_file(file: &Path) -> Option<PidRecord> {
    let v: Value = serde_json::from_str(&std::fs::read_to_string(file).ok()?).ok()?;
    Some(PidRecord {
        pid: u32::try_from(v["pid"].as_u64()?).ok()?,
        port: u16::try_from(v["port"].as_u64().unwrap_or(0)).ok()?,
        instance: v["instance"].as_str().unwrap_or_default().to_owned(),
    })
}

/// Writes the pid file.
///
/// # Errors
/// When it cannot be written.
pub fn write_pid_file(
    file: &Path,
    port: u16,
    started_at: &str,
    instance: &str,
) -> std::io::Result<()> {
    let rec = json!({ "pid": std::process::id(), "port": port, "startedAt": started_at, "instance": instance });
    std::fs::write(file, rec.to_string())
}

/// One request to a server on this machine: the status and the body. A write carries an empty
/// JSON object. `None` when nothing answers.
pub async fn call(
    port: u16,
    method: &str,
    path: &str,
    token: Option<&str>,
) -> Option<(u16, String)> {
    call_with(port, method, path, token, "{}").await
}

/// [`call`] with the JSON text a write carries. A `GET` carries none.
pub async fn call_with(
    port: u16,
    method: &str,
    path: &str,
    token: Option<&str>,
    body: &str,
) -> Option<(u16, String)> {
    let mut s = TcpStream::connect(("127.0.0.1", port)).await.ok()?;
    let auth = token
        .map(|t| format!("Authorization: Bearer {t}\r\n"))
        .unwrap_or_default();
    let (kind, body) = if method == "GET" {
        ("", "")
    } else {
        ("Content-Type: application/json\r\n", body)
    };
    let len = body.len();
    let req = format!(
        "{method} {path} HTTP/1.1\r\nHost: 127.0.0.1:{port}\r\n{auth}{kind}Content-Length: {len}\r\nConnection: close\r\n\r\n{body}"
    );
    s.write_all(req.as_bytes()).await.ok()?;
    let mut buf = Vec::new();
    s.read_to_end(&mut buf).await.ok()?;
    let text = String::from_utf8_lossy(&buf);
    let (head, body) = text.split_once("\r\n\r\n")?;
    let status = head.split(' ').nth(1)?.parse().ok()?;
    Some((status, body.to_owned()))
}

/// Whether a process with this id runs (one that denies the query counts as running).
#[must_use]
pub fn pid_alive(pid: u32) -> bool {
    crate::adapters::procs::alive(pid)
}

/// The health answer of the server on `port`, when it says 200.
pub async fn probe(port: u16, token: Option<&str>) -> Option<Value> {
    let (status, body) = tokio::time::timeout(PROBE, call(port, "GET", "/api/health", token))
        .await
        .ok()??;
    (status == 200).then(|| serde_json::from_str(&body).ok())?
}

/// The live server of this home, if any.
pub async fn running_instance(pid_file: &Path, token: Option<&str>) -> Option<PidRecord> {
    let rec = read_pid_file(pid_file)?;
    let health = probe(rec.port, token).await?;
    (health["instance"].as_str() == Some(rec.instance.as_str())).then_some(rec)
}
