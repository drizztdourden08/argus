//! Restart safety: at boot nothing stays "running" or "queued". Rows are written the way a
//! crashed server would have left them; "pid still alive" and Stop-kills-the-orphan go
//! through the fake host (runs.recover; the real processes are in `system_host`).

use std::collections::BTreeMap;
use std::sync::Arc;

use argus_launchers::{FakeHost, MemoryStore, ProcessHost, RunEvents, RunStatus, SpawnRequest};

use crate::common::{Env, EnvOptions, left_behind, page, unbooted};

type Left<'a> = (&'a str, RunStatus, Option<u32>);

/// A process the fake host started before the "crash": alive, and a shell of ours.
fn orphan(host: &FakeHost) -> u32 {
    let request = SpawnRequest {
        cmd: "orphan".to_owned(),
        cwd: ".".into(),
        env: BTreeMap::new(),
        visible: false,
        stdin: false,
    };
    drop(host.spawn(&request).unwrap());
    *host.pids().last().unwrap()
}

/// A service that booted over the rows a crash left behind.
fn restarted(host: &FakeHost, rows: &[Left<'_>]) -> Env {
    let store = Arc::new(MemoryStore::new());
    for (rid, status, pid) in rows {
        left_behind(&store, rid, *status, *pid);
    }
    let env = unbooted(&store, host);
    env.svc.recover();
    env
}

const GONE: Option<u32> = Some(2_147_483_000);

/// The service after a crash that left a live orphan (R-1), one whose pid is gone (R-2), a
/// queued run (R-3) and a running run with no pid (R-4); with the host and R-1's pid.
fn crashed() -> (Env, FakeHost, u32) {
    let host = FakeHost::new();
    let live = orphan(&host);
    let rows = [
        ("R-1", RunStatus::Running, Some(live)),
        ("R-2", RunStatus::Running, GONE),
        ("R-3", RunStatus::Queued, None),
        ("R-4", RunStatus::Running, None),
    ];
    (restarted(&host, &rows), host, live)
}

#[tokio::test(start_paused = true)]
async fn boot_fails_queued_rows_marks_running_rows_orphan_and_says_whether_the_pid_lives() {
    let (env, _host, live) = crashed();
    let row = |rid: &str| env.svc.get("alpha", rid).unwrap();
    let note = format!("server restarted while running (pid {live} still alive; Stop kills it)");
    assert_eq!(
        (row("R-1").status, row("R-1").orphan),
        (RunStatus::Failed, true)
    );
    assert_eq!(row("R-1").error.as_deref(), Some(note.as_str()));
    assert!(row("R-1").ended_at.is_some());
    assert_eq!(
        (row("R-2").status, row("R-2").orphan),
        (RunStatus::Failed, true)
    );
    assert_eq!(
        row("R-2").error.as_deref(),
        Some("server restarted while running")
    );
    assert_eq!(
        (row("R-3").status, row("R-3").orphan),
        (RunStatus::Failed, false)
    );
    assert_eq!(
        row("R-3").error.as_deref(),
        Some("server restarted before start")
    );
    assert_eq!(
        row("R-4").error.as_deref(),
        Some("server restarted while running")
    );
    assert_eq!(env.events.doc("alpha", "R-1").unwrap()["orphan"], true);
    assert_eq!(env.events.doc("alpha", "R-3").unwrap()["status"], "failed");
}

#[tokio::test(start_paused = true)]
async fn stop_on_an_orphan_kills_the_recorded_pid_tree_and_clears_the_flag() {
    let (env, host, live) = crashed();
    let stopped = env.svc.stop("alpha", "R-1", "tester").await.unwrap();
    assert!(!stopped.orphan);
    assert!(stopped.error.unwrap().ends_with("killed by tester"));
    assert!(!host.is_alive(live));
    let gone = env.svc.stop("alpha", "R-2", "tester").await.unwrap();
    assert!(gone.orphan, "an orphan whose pid is gone changes nothing");
}

#[tokio::test(start_paused = true)]
async fn a_recycled_pid_that_is_not_a_run_shell_is_never_killed() {
    let host = FakeHost::new();
    host.add_foreign(777);
    let env = restarted(&host, &[("R-1", RunStatus::Running, Some(777))]);
    let row = env.svc.stop("alpha", "R-1", "tester").await.unwrap();
    assert!(row.orphan, "left alone");
    assert!(host.is_alive(777), "the unrelated process survives");
    assert!(host.killed().is_empty());
}

#[tokio::test(start_paused = true)]
async fn boot_rebuilds_the_run_docs_and_drops_docs_of_runs_that_no_longer_exist() {
    let (store, host) = (Arc::new(MemoryStore::new()), FakeHost::new());
    left_behind(&store, "R-1", RunStatus::Running, None);
    let env = unbooted(&store, &host);
    let ghost = left_behind(&MemoryStore::new(), "R-77", RunStatus::Running, None);
    RunEvents::project(env.events.as_ref(), &ghost);
    env.svc.recover();
    let docs = env.events.current("alpha");
    assert_eq!(docs.len(), 1);
    assert_eq!(
        (docs[0]["id"].as_str(), docs[0]["status"].as_str()),
        (Some("R-1"), Some("failed"))
    );
}

#[tokio::test(start_paused = true)]
async fn after_a_restart_the_counters_continue_and_new_runs_work() {
    let (store, host) = (Arc::new(MemoryStore::new()), FakeHost::new());
    let first = Env::build(EnvOptions {
        store: Some(Arc::clone(&store)),
        host: Some(host.clone()),
        ..EnvOptions::default()
    });
    first.define("alpha", vec![first.def("quick", "quick")]);
    first.start("alpha", "quick");
    host.exit(host.pids()[0], Some(0)).await;
    first.finished("alpha", "R-1").await;
    first.svc.shutdown().await;
    let second = unbooted(&store, &host);
    second.define("alpha", vec![second.def("quick", "quick")]);
    second.svc.recover();
    let run = second.start("alpha", "quick");
    assert_eq!(run.id, "R-2");
    host.exit(host.pids()[1], Some(0)).await;
    second.finished("alpha", "R-2").await;
}

#[tokio::test(start_paused = true)]
async fn shutdown_leaves_a_visible_run_alone_as_an_orphan_that_a_new_service_can_stop() {
    let env = Env::new();
    let app = argus_launchers::RunDef {
        hand: true,
        visible: true,
        ..env.def("app", "app")
    };
    env.define("alpha", vec![app]);
    let run = env.svc.start("alpha", "app", &page()).unwrap().run;
    assert!(run.visible);
    let pid = run.pid.unwrap();
    env.svc.shutdown().await;
    let row = env.svc.get("alpha", "R-1").unwrap();
    assert_eq!((row.status, row.orphan), (RunStatus::Failed, true));
    let note = format!("server stopped while running (pid {pid} still alive; Stop kills it)");
    assert_eq!(row.error.as_deref(), Some(note.as_str()));
    assert!(env.host.is_alive(pid), "the visible run keeps running");
    let next = unbooted(&env.store, &env.host);
    next.svc.recover();
    let stopped = next.svc.stop("alpha", "R-1", "tester").await.unwrap();
    assert!(!stopped.orphan);
    assert!(!env.host.is_alive(pid));
}
