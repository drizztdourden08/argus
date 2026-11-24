//! K1: how the project of a call is found.

// test-tag: critical

use std::fs;

use crate::common::{Home, board_with, opts};
use argus_coord::who::{find_project, project_of_session};

fn wire(dir: &std::path::Path, key: &str) {
    fs::create_dir_all(dir).unwrap();
    fs::write(
        dir.join(".ai-config.json"),
        format!("{{\"projectKey\":\"{key}\"}}"),
    )
    .unwrap();
}

#[test]
fn project_wins_even_with_a_session_and_a_wired_folder() {
    let home = Home::new();
    let checkout = home.folder("checkout");
    wire(&checkout, "wired");
    let env = home.env_with("CLAUDE_CODE_SESSION_ID", "sid-1");
    let found = find_project(&env, &checkout, &opts(&["--project", "named"]), None);
    assert_eq!(found.unwrap(), "named");
}

#[test]
fn a_project_given_twice_is_a_list_and_does_not_count() {
    let home = Home::new();
    let checkout = home.folder("checkout");
    wire(&checkout, "wired");
    let opts = opts(&["--project", "a", "--project", "b"]);
    assert_eq!(
        find_project(&home.env(), &checkout, &opts, None).unwrap(),
        "wired"
    );
}

#[test]
fn coord_project_comes_next() {
    let home = Home::new();
    let checkout = home.folder("checkout");
    wire(&checkout, "wired");
    let env = home.env_with("COORD_PROJECT", "from-env");
    assert_eq!(
        find_project(&env, &checkout, &opts(&[]), None).unwrap(),
        "from-env"
    );
}

#[test]
fn the_board_holding_the_session_names_the_project_wherever_the_cwd_is() {
    let home = Home::new();
    let elsewhere = home.folder("elsewhere");
    wire(&elsewhere, "wired");
    home.write_board(
        "alpha",
        &board_with(
            "alpha",
            &[("lead", "lead", "active", Some("sid-lead"), None)],
        ),
    );
    home.write_board(
        "beta",
        &board_with("beta", &[("ui", "thread", "active", Some("sid-ui"), None)]),
    );
    let env = home.env_with("CLAUDE_CODE_SESSION_ID", "sid-ui");
    assert_eq!(
        find_project(&env, &elsewhere, &opts(&[]), None).unwrap(),
        "beta"
    );
    // a hook passes the session id of its input, which wins over the environment
    assert_eq!(
        find_project(&env, &elsewhere, &opts(&[]), Some("sid-lead")).unwrap(),
        "alpha"
    );
}

#[test]
fn an_ended_thread_does_not_own_its_session() {
    let home = Home::new();
    let checkout = home.folder("checkout");
    wire(&checkout, "wired");
    home.write_board(
        "alpha",
        &board_with(
            "alpha",
            &[
                ("gone", "thread", "done", Some("sid-x"), None),
                ("old", "thread", "archived", Some("sid-x"), None),
            ],
        ),
    );
    let env = home.env_with("CLAUDE_CODE_SESSION_ID", "sid-x");
    assert_eq!(
        find_project(&env, &checkout, &opts(&[]), None).unwrap(),
        "wired"
    );
}

#[test]
fn a_session_id_outside_the_threads_does_not_match() {
    let home = Home::new();
    let mut board = board_with(
        "alpha",
        &[("ui", "thread", "active", Some("sid-other"), None)],
    );
    board["note"] = serde_json::json!("sid-lonely is only mentioned here");
    home.write_board("alpha", &board);
    assert_eq!(project_of_session(&home.coord_home(), "sid-lonely"), None);
    assert_eq!(
        project_of_session(&home.coord_home(), "sid-other").as_deref(),
        Some("alpha")
    );
}

#[test]
fn the_first_board_in_directory_order_wins_and_unreadable_boards_are_skipped() {
    let home = Home::new();
    fs::create_dir_all(home.coord_home().join("aaa")).unwrap();
    fs::write(
        home.coord_home().join("aaa").join("board.json"),
        "{ not json",
    )
    .unwrap();
    fs::write(home.coord_home().join("loose-file"), "x").unwrap();
    home.write_board(
        "bbb",
        &board_with("bbb", &[("t", "thread", "active", Some("sid"), None)]),
    );
    assert_eq!(
        project_of_session(&home.coord_home(), "sid").as_deref(),
        Some("bbb")
    );
}

#[test]
fn the_nearest_ai_config_above_the_cwd_names_the_project() {
    let home = Home::new();
    let top = home.folder("top");
    wire(&top, "outer");
    let inner = top.join("packages").join("app");
    fs::create_dir_all(&inner).unwrap();
    assert_eq!(
        find_project(&home.env(), &inner, &opts(&[]), None).unwrap(),
        "outer"
    );
    wire(&top.join("packages"), "inner");
    assert_eq!(
        find_project(&home.env(), &inner, &opts(&[]), None).unwrap(),
        "inner"
    );
}

#[test]
fn a_wired_folder_without_a_key_falls_through_to_the_marker() {
    let home = Home::new();
    let checkout = home.folder("checkout");
    fs::write(checkout.join(".ai-config.json"), "{}").unwrap();
    fs::write(checkout.join(".coord-project"), "  marked \n").unwrap();
    assert_eq!(
        find_project(&home.env(), &checkout, &opts(&[]), None).unwrap(),
        "marked"
    );
}

#[test]
fn nothing_naming_a_project_is_refused_with_the_node_sentence() {
    let home = Home::new();
    let bare = home.folder("bare");
    let err = find_project(&home.env(), &bare, &opts(&[]), None).unwrap_err();
    assert_eq!(
        err.to_string(),
        "cannot tell which project this is: no .ai-config.json above the cwd. Pass --project <key>."
    );
}
