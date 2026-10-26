//! Spawning one step on one engine, in one home, with an environment that cannot reach a live
//! dashboard or the real board tree.

use std::fs;
use std::io::{self, Write};
use std::process::{Command, Stdio};

use super::engines::Engines;
use super::homes::Home;
use super::step::Step;

/// Which engine runs a step.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Which {
    /// `node coord.mjs ...` with `COORD_ENGINE=node`.
    Node,
    /// `argus coord ...`.
    Argus,
}

/// Variables of the surrounding session that must not leak into a call.
const SCRUBBED: [&str; 8] = [
    "CLAUDE_CODE_SESSION_ID",
    "CLAUDE_CODE_HOST_SESSION_ID",
    "COORD_PROJECT",
    "COORD_DASHBOARD_INSTALL",
    "COORD_DASHBOARD_WAIT_MS",
    "COORD_DASHBOARD_AUTOSTART",
    "COORD_LISTEN_MS",
    "COORD_ENGINE",
];

/// What a call did, as bytes.
#[derive(Clone, Debug)]
pub struct Observed {
    /// The exit code (`None` when a signal ended the process).
    pub code: Option<i32>,
    /// Everything on stdout.
    pub stdout: Vec<u8>,
    /// Everything on stderr.
    pub stderr: Vec<u8>,
}

/// Runs `step` on `which` in `home`. `port` is the closed dashboard port of the twin.
pub fn run_step(
    engines: &Engines,
    which: Which,
    home: &Home,
    port: u16,
    step: &Step,
) -> io::Result<Observed> {
    let cwd = match &step.cwd {
        Some(folder) => home.work().join(folder),
        None => home.work(),
    };
    fs::create_dir_all(&cwd)?;
    let mut command = base_command(engines, which);
    command.args(&step.argv).current_dir(cwd);
    apply_env(&mut command, home, port, step);
    let mut child = command
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()?;
    // The call may end without reading stdin; a closed pipe is not an error.
    if let Some(mut stdin) = child.stdin.take() {
        stdin.write_all(step.stdin.as_bytes()).ok();
    }
    let out = child.wait_with_output()?;
    Ok(Observed {
        code: out.status.code(),
        stdout: out.stdout,
        stderr: out.stderr,
    })
}

fn base_command(engines: &Engines, which: Which) -> Command {
    match which {
        Which::Node => {
            let mut command = Command::new("node");
            command.arg(&engines.node_cli).env("COORD_ENGINE", "node");
            command
        }
        Which::Argus => {
            let mut command = Command::new(&engines.argus);
            command.arg("coord");
            command
        }
    }
}

/// The environment of a call: the homes, a dashboard that is down and will not be started,
/// the session id of the step, then the step's own variables.
fn apply_env(command: &mut Command, home: &Home, port: u16, step: &Step) {
    for name in SCRUBBED {
        command.env_remove(name);
    }
    command
        .env("COORD_HOME", home.coord())
        .env("DASHBOARD_HOME", home.dashboard())
        .env("DASHBOARD_PORT", port.to_string())
        .env("COORD_DASHBOARD_NO_AUTOSTART", "1");
    if let Some(session) = &step.session {
        command.env("CLAUDE_CODE_SESSION_ID", session);
    }
    for (name, value) in &step.env {
        command.env(name, value);
    }
}
