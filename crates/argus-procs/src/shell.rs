//! Running a hidden helper program with a time limit and reading what it prints.

use std::io::Read;
use std::process::{Command, Stdio};
use std::sync::mpsc;
use std::thread;
use std::time::Duration;

#[cfg(windows)]
use std::os::windows::process::CommandExt;

#[cfg(windows)]
const CREATE_NO_WINDOW: u32 = 0x0800_0000;

/// The most a helper may run before it is stopped.
pub const TIMEOUT: Duration = Duration::from_secs(30);

fn hidden(program: &str, args: &[String]) -> Command {
    let mut command = Command::new(program);
    command
        .args(args)
        .stdin(Stdio::null())
        .stderr(Stdio::piped())
        .stdout(Stdio::piped());
    #[cfg(windows)]
    command.creation_flags(CREATE_NO_WINDOW);
    command
}

/// What a helper printed and whether it succeeded.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Output {
    pub ok: bool,
    pub stdout: String,
    pub stderr: String,
}

fn drain(mut source: impl Read) -> String {
    let mut text = String::new();
    source.read_to_string(&mut text).ok();
    text
}

/// Runs `program` without a window; a helper still running after `limit` is stopped and reported as a failure.
///
/// # Errors
/// The reason the helper could not start, or that it timed out.
pub fn run_hidden(program: &str, args: &[String], limit: Duration) -> Result<Output, String> {
    let mut child = hidden(program, args)
        .spawn()
        .map_err(|e| format!("{program} failed: {e}"))?;
    let stderr = child.stderr.take().map(|s| thread::spawn(move || drain(s)));
    let (tx, rx) = mpsc::channel();
    if let Some(source) = child.stdout.take() {
        thread::spawn(move || tx.send(drain(source)).ok());
    }
    let Ok(stdout) = rx.recv_timeout(limit) else {
        child.kill().ok();
        child.wait().ok();
        return Err(format!(
            "{program} failed: timed out after {}s",
            limit.as_secs()
        ));
    };
    let ok = child.wait().is_ok_and(|s| s.success());
    let stderr = stderr.and_then(|h| h.join().ok()).unwrap_or_default();
    Ok(Output { ok, stdout, stderr })
}
