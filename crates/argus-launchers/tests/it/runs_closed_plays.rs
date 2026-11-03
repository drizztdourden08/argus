//! A build that closed with its item has no Play definition: its hidden runs are stopped, and
//! only after the definition stayed missing on two checks in a row (I-78).
//!
//! Keep rule: S2.
// test-tag: critical

use argus_launchers::RunStatus;

use crate::common::Env;

#[tokio::test(start_paused = true)]
async fn a_hidden_play_run_is_stopped_once_its_definition_stays_missing_for_two_checks() {
    let env = Env::new();
    env.define("alpha", vec![env.def("launcher-build-2", "slow")]);
    let run = env.start("alpha", "launcher-build-2");
    env.define("alpha", vec![]);
    assert_eq!(env.svc.stop_closed_plays().await.len(), 0);
    assert_eq!(
        env.svc.get("alpha", &run.id).unwrap().status,
        RunStatus::Running
    );
    assert_eq!(env.svc.stop_closed_plays().await, ["alpha:R-1"]);
    let row = env.svc.get("alpha", &run.id).unwrap();
    assert_eq!(row.status, RunStatus::Stopped);
    assert_eq!(row.by, "build closed");
}

#[tokio::test(start_paused = true)]
async fn a_definition_that_comes_back_between_the_checks_keeps_the_run() {
    let env = Env::new();
    let def = env.def("launcher-build-2", "slow");
    env.define("alpha", vec![def.clone()]);
    let run = env.start("alpha", "launcher-build-2");
    env.define("alpha", vec![]);
    assert_eq!(env.svc.stop_closed_plays().await.len(), 0);
    env.define("alpha", vec![def]);
    assert_eq!(env.svc.stop_closed_plays().await.len(), 0);
    env.define("alpha", vec![]);
    assert_eq!(env.svc.stop_closed_plays().await.len(), 0);
    assert_eq!(
        env.svc.get("alpha", &run.id).unwrap().status,
        RunStatus::Running
    );
}

#[tokio::test(start_paused = true)]
async fn a_run_that_is_not_a_play_is_never_stopped_for_a_missing_definition() {
    let env = Env::new();
    env.define("alpha", vec![env.def("tree", "tree")]);
    let run = env.start("alpha", "tree");
    env.define("alpha", vec![]);
    for _ in 0..3 {
        assert_eq!(env.svc.stop_closed_plays().await.len(), 0);
    }
    assert_eq!(
        env.svc.get("alpha", &run.id).unwrap().status,
        RunStatus::Running
    );
}
