//! T3: a run stopped in the middle and run again equals a run that was not stopped.

use std::fs;

use argus_migrate::{Fault, Mode, run};

use crate::fixture::{Fixture, snapshot};

fn plan_file(fx: &Fixture) -> std::path::PathBuf {
    fx.home.join("board-ids.plan.json")
}

fn reference() -> (Fixture, String) {
    let fx = Fixture::new();
    run(&fx.options(Mode::DryRun)).unwrap();
    let plan = fs::read_to_string(plan_file(&fx)).unwrap();
    run(&fx.options(Mode::Migrate)).unwrap();
    (fx, plan)
}

fn stop_and_rerun(plan: &str, fault: Fault) -> Fixture {
    let fx = Fixture::new();
    fs::write(plan_file(&fx), plan).unwrap();
    assert!(run(&fx.faulty(Mode::Migrate, Some(fault))).is_err());
    run(&fx.options(Mode::Migrate)).unwrap();
    fx
}

// test-tag: critical
#[test]
fn a_stopped_run_finished_by_a_rerun_equals_an_uninterrupted_run() {
    let (reference_fx, plan) = reference();
    let want = snapshot::take(&reference_fx.home);
    let faults = [1, 2, 3, 4, 5]
        .map(Fault::AfterBoard)
        .into_iter()
        .chain([Fault::AfterDb]);
    for fault in faults {
        let fx = stop_and_rerun(&plan, fault);
        assert_eq!(snapshot::take(&fx.home), want, "{fault:?}");
        assert_eq!(fx.table(), reference_fx.table(), "{fault:?}");
    }
}

#[test]
fn a_second_full_run_changes_no_file() {
    let (fx, _) = reference();
    let before = snapshot::hashes(&fx.home);
    run(&fx.options(Mode::Migrate)).unwrap();
    assert_eq!(snapshot::hashes(&fx.home), before);
}
