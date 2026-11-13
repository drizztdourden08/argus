//! T5: the guards: a live server stops the run, a dry run changes nothing but the plan, a restore
//! puts the backup back.

use std::fs;

use argus_migrate::{Mode, Options, run};

use crate::fixture::{Fixture, snapshot};

fn alive(_pid: u32) -> bool {
    true
}

// test-tag: critical
#[test]
fn a_live_server_refuses_the_run() {
    let fx = Fixture::new();
    fs::write(
        fx.home.join("dashboard").join("server.pid"),
        r#"{"pid":4242,"port":47615}"#,
    )
    .unwrap();
    let before = snapshot::take(&fx.home);
    let opts = Options {
        pid_alive: alive,
        ..fx.options(Mode::Migrate)
    };
    assert!(run(&opts).is_err());
    assert_eq!(snapshot::take(&fx.home), before);
}

#[test]
fn a_dry_run_writes_only_the_plan_and_a_rerun_reuses_its_ids() {
    let fx = Fixture::new();
    let before = snapshot::take(&fx.home);
    run(&fx.options(Mode::DryRun)).unwrap();
    assert_eq!(snapshot::take(&fx.home), before);
    let plan = fs::read_to_string(fx.home.join("board-ids.plan.json")).unwrap();
    run(&fx.options(Mode::DryRun)).unwrap();
    assert_eq!(
        fs::read_to_string(fx.home.join("board-ids.plan.json")).unwrap(),
        plan
    );
}

#[test]
fn a_restore_puts_the_backup_back() {
    let fx = Fixture::new();
    let before = snapshot::take(&fx.home);
    run(&fx.options(Mode::Migrate)).unwrap();
    let backup = fx.backup_dir();
    run(&fx.options(Mode::Restore(backup))).unwrap();
    assert_eq!(snapshot::take(&fx.home), before);
}
