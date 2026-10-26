//! Stopping: the tree is ended through the host, concurrent stops agree, shutdown stops
//! hidden runs, and a shell that exited while a grandchild still holds the pipes does not
//! keep the run "running" (runs.kill, on the fake host; the real tree kill is in
//! `system_host`).

use argus_launchers::{ProcessHost, RunStatus};

use crate::common::{Env, cli};

#[tokio::test(start_paused = true)]
async fn stop_ends_the_tree_through_the_host_and_records_who() {
    let env = Env::new();
    env.define("alpha", vec![env.def("tree", "tree")]);
    let run = env.start("alpha", "tree");
    let pid = run.pid.unwrap();
    let stopped = env.svc.stop("alpha", &run.id, "tester").await.unwrap();
    assert_eq!(stopped.status, RunStatus::Stopped);
    assert_eq!(stopped.by, "tester");
    assert_eq!(stopped.error, None);
    assert_eq!(env.host.killed(), [pid]);
    assert!(!env.host.is_alive(pid));
    let text = env.text("alpha", &run.id, None);
    assert!(text.contains("stop requested by tester") && text.contains("stopped by tester"));
    assert_eq!(
        env.events.doc("alpha", &run.id).unwrap()["status"],
        "stopped"
    );
}

#[tokio::test(start_paused = true)]
async fn two_concurrent_stops_agree_and_stopping_a_finished_run_changes_nothing() {
    let env = Env::new();
    env.define("alpha", vec![env.def("slow", "slow")]);
    let run = env.start("alpha", "slow");
    let (a, b) = tokio::join!(
        env.svc.stop("alpha", &run.id, "one"),
        env.svc.stop("alpha", &run.id, "two")
    );
    let (a, b) = (a.unwrap(), b.unwrap());
    assert_eq!(
        (a.status, b.status),
        (RunStatus::Stopped, RunStatus::Stopped)
    );
    assert_eq!(a.by, "one", "the first stop is the one recorded");
    let again = env.svc.stop("alpha", &run.id, "three").await.unwrap();
    assert_eq!(
        (again.status, again.by.as_str()),
        (RunStatus::Stopped, "one")
    );
    assert_eq!(env.host.killed().len(), 1, "the tree is ended once");
    assert_eq!(
        env.svc.stop("alpha", "R-99", "x").await.unwrap_err().code,
        "unknown_run"
    );
    assert_eq!(
        env.svc
            .stop("alpha", "nonsense", "x")
            .await
            .unwrap_err()
            .code,
        "bad_request"
    );
}

#[tokio::test(start_paused = true)]
async fn shutdown_stops_a_hidden_run_records_server_shutdown_and_refuses_new_starts() {
    let env = Env::new();
    env.define("alpha", vec![env.def("tree", "tree")]);
    let run = env.start("alpha", "tree");
    env.svc.shutdown().await;
    let row = env.svc.get("alpha", &run.id).unwrap();
    assert_eq!(row.status, RunStatus::Stopped);
    assert_eq!(row.by, "server shutdown");
    assert_eq!(env.host.killed(), [run.pid.unwrap()]);
    let refused = env.svc.start("alpha", "tree", &cli()).unwrap_err();
    assert_eq!((refused.status, refused.code), (503, "unavailable"));
}

#[tokio::test(start_paused = true)]
async fn shutdown_fails_the_runs_still_queued() {
    let env = Env::with_limits(argus_launchers::RunLimits {
        max_concurrent: 0,
        ..argus_launchers::RunLimits::default()
    });
    env.define("alpha", vec![env.def("slow", "slow")]);
    let run = env.start("alpha", "slow");
    env.svc.shutdown().await;
    let row = env.svc.get("alpha", &run.id).unwrap();
    assert_eq!(row.status, RunStatus::Failed);
    assert_eq!(
        row.error.as_deref(),
        Some("the dashboard stopped before this run started")
    );
}

#[tokio::test(start_paused = true)]
async fn a_shell_that_exits_while_a_grandchild_holds_the_pipes_still_finishes_the_run() {
    let env = Env::new();
    env.define("alpha", vec![env.def("leaky", "leaky")]);
    let run = env.start("alpha", "leaky");
    let pid = run.pid.unwrap();
    env.host.say(pid, "GC 4242\n").await;
    env.host.exit_keeping_pipes(pid, Some(0));
    let done = env.finished("alpha", &run.id).await;
    assert_eq!(done.status, RunStatus::Exited);
    assert!(env.text("alpha", &run.id, Some("out")).contains("GC 4242"));
}
