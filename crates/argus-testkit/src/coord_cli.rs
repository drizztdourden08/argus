//! The Node coord CLI as a client of a test server: `node <ARGUS_COORD_CLI> <args>` with the
//! temp homes of the server, so the server is proven against the real client it must serve at
//! the switch-over. Without `ARGUS_COORD_CLI` the cases that need it say why and return.

use std::path::{Path, PathBuf};
use std::process::Stdio;
use std::time::Duration;

use tokio::io::AsyncWriteExt;
use tokio::process::Command;

use crate::TestServer;

const LIMIT: Duration = Duration::from_secs(30);

/// What a CLI run left behind.
#[derive(Debug, Clone)]
pub struct CliOut {
    /// The exit code (`-1` when the process was ended by a signal or the time limit).
    pub status: i32,
    /// Standard output.
    pub stdout: String,
    /// Standard error.
    pub stderr: String,
}

/// The located CLI.
#[derive(Debug, Clone)]
pub struct CoordCli {
    script: PathBuf,
}

impl CoordCli {
    /// The CLI named by `ARGUS_COORD_CLI`, or `None` after saying the case is skipped.
    #[must_use]
    pub fn locate() -> Option<CoordCli> {
        let script = std::env::var_os("ARGUS_COORD_CLI").map(PathBuf::from);
        match script.filter(|s| s.is_file()) {
            Some(script) => Some(CoordCli { script }),
            None => {
                eprintln!("skipped: ARGUS_COORD_CLI does not name the Node coord CLI (coord.mjs)");
                None
            }
        }
    }

    /// Runs `coord <args>` against the homes of `server`, from `cwd`, with `extra` environment
    /// variables and `input` on standard input.
    pub async fn run(
        &self,
        server: &TestServer,
        args: &[&str],
        cwd: &Path,
        extra: &[(&str, &str)],
    ) -> CliOut {
        self.run_with_input(server, args, cwd, extra, "").await
    }

    /// [`Self::run`] with text on standard input.
    pub async fn run_with_input(
        &self,
        server: &TestServer,
        args: &[&str],
        cwd: &Path,
        extra: &[(&str, &str)],
        input: &str,
    ) -> CliOut {
        let mut command = Command::new("node");
        command
            .arg("--no-warnings")
            .arg(&self.script)
            .args(args)
            .current_dir(cwd)
            .env("COORD_HOME", &server.coord_home)
            .env("DASHBOARD_HOME", &server.dashboard_home)
            .env_remove("CLAUDE_CODE_SESSION_ID")
            .env_remove("COORD_PROJECT")
            .envs(extra.iter().copied())
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .kill_on_drop(true);
        hide_window(&mut command);
        collect(command, input).await
    }
}

#[cfg(windows)]
fn hide_window(command: &mut Command) {
    const NO_WINDOW: u32 = 0x0800_0000;
    command.creation_flags(NO_WINDOW);
}

#[cfg(not(windows))]
fn hide_window(_command: &mut Command) {}

async fn collect(mut command: Command, input: &str) -> CliOut {
    let Ok(mut child) = command.spawn() else {
        return failed("node could not be started");
    };
    if let Some(mut stdin) = child.stdin.take() {
        let _written = stdin.write_all(input.as_bytes()).await;
    }
    match tokio::time::timeout(LIMIT, child.wait_with_output()).await {
        Ok(Ok(out)) => CliOut {
            status: out.status.code().unwrap_or(-1),
            stdout: String::from_utf8_lossy(&out.stdout).into_owned(),
            stderr: String::from_utf8_lossy(&out.stderr).into_owned(),
        },
        _ => failed("the coord CLI did not finish in 30 s"),
    }
}

fn failed(why: &str) -> CliOut {
    CliOut {
        status: -1,
        stdout: String::new(),
        stderr: why.to_owned(),
    }
}
