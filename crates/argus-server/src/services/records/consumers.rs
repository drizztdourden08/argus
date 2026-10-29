//! The checkouts that use the local packages: the output of `node localpkg.mjs status --json`,
//! asked at most once every 30 s and never for longer than 5 s. When the tool is missing or
//! too old to know `--json`, the list is empty and the answer says why.

use std::path::PathBuf;
use std::sync::Arc;
use std::time::{Duration, Instant};

use argus_records::registry::consumers_from;
use async_trait::async_trait;
use serde_json::Value;
use tokio::sync::Mutex;

const KEEP: Duration = Duration::from_secs(30);
const LIMIT: Duration = Duration::from_secs(5);

/// Where the list of consumers comes from. A test supplies its own.
#[async_trait]
pub trait ConsumerStatus: Send + Sync {
    /// The text `localpkg status --json` prints, or why it could not be had.
    async fn status_json(&self) -> Result<String, String>;
}

/// The real tool, run with `node`.
pub struct NodeStatus {
    /// The `node` program.
    pub node: String,
    /// The `localpkg.mjs` script.
    pub script: PathBuf,
    /// `LOCALPKG_HOME` for the tool, so it reads the registry this server shows.
    pub home: PathBuf,
}

#[cfg(windows)]
fn hide_window(command: &mut tokio::process::Command) {
    command.creation_flags(0x0800_0000);
}

#[cfg(not(windows))]
fn hide_window(_command: &mut tokio::process::Command) {}

#[async_trait]
impl ConsumerStatus for NodeStatus {
    async fn status_json(&self) -> Result<String, String> {
        let mut command = tokio::process::Command::new(&self.node);
        command
            .arg(&self.script)
            .args(["status", "--json"])
            .env("LOCALPKG_HOME", &self.home)
            .stdin(std::process::Stdio::null())
            .kill_on_drop(true);
        hide_window(&mut command);
        let run = tokio::time::timeout(LIMIT, command.output());
        let out = run
            .await
            .map_err(|_| "localpkg status timed out".to_owned())?
            .map_err(|e| format!("localpkg status did not run: {e}"))?;
        if out.status.success() {
            Ok(String::from_utf8_lossy(&out.stdout).into_owned())
        } else {
            Err("localpkg status --json failed (the tool may not have the flag yet)".to_owned())
        }
    }
}

/// What the registry shows of the consumers.
#[derive(Clone)]
pub struct Consumers {
    /// The checkouts that are on.
    pub list: Vec<Value>,
    /// Why the list is empty when it could not be had.
    pub note: Option<String>,
}

/// The consumers, kept for 30 s.
pub struct ConsumerCache {
    source: Option<Arc<dyn ConsumerStatus>>,
    kept: Mutex<Option<(Instant, Consumers)>>,
}

impl ConsumerCache {
    /// A cache over a source; none means the tool is off.
    #[must_use]
    pub fn new(source: Option<Arc<dyn ConsumerStatus>>) -> Self {
        ConsumerCache {
            source,
            kept: Mutex::new(None),
        }
    }

    async fn ask(&self) -> Consumers {
        let Some(source) = &self.source else {
            return Consumers {
                list: Vec::new(),
                note: Some("the consumers are not asked in this run".to_owned()),
            };
        };
        match source.status_json().await {
            Ok(text) => consumers_from(&text).map_or_else(
                || Consumers {
                    list: Vec::new(),
                    note: Some("localpkg status did not print consumers as JSON".to_owned()),
                },
                |list| Consumers { list, note: None },
            ),
            Err(note) => Consumers {
                list: Vec::new(),
                note: Some(note),
            },
        }
    }

    /// The consumers: the kept answer while it is under 30 s old, else a new one. Concurrent
    /// callers share one run of the tool.
    pub async fn get(&self) -> Consumers {
        let mut kept = self.kept.lock().await;
        if let Some((_, consumers)) = kept.as_ref().filter(|(at, _)| at.elapsed() < KEEP) {
            return consumers.clone();
        }
        let fresh = self.ask().await;
        *kept = Some((Instant::now(), fresh.clone()));
        fresh
    }
}
