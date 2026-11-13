// test-tag: critical
//! T6: the data-loss review: the marker survives a refused rerun, a rollback repairs a board whose
//! folder moved but whose file kept the id, a rollback that would collide changes nothing, and a
//! lost plan file does not draw new ids.

use std::fs;

use argus_core::boards;
use argus_migrate::{Fault, Mode, run};
use rusqlite::Connection;

use crate::fixture::{Fixture, snapshot};

fn id_of(fx: &Fixture, key: &str) -> String {
    let table = boards::load(&fx.home).unwrap();
    let found = table
        .boards
        .iter()
        .find(|(_, e)| e.aliases.iter().any(|a| a == key));
    found.map(|(id, _)| id.clone()).unwrap()
}

fn table_of(fx: &Fixture) -> Vec<(String, Vec<String>)> {
    let table = boards::load(&fx.home).unwrap();
    table
        .boards
        .iter()
        .map(|(id, e)| (id.clone(), e.aliases.clone()))
        .collect()
}

fn marker(fx: &Fixture) -> bool {
    fx.home.join("MIGRATING").exists()
}

#[test]
fn a_refused_rerun_of_a_half_migrated_home_keeps_the_marker() {
    let fx = Fixture::new();
    assert!(run(&fx.faulty(Mode::Migrate, Some(Fault::AfterBoard(1)))).is_err());
    let board = fx.backup_dir().join("alpha").join("board.json");
    let mut bytes = fs::read(&board).unwrap();
    bytes.push(b'!');
    fs::write(&board, bytes).unwrap();
    let message = run(&fx.options(Mode::Migrate)).unwrap_err().to_string();
    assert!(marker(&fx));
    assert!(message.contains("marker stays"), "{message}");
}

#[test]
fn a_rollback_repairs_a_folder_that_moved_without_its_file() {
    let fx = Fixture::new();
    run(&fx.options(Mode::Migrate)).unwrap();
    let id = id_of(&fx, "alpha");
    fs::rename(fx.home.join(&id), fx.home.join("alpha")).unwrap();
    run(&fx.options(Mode::Rollback)).unwrap();
    let text = fs::read_to_string(fx.home.join("alpha").join("board.json")).unwrap();
    let board: serde_json::Value = serde_json::from_str(&text).unwrap();
    assert_eq!(board["project"], "alpha");
}

#[test]
fn a_rollback_that_would_collide_stops_before_any_write() {
    let fx = Fixture::new();
    run(&fx.options(Mode::Migrate)).unwrap();
    let conn = Connection::open(fx.home.join("dashboard").join("dashboard.db")).unwrap();
    conn.execute("INSERT INTO kv(key,value) VALUES ('run:alpha','1')", [])
        .unwrap();
    drop(conn);
    let before = snapshot::take(&fx.home);
    let message = run(&fx.options(Mode::Rollback)).unwrap_err().to_string();
    assert!(message.contains("run:alpha"), "{message}");
    assert_eq!(snapshot::take(&fx.home), before);
    assert!(fx.home.join("board-ids.journal.json").exists());
    assert!(!marker(&fx));
}

#[test]
fn a_lost_plan_file_after_a_partial_run_draws_no_new_id() {
    let fx = Fixture::new();
    assert!(run(&fx.faulty(Mode::Migrate, Some(Fault::AfterBoard(2)))).is_err());
    let before = table_of(&fx);
    fs::remove_file(fx.home.join("board-ids.plan.json")).unwrap();
    run(&fx.options(Mode::Migrate)).unwrap();
    assert_eq!(table_of(&fx), before);
}
