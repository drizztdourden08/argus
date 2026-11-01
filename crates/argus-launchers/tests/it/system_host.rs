//! The real process host on Windows: `cmd.exe /d /s /c`, hidden, with the tree kill scoped
//! to the spawned pid. Only harmless commands run (echo, ping, a node script that ignores the
//! polite signal) and every test kills what it spawned, by pid, in a guard that also runs when an
//! assertion fails. The stubborn-process case ports `runs.kill.test.mjs`.
//!
//! Keep rule: E2.
// test-tag: e2e
#![cfg(windows)]

use std::os::windows::process::CommandExt;
use std::process::{Command, Stdio};
use std::sync::Arc;
use std::time::{Duration, Instant};

use argus_launchers::{
    MemoryDefs, MemoryStore, ProcessHost, RecordingEvents, RunConfig, RunDef, RunDeps, RunService,
    RunStatus, StartOptions, SystemClock, SystemHost,
};
use sysinfo::{Pid, ProcessesToUpdate, System};
use tempfile::TempDir;

use crate::common::{left_behind, until};

const CREATE_NO_WINDOW: u32 = 0x0800_0000;

/// A command that opens no console window and has no stdio.
fn hidden(program: &str) -> Command {
    let mut command = Command::new(program);
    command
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .creation_flags(CREATE_NO_WINDOW);
    command
}

/// Kills the process trees it was given by pid when dropped.
struct Reaper(Vec<u32>);

impl Drop for Reaper {
    fn drop(&mut self) {
        for pid in &self.0 {
            let _ = hidden("taskkill")
                .args(["/PID", &pid.to_string(), "/T", "/F"])
                .status();
        }
    }
}

struct Real {
    svc: RunService<SystemHost>,
    store: Arc<MemoryStore>,
    dir: TempDir,
}

fn real(defs: Vec<RunDef>) -> Real {
    real_with(defs, RunConfig::default())
}

fn real_with(defs: Vec<RunDef>, config: RunConfig) -> Real {
    let dir = TempDir::new().unwrap();
    let store = Arc::new(MemoryStore::new());
    let memory = Arc::new(MemoryDefs::new());
    let cwd = dir.path().to_string_lossy().into_owned();
    memory.set(
        "alpha",
        defs.into_iter()
            .map(|d| RunDef {
                cwd: cwd.clone(),
                ..d
            })
            .collect(),
    );
    let svc = RunService::new(RunDeps {
        host: SystemHost,
        store: Arc::clone(&store) as _,
        events: Arc::new(RecordingEvents::new()),
        defs: memory,
        config: config.fixed(),
        clock: Arc::new(SystemClock),
    });
    svc.recover();
    Real { svc, store, dir }
}

fn def(name: &str, cmd: &str) -> RunDef {
    RunDef {
        name: name.to_owned(),
        cmd: cmd.to_owned(),
        ..RunDef::default()
    }
}

fn start(real: &Real, name: &str) -> argus_launchers::RunRow {
    real.svc
        .start("alpha", name, &StartOptions::default())
        .unwrap()
        .run
}

async fn finished(real: &Real, rid: &str) -> argus_launchers::RunRow {
    until("the run to finish", || {
        real.svc.get("alpha", rid).filter(|r| !r.status.is_live())
    })
    .await
}

fn texts(real: &Real, rid: &str, stream: &str) -> String {
    let options = argus_launchers::LogOptions {
        stream: Some(stream.to_owned()),
        ..argus_launchers::LogOptions::default()
    };
    let lines = real.svc.logs("alpha", rid, &options).unwrap().lines;
    lines
        .into_iter()
        .map(|l| l.text)
        .collect::<Vec<_>>()
        .join("\n")
}

fn child_ping(shell: u32) -> Option<u32> {
    let mut system = System::new();
    system.refresh_processes(ProcessesToUpdate::All, true);
    system
        .processes()
        .values()
        .find(|p| {
            p.parent() == Some(Pid::from_u32(shell))
                && p.name().to_string_lossy().eq_ignore_ascii_case("ping.exe")
        })
        .map(|p| p.pid().as_u32())
}

#[tokio::test]
#[ignore = "e2e"]
async fn a_hidden_echo_exits_zero_and_logs_its_output() {
    let real = real(vec![def("hello", "echo hi")]);
    let run = start(&real, "hello");
    let done = finished(&real, &run.id).await;
    assert_eq!((done.status, done.exit_code), (RunStatus::Exited, Some(0)));
    assert_eq!(texts(&real, &run.id, "out"), "hi");
    assert!(!done.visible && done.pid.is_some());
}

#[tokio::test]
#[ignore = "e2e"]
async fn a_command_the_shell_cannot_find_fails_with_a_message_on_stderr() {
    let real = real(vec![def("nope", "definitely-not-a-command-xyz")]);
    let run = start(&real, "nope");
    let done = finished(&real, &run.id).await;
    assert_eq!(done.status, RunStatus::Failed);
    assert_ne!(done.exit_code, Some(0));
    assert_ne!(texts(&real, &run.id, "err"), "");
}

