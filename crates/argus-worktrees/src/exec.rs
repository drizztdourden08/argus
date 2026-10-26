//! Running a program with a time limit: the one place a process is spawned.

use std::io::Read;
use std::process::{Command, Stdio};
use std::thread;
use std::time::{Duration, Instant};

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

fn reader<R: Read + Send + 'static>(source: Option<R>) -> thread::JoinHandle<String> {
    thread::spawn(move || {
        let mut bytes = Vec::new();
        if let Some(mut source) = source {
            let _ = source.read_to_end(&mut bytes);
        }
        String::from_utf8_lossy(&bytes).into_owned()
    })
}

#[cfg(windows)]
fn hide_window(command: &mut Command) {
    use std::os::windows::process::CommandExt;
    command.creation_flags(0x0800_0000);
}

#[cfg(not(windows))]
fn hide_window(_command: &mut Command) {}

/// Run a program, wait at most `timeout`, kill it after that.
pub fn run_command(spec: &Spec<'_>) -> GitOutput {
    let mut command = Command::new(spec.program);
    command
        .args(spec.args)
        .current_dir(spec.cwd)
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());
    for (key, value) in spec.env {
        command.env(key, value);
    }
    hide_window(&mut command);
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
                let (stdout, stderr) = (
                    out.join().unwrap_or_default(),
                    err.join().unwrap_or_default(),
                );
                return GitOutput {
                    code: status.code(),
                    stdout,
                    stderr,
                    timed_out: false,
                    error: None,
                };
            }
            Ok(None) if Instant::now() >= deadline => {
                let _ = child.kill();
                let _ = child.wait();
                let text = format!(
                    "{} timed out after {} s",
                    spec.program,
                    spec.timeout.as_secs()
                );
                return GitOutput {
                    timed_out: true,
                    error: Some(text),
                    ..GitOutput::default()
                };
            }
            Ok(None) => thread::sleep(Duration::from_millis(5)),
            Err(e) => return GitOutput::refused(&e.to_string()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[cfg(windows)]
    fn sleeper() -> (&'static str, Vec<String>) {
        ("ping", ["-n", "8", "127.0.0.1"].map(String::from).to_vec())
    }

    #[cfg(not(windows))]
    fn sleeper() -> (&'static str, Vec<String>) {
        ("sleep", vec!["8".to_string()])
    }

    #[test]
    fn a_program_that_hangs_is_stopped_after_the_timeout() {
        let (program, args) = sleeper();
        let started = Instant::now();
        let out = run_command(&Spec {
            program,
            args: &args,
            cwd: ".",
            env: &[],
            timeout: Duration::from_millis(150),
        });
        assert!(out.timed_out);
        assert!(out.error.unwrap().contains("timed out"));
        assert!(started.elapsed() < Duration::from_secs(5));
    }

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
    fn first_line_skips_blank_lines() {
        assert_eq!(first_line("\n  \n  error: x  \nmore"), "error: x");
        assert_eq!(
            GitOutput::failure(128, "fatal: no\n").failure_text(),
            "fatal: no"
        );
        assert_eq!(GitOutput::failure(3, "").failure_text(), "exit 3");
    }
}
