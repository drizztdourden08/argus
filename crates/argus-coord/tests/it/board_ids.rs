//! I-153 P3: coord turns every board reference into the board id, and refuses writes while a
//! board migration runs.

// test-tag: critical

use std::fs;

use crate::common::{Home, Ran, board_with, coord};
use serde_json::{Value, json};

const ID: &str = "bk3m7q";

fn table_home() -> Home {
    let home = Home::new();
    home.write_board(ID, &board_with(ID, &[]));
    let table = json!({
        "version": 1,
        "state": "active",
        "boards": {
            ID: {
                "name": "Alpha",
                "aliases": ["alpha"],
                "createdAt": "2026-11-01T00:00:00.000Z"
            }
        }
    });
    fs::write(
        home.coord_home().join("boards.json"),
        serde_json::to_string_pretty(&table).unwrap(),
    )
    .unwrap();
    home
}

fn table(home: &Home) -> Value {
    let text = fs::read_to_string(home.coord_home().join("boards.json")).unwrap();
    serde_json::from_str(&text).unwrap()
}

fn run(home: &Home, argv: &[&str]) -> Ran {
    coord(argv, home.env(), home.path())
}

#[test]
fn an_alias_resolves_to_the_id() {
    let home = table_home();
    let ran = run(&home, &["board", "path", "--project", "alpha"]);
    assert_eq!(ran.code, 0, "{}", ran.err);
    assert!(ran.out.contains(ID), "{}", ran.out);
}

#[test]
fn an_unknown_name_lists_the_known_boards_by_name_and_id() {
    let home = table_home();
    let ran = run(&home, &["board", "path", "--project", "nope"]);
    assert_eq!(ran.code, 1);
    assert!(ran.err.contains("Alpha (bk3m7q)"), "{}", ran.err);
}

#[test]
fn board_id_prints_id_name_and_aliases() {
    let home = table_home();
    let ran = run(&home, &["board", "id", "--project", "alpha"]);
    assert_eq!(ran.code, 0, "{}", ran.err);
    assert_eq!(ran.out.trim(), "bk3m7q Alpha alpha");
}

#[test]
fn board_id_without_a_table_entry_gives_the_folder() {
    let home = Home::new();
    home.write_board("legacy", &board_with("legacy", &[]));
    let ran = run(&home, &["board", "id", "--project", "legacy"]);
    assert_eq!(ran.code, 0, "{}", ran.err);
    assert_eq!(ran.out.trim(), "legacy legacy");
}

#[test]
fn board_name_writes_the_table_and_keeps_the_aliases() {
    let home = table_home();
    let ran = run(&home, &["board", "name", "Beta", "--project", "alpha"]);
    assert_eq!(ran.code, 0, "{}", ran.err);
    let after = table(&home);
    assert_eq!(after["boards"][ID]["name"], "Beta");
    assert_eq!(after["boards"][ID]["aliases"], json!(["alpha"]));
}

#[test]
fn a_write_is_refused_while_the_migration_runs() {
    let home = table_home();
    fs::write(home.coord_home().join("MIGRATING"), "").unwrap();
    let ran = run(&home, &["board", "name", "Beta", "--project", "alpha"]);
    assert_eq!(ran.code, 1);
    assert!(
        ran.err
            .contains("board migration in progress, try again in a minute"),
        "{}",
        ran.err
    );
    assert_eq!(table(&home)["boards"][ID]["name"], "Alpha");
}
