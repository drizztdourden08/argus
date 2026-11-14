//! The headless resume: the CLI's print mode with `--resume <session>` and the wake text, from the desktop's own CLI, hidden, in the
//! session's folder, with the parent's auth and session variables stripped so the CLI signs in by itself.
//! It ends by itself or is killed by its pid after 20 minutes. Never killed by name.

use std::path::Path;
use std::process::Stdio;
use std::sync::mpsc;
use std::thread;
use std::time::Duration;

use argus_core::process::hidden_command;
use argus_overlord::session::env::strip_env;
use argus_procs::{Killer, ProcRow, Sampler, SystemKiller, SystemSampler};

/// A headless resume that runs longer than this is killed.
pub const TIMEOUT: Duration = Duration::from_secs(20 * 60);

/// Does a running CLI process carry this session id on its command line?
#[must_use]
pub fn carries_session(rows: &[ProcRow], session_id: &str) -> bool {
    rows.iter()
        .any(|row| row.name.to_ascii_lowercase().contains("claude") && row.cmd.contains(session_id))
}

/// Is a CLI process running that carries the session id? A machine that cannot be read says no.
#[must_use]
pub fn session_running(session_id: &str) -> bool {
    SystemSampler
        .sample()
        .is_ok_and(|sample| carries_session(&sample.procs, session_id))
}

/// Start the resume and return at once; a helper thread waits for the child and ends it after [`TIMEOUT`].
///
/// # Errors
/// When the CLI could not be started.
pub fn resume(exe: &Path, cwd: &Path, session_id: &str, text: &str) -> std::io::Result<()> {
    let mut command = hidden_command(exe);
    command
        .args([
            "-p",
            "--resume",
            session_id,
            text,
            "--output-format",
            "json",
        ])
        .current_dir(cwd)
        .env_clear()
        .envs(strip_env(std::env::vars()))
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null());
    let mut child = command.spawn()?;
    let pid = child.id();
    let (done, finished) = mpsc::channel::<()>();
    thread::spawn(move || {
        if finished.recv_timeout(TIMEOUT) == Err(mpsc::RecvTimeoutError::Timeout) {
            drop(SystemKiller.kill(pid, true, &[]));
        }
    });
    thread::spawn(move || {
        drop(child.wait());
        let _ = done.send(());
    });
    Ok(())
}

// test-tag: critical
#[cfg(test)]
mod tests {
    use super::*;

    fn row(name: &str, cmd: &str) -> ProcRow {
        ProcRow {
            pid: 1,
            ppid: 0,
            name: name.to_owned(),
            cmd: cmd.to_owned(),
            path: None,
            mem_bytes: 0,
            start_ms: 0,
            cpu_ms: 0,
            responding: None,
            cwd: None,
        }
    }

    #[test]
    fn only_a_cli_process_with_the_id_carries_the_session() {
        let rows = [
            row("claude.exe", "claude.exe --resume abc-123"),
            row("node.exe", "node other abc-999"),
        ];
        assert!(carries_session(&rows, "abc-123"));
        assert!(!carries_session(&rows, "abc-999"));
        assert!(!carries_session(&rows, "zzz"));
    }
}
