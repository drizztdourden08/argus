//! Typing into a run: one line for the process's stdin (the terminal panel's input row).

use tokio::io::AsyncWriteExt;

use crate::error::RunError;
use crate::host::ProcessHost;
use crate::names::{check_key, check_rid};
use crate::record::RunStatus;
use crate::run_service::RunService;

/// stdin lines are small commands typed into the terminal panel, not data transfers.
pub const STDIN_MAX: usize = 4096;

impl<H: ProcessHost> RunService<H> {
    /// Sends `text` plus a newline to the stdin of a running run and notes it in the log as
    /// a `> text` line, because a pipe does not echo what is typed. Returns the bytes
    /// written.
    pub async fn send_input(&self, key: &str, rid: &str, text: &str) -> Result<usize, RunError> {
        check_key(key)?;
        check_rid(rid)?;
        let row = self.known_run(key, rid)?;
        if text.len() > STDIN_MAX {
            return Err(RunError::too_large(format!(
                "a stdin line is at most {STDIN_MAX} bytes"
            )));
        }
        let running = self
            .live_run(key, rid)
            .filter(|_| row.status == RunStatus::Running)
            .ok_or_else(|| {
                RunError::conflict(format!(
                    "{rid} is not running ({}); there is nothing to type into",
                    row.status.as_str()
                ))
            })?;
        let line = text
            .strip_suffix("\r\n")
            .or_else(|| text.strip_suffix('\n'))
            .unwrap_or(text);
        let mut stdin = running.stdin.lock().await;
        let pipe = stdin.as_mut().ok_or_else(|| {
            RunError::conflict(format!(
                "{rid} has no stdin (only launches and bundles keep one open)"
            ))
        })?;
        let written = format!("{line}\n");
        pipe.write_all(written.as_bytes())
            .await
            .map_err(|err| RunError::conflict(format!("{rid} did not take the line: {err}")))?;
        let _ = pipe.flush().await;
        drop(stdin);
        running.sys(&format!("> {}", running.redact.redact(line)));
        Ok(written.len())
    }
}
