//! T4: a backup that does not check stops the run before any change.

use argus_migrate::{Fault, Mode, run};

use crate::fixture::{Fixture, snapshot};

// test-tag: critical
#[test]
fn a_corrupted_backup_stops_before_any_change() {
    let fx = Fixture::new();
    let before = snapshot::take(&fx.home);
    let result = run(&fx.faulty(Mode::Migrate, Some(Fault::CorruptBackup)));
    assert!(result.is_err());
    assert_eq!(snapshot::take(&fx.home), before);
    assert!(!fx.home.join("MIGRATING").exists());
    assert!(!fx.home.join("boards.json").exists());
}