#[tokio::test]
#[ignore = "e2e"]
async fn the_run_sees_its_names_and_its_definition_env() {
    let mut hello = def(
        "envy",
        "echo %COORD_PROJECT%-%COORD_RUN_ID%-%GREETING%-%COORD_DASHBOARD%",
    );
    hello.env.insert("GREETING".to_owned(), "hello".to_owned());
    let real = real(vec![hello]);
    let run = start(&real, "envy");
    finished(&real, &run.id).await;
    assert_eq!(texts(&real, &run.id, "out"), "alpha-R-1-hello-1");
}

/// A run whose shell has a `ping` under it: the service, the run id, the shell and ping pids,
/// and the guard that kills the tree if an assertion fails.
async fn ping_tree() -> (Real, String, (u32, u32), Reaper) {
    let real = real(vec![def("tree", "ping -n 60 127.0.0.1 >nul")]);
    let run = start(&real, "tree");
    let shell = run.pid.unwrap();
    let reaper = Reaper(vec![shell]);
    let ping = until("the ping under the shell", || child_ping(shell)).await;
    (real, run.id, (shell, ping), reaper)
}

async fn gone((shell, ping): (u32, u32)) {
    until("the tree to die", || {
        (!SystemHost.is_alive(shell) && !SystemHost.is_alive(ping)).then_some(())
    })
    .await;
}

#[tokio::test]
#[ignore = "e2e"]
async fn stop_kills_the_shell_and_the_process_under_it_and_nothing_else() {
    let (real, rid, pids, _reaper) = ping_tree().await;
    assert!(SystemHost.is_alive(pids.0) && SystemHost.is_alive(pids.1));
    let stopped = real.svc.stop("alpha", &rid, "tester").await.unwrap();
    assert_eq!(
        (stopped.status, stopped.by.as_str()),
        (RunStatus::Stopped, "tester")
    );
    gone(pids).await;
}

#[tokio::test]
#[ignore = "e2e"]
async fn shutdown_ends_a_hidden_run_and_leaves_no_process() {
    let (real, rid, pids, _reaper) = ping_tree().await;
    real.svc.shutdown().await;
    let row = real.svc.get("alpha", &rid).unwrap();
    assert_eq!(
        (row.status, row.by.as_str()),
        (RunStatus::Stopped, "server shutdown")
    );
    gone(pids).await;
}

#[allow(clippy::zombie_processes)]
fn detached(program: &str, args: &[&str]) -> u32 {
    hidden(program).args(args).spawn().unwrap().id()
}

fn orphan_row(real: &Real, pid: u32) {
    left_behind(&real.store, "R-1", RunStatus::Running, Some(pid));
}

#[tokio::test]
#[ignore = "e2e"]
async fn an_orphan_shell_is_found_alive_and_stop_kills_it() {
    let shell = detached("cmd.exe", &["/d", "/s", "/c", "ping -n 60 127.0.0.1 >nul"]);
    let _reaper = Reaper(vec![shell]);
    let real = real(Vec::new());
    orphan_row(&real, shell);
    real.svc.recover();
    let row = real.svc.get("alpha", "R-1").unwrap();
    assert!(row.orphan);
    assert!(row.error.unwrap().contains("still alive; Stop kills it"));
    let stopped = real.svc.stop("alpha", "R-1", "tester").await.unwrap();
    assert!(!stopped.orphan);
    until("the orphan to die", || {
        (!SystemHost.is_alive(shell)).then_some(())
    })
    .await;
    assert!(real.dir.path().exists());
}

#[tokio::test]
#[ignore = "e2e"]
async fn a_recycled_pid_that_is_not_a_cmd_shell_is_never_killed() {
    let plain = detached("ping.exe", &["-n", "60", "127.0.0.1"]);
    let _reaper = Reaper(vec![plain]);
    let real = real(Vec::new());
    orphan_row(&real, plain);
    real.svc.recover();
    let row = real.svc.stop("alpha", "R-1", "tester").await.unwrap();
    assert!(row.orphan, "left alone");
    assert!(SystemHost.is_alive(plain), "the unrelated process survives");
}

#[tokio::test]
#[ignore = "e2e"]
async fn a_process_that_ignores_the_polite_signal_is_ended_by_the_forced_kill() {
    let scripts = TempDir::new().unwrap();
    let script = scripts.path().join("stubborn.cjs");
    let body = "process.on('SIGTERM', () => console.log('ignoring')); console.log('ready'); setInterval(() => {}, 1000);";
    std::fs::write(&script, body).unwrap();
    let command = format!("node \"{}\"", script.display());
    let mut config = RunConfig::default();
    config.limits.stop_grace_ms = 600;
    let real = real_with(vec![def("stubborn", &command)], config);
    let run = start(&real, "stubborn");
    let shell = run.pid.unwrap();
    let _reaper = Reaper(vec![shell]);
    until("the process to be ready", || {
        texts(&real, &run.id, "out").contains("ready").then_some(())
    })
    .await;
    let began = Instant::now();
    let stopped = real.svc.stop("alpha", &run.id, "tester").await.unwrap();
    assert_eq!(stopped.status, RunStatus::Stopped);
    assert!(began.elapsed() < Duration::from_secs(5));
    assert!(!SystemHost.is_alive(shell));
}
