//! Process spawning behind a trait. A launch is ONE command line handed to the platform
//! shell (`cmd.exe /d /s /c` on Windows, `sh -c` elsewhere); the command string always comes
//! from a lead's own definition, never from a request.

use std::collections::BTreeMap;
use std::future::Future;
use std::io;
use std::path::PathBuf;
use std::time::Duration;

use tokio::io::{AsyncRead, AsyncWrite};
use tokio::sync::oneshot;

pub type BoxRead = Box<dyn AsyncRead + Send + Unpin>;
pub type BoxWrite = Box<dyn AsyncWrite + Send + Unpin>;

/// A process to start. The child inherits the server's environment; `env` are the
/// variables set on top of it.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SpawnRequest {
    pub cmd: String,
    pub cwd: PathBuf,
    pub env: BTreeMap<String, String>,
    pub visible: bool,
    pub stdin: bool,
}

/// How a process ended.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct ExitInfo {
    pub code: Option<i32>,
    pub signal: Option<String>,
}

/// A started process: its pid, its pipes, and the exit report.
pub struct SpawnedProcess {
    pub pid: Option<u32>,
    pub stdout: BoxRead,
    pub stderr: BoxRead,
    pub stdin: Option<BoxWrite>,
    pub exit: oneshot::Receiver<ExitInfo>,
}

/// What the platform is asked for: the shell, whether a console window may appear, whether
/// the process survives the dashboard, and whether stdin is a pipe.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SpawnOptions {
    pub shell: &'static str,
    pub shell_args: &'static [&'static str],
    pub hide_window: bool,
    pub detached: bool,
    pub stdin_pipe: bool,
}

/// A hidden run opens no console window: its output goes to the dashboard's terminal and a
/// GUI program it starts opens its own window as usual. A visible run (a definition with
/// `window: true`, or a handed build) is detached so it survives a dashboard restart; POSIX
/// runs always lead their own process group so the whole tree can be signalled.
pub fn spawn_options(request: &SpawnRequest) -> SpawnOptions {
    let windows = cfg!(windows);
    SpawnOptions {
        shell: if windows { "cmd.exe" } else { "sh" },
        shell_args: if windows {
            &["/d", "/s", "/c"]
        } else {
            &["-c"]
        },
        hide_window: !request.visible,
        detached: request.visible || !windows,
        stdin_pipe: request.stdin,
    }
}

/// Starts processes and ends their trees. Every kill is scoped to a pid the caller
/// started and its descendants, never to an image name.
pub trait ProcessHost: Send + Sync + 'static {
    /// Starts the process; an error is a start that cannot happen.
    fn spawn(&self, request: &SpawnRequest) -> io::Result<SpawnedProcess>;

    /// Asks the tree of `pid` to end, and forces it after `grace`. Returns when the
    /// shell is gone or the attempt is over.
    fn kill_tree(&self, pid: u32, grace: Duration) -> impl Future<Output = ()> + Send;

    /// True while the process (on POSIX, its group) exists.
    fn is_alive(&self, pid: u32) -> bool;

    /// True when `pid` is alive and is a shell of the kind a run starts, which guards
    /// against a recycled pid when an old row is stopped.
    fn looks_like_ours(&self, pid: u32) -> impl Future<Output = bool> + Send;
}
