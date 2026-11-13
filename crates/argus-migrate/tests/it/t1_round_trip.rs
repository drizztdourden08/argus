//! T1: a migration followed by a rollback gives back the home as it was.

use argus_migrate::{Mode, run};

use crate::fixture::{Fixture, snapshot};

// test-tag: critical
#[test]
fn migrate_then_rollback_restores_every_file_and_row() {
    let fx = Fixture::new();
    let before = snapshot::take(&fx.home);
    run(&fx.options(Mode::Migrate)).unwrap();
    assert_ne!(snapshot::take(&fx.home), before);
    run(&fx.options(Mode::Rollback)).unwrap();
    assert_eq!(snapshot::take(&fx.home), before);
}
