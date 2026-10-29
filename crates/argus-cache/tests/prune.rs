//! The prune rule of the contract on temporary cache roots and temporary project folders.

mod rig;

use argus_cache::{Kind, PlaceKind, PruneError};
use rig::{DAY, Rig};

const CORE: &str = "@drizztdourden08+brock-core";
const LOCK_V9: &str = "lockfileVersion: '9.0'\n\nimporters:\n\n  .:\n    dependencies:\n      '@drizztdourden08/brock-core':\n        specifier: ^0.115.0\n        version: 0.115.3(react@19.0.0)\n\npackages:\n\n  '@drizztdourden08/brock-core@0.115.3':\n    resolution: {integrity: sha512-x}\n\n  electron@42.11.8:\n    resolution: {integrity: sha512-y}\n\nsnapshots:\n\n  '@drizztdourden08/brock-core@0.115.3(react@19.0.0)': {}\n";

#[test]
fn an_entry_a_project_resolves_stays_however_old_it_is() {
    let rig = Rig::new();
    rig.entry("package", CORE, "0.115.3", "node-aa", 100, 100 * DAY);
    rig.project("brock", LOCK_V9);
    let outcome = rig.prune(14).expect("prune");
    assert_eq!(outcome.deleted, Vec::new(), "{outcome:?}");
    assert!(rig.exists("package", CORE, "0.115.3", "node-aa"));
    let report = rig.report(14);
    let row = &report.rows[0];
    assert!(row.in_use);
    assert_eq!(row.users[0].label, "brock");
    assert_eq!(row.users[0].kind, PlaceKind::Project);
    assert_eq!(row.delete_after, None);
}

#[test]
fn an_unused_entry_inside_the_grace_period_stays_and_says_when_it_goes() {
    let rig = Rig::new();
    rig.entry("package", CORE, "0.114.0", "node-aa", 10, 3 * DAY);
    rig.project("brock", LOCK_V9);
    let outcome = rig.prune(14).expect("prune");
    assert_eq!(outcome.deleted, Vec::new());
    assert!(rig.exists("package", CORE, "0.114.0", "node-aa"));
    let report = rig.report(14);
    assert!(!report.rows[0].in_use);
    assert_eq!(report.rows[0].prunable, 0);
    assert_eq!(
        report.rows[0].delete_after.as_deref(),
        Some(rig.iso_days_ago(3 - 14).as_str())
    );
}

#[test]
fn an_unused_entry_past_the_grace_period_goes_whole_with_its_empty_folders() {
    let rig = Rig::new();
    rig.entry("package", CORE, "0.100.0", "node-aa", 700, 20 * DAY);
    rig.entry("package", CORE, "0.100.0", "node-bb", 300, 30 * DAY);
    rig.project("brock", LOCK_V9);
    let outcome = rig.prune(14).expect("prune");
    assert_eq!(outcome.deleted.len(), 2);
    assert_eq!(outcome.freed_bytes, 1000);
    assert_eq!(outcome.deleted[0].kind, Kind::Package);
    assert!(!rig.exists("package", CORE, "0.100.0", "node-aa"));
    assert!(!rig.version_folder("package", CORE, "0.100.0").exists());
    assert!(!rig.name_folder("package", CORE).exists());
    assert!(rig.root_path().join("package").exists());
    assert!(outcome.summary().contains("2 entries"));
    assert!(outcome.summary().contains("temp folders"));
}

#[test]
fn a_version_keeps_its_folder_while_one_build_key_is_inside_the_grace_period() {
    let rig = Rig::new();
    rig.entry("package", CORE, "0.100.0", "node-old", 10, 40 * DAY);
    rig.entry("package", CORE, "0.100.0", "node-new", 10, DAY);
    let outcome = rig.prune(14).expect("prune");
    assert_eq!(outcome.deleted.len(), 1);
    assert_eq!(outcome.deleted[0].build_key, "node-old");
    assert!(rig.exists("package", CORE, "0.100.0", "node-new"));
    assert!(!rig.exists("package", CORE, "0.100.0", "node-old"));
}

