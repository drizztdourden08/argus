//! The probe's one `claude` child: spawned hidden with the stored key in its env only.

use std::fs::File;
use std::io::Write;
use std::path::Path;
use std::process::Stdio;
use std::time::Duration;

use argus_core::process::hidden_command;
use argus_overlord::ports::ApiKey;
use argus_overlord::protocol::{Event, decode_line};
use argus_overlord::session::args::build_probe_argv;
use argus_overlord::session::env::build_env;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader, Lines};
use tokio::process::{Child as Process, ChildStderr, ChildStdin, ChildStdout, Command};

use super::probe::Probe;
use super::redact::Redactor;
use crate::lifecycle::spawn::kill_tree;

/// The spend ceiling of the probe, in USD.
const BUDGET_USD: f64 = 1.0;
/// The approval timeout handed to the env builder; the probe holds no approvals.
const APPROVAL_MIN: u64 = 30;
/// How long a closed stdin may take to end the child.
const EXIT_WAIT: Duration = Duration::from_secs(10);

/// A running child with its pipes.
pub struct Child {
    process: Process,
    stdin: Option<ChildStdin>,
    lines: Lines<BufReader<ChildStdout>>,
}

impl Child {
    /// Starts `claude` in `cwd` with stderr going, redacted, to `stderr_log`.
    pub fn start(
        probe: &mut Probe,
        exe: &Path,
        key: &ApiKey,
        cwd: &Path,
        stderr_log: &Path,
    ) -> Result<Self, String> {
        let env = build_env(std::env::vars(), key, APPROVAL_MIN);
        let mut command = Command::from(hidden_command(exe));
        command
            .args(build_probe_argv("sonnet", BUDGET_USD))
            .env_clear()
            .envs(env.pairs().iter().map(|(k, v)| (k, v)))
            .current_dir(cwd)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .kill_on_drop(true);
        let mut process = command
            .spawn()
            .map_err(|e| format!("cannot start claude: {e}"))?;
        probe.pid = process.id();
        let stdout = process.stdout.take().ok_or("no stdout pipe")?;
        if let Some(stderr) = process.stderr.take() {
            drain_stderr(stderr, stderr_log, probe.redactor())?;
        }
        Ok(Self {
            stdin: process.stdin.take(),
            lines: BufReader::new(stdout).lines(),
            process,
        })
    }

    /// Writes one line to the child's stdin.
    pub async fn send(&mut self, line: &str) -> Result<(), String> {
        let stdin = self.stdin.as_mut().ok_or("stdin is closed")?;
        stdin
            .write_all(line.as_bytes())
            .await
            .map_err(|e| format!("cannot write to the child: {e}"))?;
        stdin
            .flush()
            .await
            .map_err(|e| format!("cannot write to the child: {e}"))
    }

    /// The next well formed event; each raw line is logged first. Blank and malformed lines are
    /// logged and skipped.
    pub async fn next_event(&mut self, probe: &mut Probe) -> Result<Event, String> {
        loop {
            let line = self
                .lines
                .next_line()
                .await
                .map_err(|e| format!("cannot read the child: {e}"))?
                .ok_or("the child closed its stdout")?;
            probe.log_raw(&line);
            if let Ok(event) = decode_line(&line) {
                return Ok(event);
            }
        }
    }

    /// Closes stdin, waits up to 10 s for the exit, then tree kills by pid. Returns the exit text.
    pub async fn finish(&mut self, probe: &mut Probe) -> String {
        self.stdin = None;
        let waited = tokio::time::timeout(EXIT_WAIT, self.process.wait()).await;
        probe.pid = None;
        match waited {
            Ok(Ok(status)) => status
                .code()
                .map_or_else(|| "signal".to_owned(), |c| c.to_string()),
            Ok(Err(e)) => format!("wait failed: {e}"),
            Err(_) => {
                if let Some(pid) = self.process.id() {
                    kill_tree(pid);
                }
                "killed after 10 s".to_owned()
            }
        }
    }
}

/// Copies the child's stderr, redacted, to a file in a background task.
fn drain_stderr(stderr: ChildStderr, path: &Path, redactor: Redactor) -> Result<(), String> {
    let mut file =
        File::create(path).map_err(|e| format!("cannot create {}: {e}", path.display()))?;
    tokio::spawn(async move {
        let mut lines = BufReader::new(stderr).lines();
        while let Ok(Some(line)) = lines.next_line().await {
            let _ignored = writeln!(file, "{}", redactor.apply(&line));
        }
    });
    Ok(())
}
