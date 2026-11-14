//! The real [`ChildHost`]: a hidden `claude` process with piped stdin and stdout, stderr to a log
//! with the key replaced, and an end by pid (close stdin, wait, then the launchers' tree kill).

use std::collections::HashMap;
use std::path::Path;
use std::process::Stdio;
use std::sync::{Arc, Mutex, PoisonError};
use std::time::Duration;

use argus_core::process::hidden_command;
use argus_launchers::{ProcessHost, SystemHost};
use argus_overlord::ports::{ChildHost, ChildSpec, HostFault, LineStream, SpawnedChild};
use tokio::fs::OpenOptions;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader, Lines};
use tokio::process::{ChildStderr, ChildStdin, ChildStdout, Command};
use tokio::sync::{Mutex as AsyncMutex, watch};
use tokio::time::timeout;

/// How long a stopping child has to exit after its stdin closes.
const STOP_GRACE: Duration = Duration::from_secs(10);
/// The grace the tree kill gives before it forces.
const KILL_GRACE: Duration = Duration::from_secs(2);
const KEY_VAR: &str = "ANTHROPIC_API_KEY";

struct Entry {
    stdin: Arc<AsyncMutex<Option<ChildStdin>>>,
    exited: watch::Receiver<bool>,
}

/// Starts and ends the children of the hosted Overlord.
#[derive(Clone, Default)]
pub struct SystemChildHost {
    children: Arc<Mutex<HashMap<u32, Entry>>>,
}

/// The stdout lines of a child.
pub struct StdoutLines(Lines<BufReader<ChildStdout>>);

impl LineStream for StdoutLines {
    async fn next_line(&mut self) -> Option<String> {
        self.0.next_line().await.ok().flatten()
    }
}

impl SystemChildHost {
    fn entry_parts(
        &self,
        pid: u32,
    ) -> Option<(Arc<AsyncMutex<Option<ChildStdin>>>, watch::Receiver<bool>)> {
        let children = self.children.lock().unwrap_or_else(PoisonError::into_inner);
        children
            .get(&pid)
            .map(|entry| (Arc::clone(&entry.stdin), entry.exited.clone()))
    }

    fn forget(&self, pid: u32) {
        let mut children = self.children.lock().unwrap_or_else(PoisonError::into_inner);
        children.remove(&pid);
    }
}

fn command_for(spec: &ChildSpec) -> Command {
    let mut command = Command::from(hidden_command(&spec.program));
    command
        .args(&spec.args)
        .current_dir(&spec.cwd)
        .env_clear()
        .envs(spec.env.pairs().iter().map(|(k, v)| (k, v)))
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());
    command
}

fn redact(line: &str, key: Option<&str>) -> String {
    match key {
        Some(key) if !key.is_empty() => line.replace(key, "<redacted>"),
        _ => line.to_owned(),
    }
}

async fn log_stderr(stderr: ChildStderr, log: &Path, key: Option<String>) {
    let Ok(mut file) = OpenOptions::new().create(true).append(true).open(log).await else {
        return;
    };
    let mut lines = BufReader::new(stderr).lines();
    while let Ok(Some(line)) = lines.next_line().await {
        let text = format!("{}\n", redact(&line, key.as_deref()));
        if file.write_all(text.as_bytes()).await.is_err() {
            return;
        }
    }
}

impl ChildHost for SystemChildHost {
    type Lines = StdoutLines;

    fn spawn(&self, spec: &ChildSpec) -> Result<SpawnedChild<StdoutLines>, HostFault> {
        let mut child = command_for(spec)
            .spawn()
            .map_err(|error| HostFault(format!("cannot start {}: {error}", spec.program.display())))?;
        let pid = child
            .id()
            .ok_or_else(|| HostFault("the child ended before it had a pid".to_owned()))?;
        let (stdin, stdout, stderr) = (child.stdin.take(), child.stdout.take(), child.stderr.take());
        let (Some(stdin), Some(stdout), Some(stderr)) = (stdin, stdout, stderr) else {
            return Err(HostFault("the child pipes are missing".to_owned()));
        };
        let (sender, exited) = watch::channel(false);
        let entry = Entry {
            stdin: Arc::new(AsyncMutex::new(Some(stdin))),
            exited,
        };
        self.children
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .insert(pid, entry);
        let key = spec.env.get(KEY_VAR).map(str::to_owned);
        let log = spec.stderr_log.clone();
        tokio::spawn(async move { log_stderr(stderr, &log, key).await });
        let host = self.clone();
        tokio::spawn(async move {
            let _status = child.wait().await;
            host.forget(pid);
            let _sent = sender.send(true);
        });
        Ok(SpawnedChild {
            pid,
            stdout: StdoutLines(BufReader::new(stdout).lines()),
        })
    }

    async fn write_line(&self, pid: u32, line: &str) -> Result<(), HostFault> {
        let (stdin, _exited) = self
            .entry_parts(pid)
            .ok_or_else(|| HostFault(format!("pid {pid} is not a running child of ours")))?;
        let mut guard = stdin.lock().await;
        let pipe = guard
            .as_mut()
            .ok_or_else(|| HostFault(format!("stdin of {pid} is closed")))?;
        let text = format!("{line}\n");
        pipe.write_all(text.as_bytes())
            .await
            .map_err(|error| HostFault(format!("cannot write to {pid}: {error}")))?;
        pipe.flush()
            .await
            .map_err(|error| HostFault(format!("cannot flush {pid}: {error}")))
    }

    async fn close_stdin(&self, pid: u32) {
        if let Some((stdin, _exited)) = self.entry_parts(pid) {
            stdin.lock().await.take();
        }
    }

    async fn stop(&self, pid: u32) {
        let Some((_stdin, mut exited)) = self.entry_parts(pid) else {
            return;
        };
        self.close_stdin(pid).await;
        let gone = timeout(STOP_GRACE, exited.wait_for(|done| *done)).await;
        if gone.is_err() {
            self.kill_tree(pid).await;
        }
    }

    async fn kill_tree(&self, pid: u32) {
        if self.entry_parts(pid).is_some() {
            SystemHost.kill_tree(pid, KILL_GRACE).await;
        }
    }
}
