//! The real process host: `tokio::process` with the console hidden on Windows.

use std::future::Future;
use std::io;
use std::process::{ExitStatus, Stdio};
use std::time::Duration;

use argus_core::process::{CREATE_NO_WINDOW, hidden_command};

use tokio::process::Command;
use tokio::sync::oneshot;

use crate::host::{
    BoxRead, BoxWrite, ExitInfo, ProcessHost, SpawnOptions, SpawnRequest, SpawnedProcess,
    spawn_options,
};
use crate::tree_kill::{end_tree, is_run_shell, process_alive};

#[derive(Clone, Copy, Debug, Default)]
pub struct SystemHost;

#[cfg(windows)]
fn shell_command(request: &SpawnRequest, options: &SpawnOptions) -> Command {
    use std::os::windows::process::CommandExt;

    const DETACHED_PROCESS: u32 = 0x0000_0008;
    const CREATE_NEW_PROCESS_GROUP: u32 = 0x0000_0200;
    let program = std::env::var("ComSpec").unwrap_or_else(|_| options.shell.to_owned());
    let flags = if options.hide_window {
        CREATE_NO_WINDOW
    } else if options.detached {
        DETACHED_PROCESS | CREATE_NEW_PROCESS_GROUP
    } else {
        0
    };
    let mut command = Command::from(hidden_command(program));
    command
        .as_std_mut()
        .args(options.shell_args)
        .raw_arg(format!("\"{}\"", request.cmd))
        .creation_flags(flags);
    command
}

#[cfg(not(windows))]
fn shell_command(request: &SpawnRequest, options: &SpawnOptions) -> Command {
    use std::os::unix::process::CommandExt;

    let mut command = Command::from(hidden_command(options.shell));
    command
        .as_std_mut()
        .args(options.shell_args)
        .arg(&request.cmd);
    command.as_std_mut().process_group(0);
    command
}

fn exit_info(status: ExitStatus) -> ExitInfo {
    ExitInfo {
        code: status.code(),
        signal: signal_name(status),
    }
}

#[cfg(unix)]
fn signal_name(status: ExitStatus) -> Option<String> {
    use std::os::unix::process::ExitStatusExt;

    status.signal().map(|n| format!("SIG{n}"))
}

#[cfg(not(unix))]
fn signal_name(_status: ExitStatus) -> Option<String> {
    None
}

fn missing_pipe(name: &str) -> io::Error {
    io::Error::other(format!("the {name} pipe of the process is missing"))
}

impl ProcessHost for SystemHost {
    fn spawn(&self, request: &SpawnRequest) -> io::Result<SpawnedProcess> {
        let options = spawn_options(request);
        let mut command = shell_command(request, &options);
        command
            .current_dir(&request.cwd)
            .envs(&request.env)
            .stdin(if options.stdin_pipe {
                Stdio::piped()
            } else {
                Stdio::null()
            })
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());
        let mut child = command.spawn()?;
        let stdout: BoxRead = Box::new(child.stdout.take().ok_or_else(|| missing_pipe("stdout"))?);
        let stderr: BoxRead = Box::new(child.stderr.take().ok_or_else(|| missing_pipe("stderr"))?);
        let stdin = child.stdin.take().map(|s| Box::new(s) as BoxWrite);
        let pid = child.id();
        let (report, exit) = oneshot::channel();
        tokio::spawn(async move {
            let info = child.wait().await.map(exit_info).unwrap_or_default();
            let _ = report.send(info);
        });
        Ok(SpawnedProcess {
            pid,
            stdout,
            stderr,
            stdin,
            exit,
        })
    }

    async fn kill_tree(&self, pid: u32, grace: Duration) {
        end_tree(pid, grace).await;
    }

    fn is_alive(&self, pid: u32) -> bool {
        process_alive(pid)
    }

    fn looks_like_ours(&self, pid: u32) -> impl Future<Output = bool> + Send {
        std::future::ready(is_run_shell(pid))
    }
}