#[test]
fn a_worktree_lockfile_keeps_the_electron_entry_it_resolves() {
    let rig = Rig::new();
    rig.entry(
        "electron",
        "electron",
        "42.11.8",
        "win32-x64",
        5000,
        90 * DAY,
    );
    rig.entry(
        "electron",
        "electron",
        "41.0.0",
        "win32-x64",
        4000,
        90 * DAY,
    );
    rig.worktree("brock", "agent-1", LOCK_V9, true);
    let outcome = rig.prune(14).expect("prune");
    assert_eq!(outcome.deleted.len(), 1);
    assert_eq!(outcome.deleted[0].version, "41.0.0");
    assert!(rig.exists("electron", "electron", "42.11.8", "win32-x64"));
    let kept = rig
        .report(14)
        .rows
        .into_iter()
        .find(|r| r.version == "42.11.8");
    let users = kept.expect("row").users;
    assert_eq!(users[0].label, "brock / agent-1");
    assert_eq!(users[0].kind, PlaceKind::Worktree);
}

#[test]
fn a_name_resolved_at_another_version_does_not_keep_the_entry() {
    let rig = Rig::new();
    rig.entry("package", CORE, "0.115.4", "node-aa", 10, 60 * DAY);
    rig.project("brock", LOCK_V9);
    let outcome = rig.prune(14).expect("prune");
    assert_eq!(outcome.deleted.len(), 1);
}

#[test]
fn old_temp_folders_go_and_young_ones_stay() {
    let rig = Rig::new();
    rig.tmp("old-build", 2 * DAY);
    rig.tmp("young-build", DAY / 4);
    let outcome = rig.prune(14).expect("prune");
    assert_eq!(outcome.tmp_deleted, 1);
    assert!(!rig.root_path().join(".tmp/old-build").exists());
    assert!(rig.root_path().join(".tmp/young-build").exists());
    let report = rig.report(14);
    assert_eq!(report.totals.stale_tmp, 0);
}

#[test]
fn the_report_counts_what_a_prune_would_delete() {
    let rig = Rig::new();
    rig.entry("package", CORE, "0.100.0", "node-aa", 700, 20 * DAY);
    rig.entry("package", CORE, "0.115.3", "node-aa", 100, 20 * DAY);
    rig.tmp("old-build", 2 * DAY);
    rig.project("brock", LOCK_V9);
    let totals = rig.report(14).totals;
    assert_eq!((totals.versions, totals.entries), (2, 2));
    assert_eq!((totals.prunable_entries, totals.prunable_bytes), (1, 700));
    assert_eq!((totals.in_use_bytes, totals.unused_bytes), (100, 700));
    assert_eq!(totals.stale_tmp, 1);
    assert!(rig.exists("package", CORE, "0.100.0", "node-aa"));
}

#[test]
fn a_lockfile_that_cannot_be_read_stops_the_prune() {
    let rig = Rig::new();
    rig.entry("package", CORE, "0.100.0", "node-aa", 10, 60 * DAY);
    let place = rig.project("broken", "");
    std::fs::remove_file(place.join("pnpm-lock.yaml")).expect("remove");
    std::fs::create_dir(place.join("pnpm-lock.yaml")).expect("a folder in its place");
    let Err(PruneError::Unreadable(why)) = rig.prune(14) else {
        panic!("the prune ran");
    };
    assert!(why.contains("pnpm-lock.yaml"), "{why}");
    assert!(rig.exists("package", CORE, "0.100.0", "node-aa"));
    assert_eq!(rig.report(14).problems.len(), 1);
}

#[test]
fn folders_with_unsafe_names_are_not_entries() {
    let rig = Rig::new();
    rig.entry("package", "odd name", "1.0.0", "node-aa", 10, 60 * DAY);
    rig.entry("package", CORE, "1 0", "node-aa", 10, 60 * DAY);
    let outcome = rig.prune(14).expect("prune");
    assert_eq!(outcome.deleted, Vec::new());
    assert!(rig.name_folder("package", "odd name").exists());
}

#[test]
fn an_entry_without_a_manifest_is_judged_by_its_folder_time() {
    let rig = Rig::new();
    rig.bare_entry("package", CORE, "0.1.0", "node-aa", 60 * DAY);
    let report = rig.report(14);
    assert_eq!(report.rows[0].size_bytes, 3);
    let outcome = rig.prune(14).expect("prune");
    assert_eq!(outcome.deleted.len(), 1);
}

#[test]
fn a_missing_cache_is_an_empty_report_and_an_empty_prune() {
    let rig = Rig::without_cache();
    let report = rig.report(14);
    assert!(!report.exists);
    assert_eq!(report.rows, Vec::new());
    assert!(rig.prune(14).expect("prune").is_quiet());
}
