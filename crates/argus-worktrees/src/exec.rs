//! Running a program with a time limit: the one place a process is spawned.

use std::io::Read;
use std::process::{Child, Stdio};
use std::sync::mpsc::{self, Receiver};
use std::thread;
use std::time::{Duration, Instant};

use argus_core::process::hidden_command;

/// What a program said. A failure to run, or to finish in time, is a result and never a panic.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct GitOutput {
    pub code: Option<i32>,
    pub stdout: String,
    pub stderr: String,
    pub timed_out: bool,
    pub error: Option<String>,
}

impl GitOutput {
    /// A run that ended well with this output.
    pub fn success(stdout: &str) -> Self {
        Self {
            code: Some(0),
            stdout: stdout.to_string(),
            ..Self::default()
        }
    }

    /// A run that ended with an exit code and this message on stderr.
    pub fn failure(code: i32, stderr: &str) -> Self {
        Self {
            code: Some(code),
            stderr: stderr.to_string(),
            ..Self::default()
        }
    }

    /// A run that never happened.
    pub fn refused(error: &str) -> Self {
        Self {
            error: Some(error.to_string()),
            ..Self::default()
        }
    }

    pub fn ok(&self) -> bool {
        self.code == Some(0)
    }

    /// Did the run end because it was cut: it ran past its limit, or something killed it?
    pub fn was_cut(&self) -> bool {
        if self.timed_out {
            return true;
        }
        let said = self.error.as_deref().unwrap_or_default().to_lowercase();
        let word = |w: &str| {
            said.split(|c: char| !(c.is_alphanumeric() || c == '_'))
                .any(|t| t == w)
        };
        said.contains("timed out")
            || said.contains("sigterm")
            || said.contains("sigkill")
            || said.contains("aborterror")
            || word("killed")
            || word("aborted")
    }

    /// The first non-blank line of stderr, cut to 200 characters.
    pub fn first_line(&self) -> String {
        first_line(&self.stderr)
    }

    /// What to show when the run failed: stderr's first line, else the error, else the exit code.
    pub fn failure_text(&self) -> String {
        let line = self.first_line();
        if !line.is_empty() {
            return line;
        }
        self.error.clone().unwrap_or_else(|| {
            format!(
                "exit {}",
                self.code.map_or_else(|| "?".to_string(), |c| c.to_string())
            )
        })
    }
}

/// The first non-blank line of a text, trimmed and cut to 200 characters.
pub fn first_line(text: &str) -> String {
    text.lines()
        .map(str::trim)
        .find(|l| !l.is_empty())
        .unwrap_or_default()
        .chars()
        .take(200)
        .collect()
}

/// One program to run.
pub struct Spec<'a> {
    pub program: &'a str,
    pub args: &'a [String],
    pub cwd: &'a str,
    pub env: &'a [(&'a str, &'a str)],
    pub timeout: Duration,
}

/// How long the output may still come after the program ended: a child it left behind may hold the pipe open forever.
const DRAIN_GRACE: Duration = Duration::from_secs(2);

/// Read a pipe on its own thread, chunk by chunk, so what came before a deadline is never lost.
fn reader<R: Read + Send + 'static>(source: Option<R>) -> Receiver<Vec<u8>> {
    let (tx, rx) = mpsc::channel();
    thread::spawn(move || {
        let Some(mut source) = source else {
            return;
        };
        let mut chunk = [0u8; 8192];
        while let Ok(n) = source.read(&mut chunk) {
            if n == 0 || tx.send(chunk[..n].to_vec()).is_err() {
                return;
            }
        }
    });
    rx
}

/// What a pipe said until it closed, or until the deadline.
fn drained(pipe: &Receiver<Vec<u8>>, until: Instant) -> String {
    let mut bytes = Vec::new();
    while let Ok(chunk) = pipe.recv_timeout(until.saturating_duration_since(Instant::now())) {
        bytes.extend(chunk);
    }
    String::from_utf8_lossy(&bytes).into_owned()
}

/// Windows: the `git.exe` of `Git\cmd` is a launcher that starts the real git, so killing the launcher alone leaves the
/// real git running (and holding the worktree's files). The whole tree goes, by process id, while the launcher still
/// links it.
#[cfg(windows)]
fn kill_tree(child: &Child) {
    let mut command = hidden_command("taskkill");
    command
        .args(["/PID", &child.id().to_string(), "/T", "/F"])
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null());
    let _ = command.status();
}

#[cfg(not(windows))]
fn kill_tree(_child: &Child) {}

/// The program ran past its time: it goes, with what it started.
fn cut(spec: &Spec<'_>, child: &mut Child) -> GitOutput {
    kill_tree(child);
    let _ = child.kill();
    let _ = child.wait();
    let text = format!(
        "{} timed out after {} s",
        spec.program,
        (spec.timeout.as_millis() + 500) / 1000
    );
    GitOutput {
        timed_out: true,
        error: Some(text),
        ..GitOutput::default()
    }
}

/// Run a program, wait at most `timeout`, kill it (and what it started) after that. Once it ended, its output is read
/// for a short grace at most: a child it left behind holding the pipe never holds up the answer.
pub fn run_command(spec: &Spec<'_>) -> GitOutput {
    let mut command = hidden_command(spec.program);
    command
        .args(spec.args)
        .current_dir(spec.cwd)
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());
    for (key, value) in spec.env {
        command.env(key, value);
    }
    let mut child = match command.spawn() {
        Ok(child) => child,
        Err(err) => return GitOutput::refused(&err.to_string()),
    };
    let out = reader(child.stdout.take());
    let err = reader(child.stderr.take());
    let deadline = Instant::now() + spec.timeout;
    loop {
        match child.try_wait() {
            Ok(Some(status)) => {
                let until = Instant::now() + DRAIN_GRACE;
                return GitOutput {
                    code: status.code(),
                    stdout: drained(&out, until),
                    stderr: drained(&err, until),
                    timed_out: false,
                    error: None,
                };
            }
            Ok(None) if Instant::now() >= deadline => return cut(spec, &mut child),
            Ok(None) => thread::sleep(Duration::from_millis(5)),
            Err(e) => return GitOutput::refused(&e.to_string()),
        }
    }
}

// test-tag: critical
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_missing_program_is_a_result() {
        let out = run_command(&Spec {
            program: "argus-no-such-program",
            args: &[],
            cwd: ".",
            env: &[],
            timeout: Duration::from_secs(2),
        });
        assert_eq!(out.code, None);
        assert!(out.error.is_some());
    }

    #[test]
    fn a_run_that_timed_out_or_was_killed_was_cut() {
        assert!(GitOutput::refused("git timed out after 900 s").was_cut());
        assert!(GitOutput::refused("spawn git ENOENT SIGTERM").was_cut());
        assert!(GitOutput::refused("the process was killed").was_cut());
        assert!(
            GitOutput {
                timed_out: true,
                ..GitOutput::default()
            }
            .was_cut()
        );
        assert!(!GitOutput::refused("spawn git ENOENT").was_cut());
        assert!(!GitOutput::failure(128, "fatal: timed out").was_cut());
        assert!(!GitOutput::refused("skilled worker").was_cut());
    }

    #[test]
    fn first_line_skips_blank_lines() {
        assert_eq!(first_line("\n  \n  error: x  \nmore"), "error: x");
        assert_eq!(
            GitOutput::failure(128, "fatal: no\n").failure_text(),
            "fatal: no"
        );
        assert_eq!(GitOutput::failure(3, "").failure_text(), "exit 3");
    }
}
