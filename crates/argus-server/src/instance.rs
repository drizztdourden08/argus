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

/// Whether a process with this id is running.
#[cfg(windows)]
#[must_use]
pub fn pid_alive(pid: u32) -> bool {
    argus_native::process_alive(pid)
}

/// Whether a process with this id is running.
#[cfg(not(windows))]
#[must_use]
pub fn pid_alive(pid: u32) -> bool {
    Path::new(&format!("/proc/{pid}")).exists()
}

async fn probe(port: u16, token: Option<&str>) -> Option<Value> {
    let mut s = TcpStream::connect(("127.0.0.1", port)).await.ok()?;
    let auth = token
        .map(|t| format!("Authorization: Bearer {t}\r\n"))
        .unwrap_or_default();
    let req = format!(
        "GET /api/health HTTP/1.1\r\nHost: 127.0.0.1:{port}\r\n{auth}Connection: close\r\n\r\n"
    );
    s.write_all(req.as_bytes()).await.ok()?;
    let mut buf = Vec::new();
    s.read_to_end(&mut buf).await.ok()?;
    let text = String::from_utf8_lossy(&buf);
    let (head, body) = text.split_once("\r\n\r\n")?;
    head.starts_with("HTTP/1.1 200")
        .then(|| serde_json::from_str(body).ok())
        .flatten()
}

/// The live server of this home, if any.
pub async fn running_instance(pid_file: &Path, token: Option<&str>) -> Option<PidRecord> {
    let rec = read_pid_file(pid_file)?;
    let health = tokio::time::timeout(PROBE, probe(rec.port, token))
        .await
        .ok()??;
    (health["instance"].as_str() == Some(rec.instance.as_str())).then_some(rec)
}
