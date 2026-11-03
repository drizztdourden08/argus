//! The global concurrency limit and its FIFO queue, a queued run that is stopped, the
//! timeout, and retention (runs.lifecycle).
//!
//! Keep rule: S2.
// test-tag: standard

use argus_launchers::{RunLimits, RunStatus};

use crate::common::{Env, PNG, cli, until};

fn one_slot() -> Env {
    Env::with_limits(RunLimits {
        max_concurrent: 1,
        ..RunLimits::default()
    })
}

#[tokio::test(start_paused = true)]
async fn the_global_limit_queues_and_a_freed_slot_starts_the_oldest_queued_run_first() {
    let env = one_slot();
    env.define("alpha", vec![env.def("one", "one"), env.def("two", "two")]);
    env.define("beta", vec![env.def("three", "three")]);
    let one = env.start("alpha", "one");
    let two = env.start("alpha", "two");
    let three = env.start("beta", "three");
    assert_eq!(one.status, RunStatus::Running);
    assert_eq!(two.status, RunStatus::Queued);
    assert_eq!(three.status, RunStatus::Queued);
    assert_eq!(env.events.doc("alpha", "R-2").unwrap()["status"], "queued");
    env.host.exit(env.pid(0), Some(0)).await;
    env.finished("alpha", "R-1").await;
    until("the second run to start", || {
        (env.host.pids().len() == 2).then_some(())
    })
    .await;
    assert_eq!(
        env.svc.get("beta", "R-1").unwrap().status,
        RunStatus::Queued
    );
    env.host.exit(env.pid(1), Some(0)).await;
    let second = env.finished("alpha", "R-2").await;
    until("the third run to start", || {
        (env.host.pids().len() == 3).then_some(())
    })
    .await;
    env.host.exit(env.pid(2), Some(0)).await;
    let third = env.finished("beta", "R-1").await;
    assert_eq!(second.status, RunStatus::Exited);
    assert_eq!(third.status, RunStatus::Exited);
    assert!(second.started_at <= third.started_at, "FIFO across scopes");
}

#[tokio::test(start_paused = true)]
async fn stopping_a_queued_run_ends_it_without_ever_starting_a_process() {
    let env = one_slot();
    env.define(
        "alpha",
        vec![env.def("slow", "slow"), env.def("quick", "quick")],
    );
    env.start("alpha", "slow");
    let queued = env.start("alpha", "quick");
    assert_eq!(queued.status, RunStatus::Queued);
    let stopped = env.svc.stop("alpha", "R-2", "tester").await.unwrap();
    assert_eq!(stopped.status, RunStatus::Stopped);
    assert!(stopped.started_at.is_none() && stopped.pid.is_none());
    assert_eq!(stopped.by, "tester");
    assert_eq!(env.host.pids().len(), 1, "no second process");
    env.svc.stop("alpha", "R-1", "tester").await.unwrap();
}

#[tokio::test(start_paused = true)]
async fn a_timeout_stops_the_run_and_marks_it_failed_with_error_timeout() {
    let env = Env::with_limits(RunLimits {
        stop_grace_ms: 1000,
        ..RunLimits::default()
    });
    let long = argus_launchers::RunDef {
        timeout_sec: 1,
        ..env.def("long", "sleep")
    };
    env.define("alpha", vec![long]);
    env.start("alpha", "long");
    let done = env.finished("alpha", "R-1").await;
    assert_eq!(done.status, RunStatus::Failed);
    assert_eq!(done.error.as_deref(), Some("timeout"));
    assert!(
        env.text("alpha", "R-1", None)
            .contains("timed out after 1s")
    );
    assert_eq!(env.host.killed(), [env.pid(0)]);
}

#[tokio::test(start_paused = true)]
async fn retention_keeps_the_newest_finished_runs_with_their_logs_and_attachments() {
    let env = Env::with_limits(RunLimits {
        keep_per_def: 3,
        ..RunLimits::default()
    });
    env.define("alpha", vec![env.def("quick", "quick")]);
    let mut saved = None;
    for n in 0..5 {
        let run = env.start("alpha", "quick");
        env.host.say(env.pid(n), "x\n").await;
        env.host.exit(env.pid(n), Some(0)).await;
        env.finished("alpha", &run.id).await;
        if n == 0 {
            let info = env
                .svc
                .attach("alpha", "R-1", &PNG, "image/png", "first", "tester");
            saved = Some(info.unwrap());
        }
    }
    let ids: Vec<String> = env.list("alpha").into_iter().map(|r| r.id).collect();
    assert_eq!(ids, ["R-5", "R-4", "R-3"]);
    assert!(env.svc.get("alpha", "R-1").is_none());
    assert_eq!(env.store.stored_lines("alpha", "R-1"), 0);
    assert_eq!(env.store.attachment_count(), 0);
    assert_eq!(env.events.current("alpha").len(), 3);
    assert!(saved.is_some());
    assert!(
        env.dir
            .path()
            .join("attachments")
            .join("alpha")
            .read_dir()
            .unwrap()
            .next()
            .is_none()
    );
}

#[tokio::test(start_paused = true)]
async fn retention_drops_finished_runs_older_than_keep_days() {
    let env = Env::with_limits(RunLimits {
        keep_days: 1,
        ..RunLimits::default()
    });
    env.define("alpha", vec![env.def("quick", "quick")]);
    let run = env.svc.start("alpha", "quick", &cli()).unwrap().run;
    env.host.exit(env.pid(0), Some(0)).await;
    env.finished("alpha", &run.id).await;
    env.store.rewrite("alpha", "R-1", |row| {
        row.ended_at = Some("2020-01-01T00:00:00.000Z".to_owned());
        row.queued_at = "2020-01-01T00:00:00.000Z".to_owned();
    });
    env.svc.sweep();
    assert!(env.list("alpha").is_empty());
}
