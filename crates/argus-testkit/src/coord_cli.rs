//! The coord CLI as a client of a test server: the Node CLI as `node <ARGUS_COORD_CLI> <args>` and
//! the Rust port as `argus coord <args>`, both with the temp homes of the server, so the server is
//! proven against the real clients it must serve at and after the switch-over. Without the Node
//! script (`ARGUS_COORD_CLI`) or the binary (`ARGUS_EXE`, else the `argus` next to the test
//! executables) the cases that need it say why and return.

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

/// Which implementation of the CLI a client runs.
#[derive(Debug, Clone)]
enum Engine {
    /// `node <script>`.
    Node(PathBuf),
    /// `<exe> coord`.
    Argus(PathBuf),
}

/// The located CLI.
#[derive(Debug, Clone)]
pub struct CoordCli {
    engine: Engine,
}

impl CoordCli {
    /// The CLI named by `ARGUS_COORD_CLI`, or `None` after saying the case is skipped.
    #[must_use]
    pub fn locate() -> Option<CoordCli> {
        let script = std::env::var_os("ARGUS_COORD_CLI").map(PathBuf::from);
        let found = script.filter(|s| s.is_file());
        if found.is_none() {
            eprintln!("skipped: ARGUS_COORD_CLI does not name the Node coord CLI (coord.mjs)");
        }
        found.map(|script| CoordCli {
            engine: Engine::Node(script),
        })
    }

    /// `argus coord`, from `ARGUS_EXE` or the `argus` built next to the test executables, or
    /// `None` after saying the case is skipped.
    #[must_use]
    pub fn locate_argus() -> Option<CoordCli> {
        let found = argus_exe();
        if found.is_none() {
            eprintln!("skipped: no argus binary (set ARGUS_EXE or run `cargo build -p argus`)");
        }
        found.map(|exe| CoordCli {
            engine: Engine::Argus(exe),
        })
    }

    /// Every CLI that can run here, the Node one first. Empty after saying why when none can.
    #[must_use]
    pub fn locate_each() -> Vec<CoordCli> {
        [CoordCli::locate(), CoordCli::locate_argus()]
            .into_iter()
            .flatten()
            .collect()
    }

    /// Which client this is, for a failure message.
    #[must_use]
    pub fn name(&self) -> &'static str {
        match self.engine {
            Engine::Node(_) => "node coord",
            Engine::Argus(_) => "argus coord",
        }
    }

    fn command(&self) -> Command {
        match &self.engine {
            Engine::Node(script) => {
                let mut command = Command::new("node");
                command.arg("--no-warnings").arg(script);
                command
            }
            Engine::Argus(exe) => {
                let mut command = Command::new(exe);
                command.arg("coord");
                command
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
        let mut command = self.command();
        command
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

fn argus_exe() -> Option<PathBuf> {
    let named = std::env::var_os("ARGUS_EXE").map(PathBuf::from);
    let beside = std::env::current_exe().ok().and_then(|exe| {
        Some(
            exe.parent()?
                .parent()?
                .join(format!("argus{}", std::env::consts::EXE_SUFFIX)),
        )
    });
    named.or(beside).filter(|exe| exe.is_file())
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
