//! Tree kill. A run is a shell plus whatever it started, so ending the pid alone would leave
//! grandchildren running. Windows: `taskkill /PID <pid> /T`, then `/T /F` after the grace
//! period (or at once when the soft attempt is refused: a hidden console process has no
//! window to close, so taskkill without /F fails for it). POSIX: the run leads its own
//! process group, so a negative pid signals the whole group. Always scoped to one pid and
//! its descendants, never to an image name.

use std::time::Duration;

use argus_core::process::hidden_command;

use sysinfo::{Pid, ProcessesToUpdate, System};
use tokio::time::{Instant, sleep};

const POLL: Duration = Duration::from_millis(50);
const FORCE_WAIT: Duration = Duration::from_secs(3);

/// True while a process with this pid exists.
pub(crate) fn process_alive(pid: u32) -> bool {
    let target = Pid::from_u32(pid);
    let mut system = System::new();
    system.refresh_processes(ProcessesToUpdate::Some(&[target]), true);
    system.process(target).is_some()
}

/// True when `pid` is alive and, on Windows, is a `cmd.exe` like the shell of a run. A
/// recycled pid that belongs to something else is not ours to kill.
pub(crate) fn is_run_shell(pid: u32) -> bool {
    let target = Pid::from_u32(pid);
    let mut system = System::new();
    system.refresh_processes(ProcessesToUpdate::Some(&[target]), true);
    system.process(target).is_some_and(|process| {
        !cfg!(windows)
            || process
                .name()
                .to_string_lossy()
                .eq_ignore_ascii_case("cmd.exe")
    })
}

async fn wait_gone(pid: u32, limit: Duration) -> bool {
    let deadline = Instant::now() + limit;
    while process_alive(pid) {
        if Instant::now() >= deadline {
            return false;
        }
        sleep(POLL).await;
    }
    true
}

#[cfg(windows)]
async fn taskkill(pid: u32, force: bool) -> bool {
    use std::process::Stdio;

    let mut command = tokio::process::Command::from(hidden_command("taskkill"));
    command.args(["/PID", &pid.to_string(), "/T"]);
    if force {
        command.arg("/F");
    }
    command
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null());
    command.status().await.is_ok_and(|status| status.success())
}

/// Ends the tree of `pid`: politely first, forced after `grace`.
#[cfg(windows)]
pub(crate) async fn end_tree(pid: u32, grace: Duration) {
    if !process_alive(pid) {
        return;
    }
    if taskkill(pid, false).await && wait_gone(pid, grace).await {
        return;
    }
    taskkill(pid, true).await;
    wait_gone(pid, FORCE_WAIT).await;
}

#[cfg(not(windows))]
async fn signal(pid: u32, name: &str) {
    use std::process::Stdio;

    let send = |target: String| async move {
        tokio::process::Command::from(hidden_command("kill"))
            .args([format!("-{name}"), "--".to_owned(), target])
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status()
            .await
            .is_ok_and(|status| status.success())
    };
    if !send(format!("-{pid}")).await {
        send(pid.to_string()).await;
    }
}

/// Ends the process group of `pid`: SIGTERM first, SIGKILL after `grace`.
#[cfg(not(windows))]
pub(crate) async fn end_tree(pid: u32, grace: Duration) {
    if !process_alive(pid) {
        return;
    }
    signal(pid, "TERM").await;
    if wait_gone(pid, grace).await {
        return;
    }
    signal(pid, "KILL").await;
    wait_gone(pid, FORCE_WAIT).await;
}
