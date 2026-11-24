//! K4: who the caller is and who may act as whom.

// test-tag: critical

use std::path::Path;

use crate::common::{Home, board_with, opts};
use argus_board::ops::Role;
use argus_coord::cli::Env;
use argus_coord::who::{caller, me, resolve_thread};
use serde_json::{Value, json};

fn with_session(home: &Home, sid: &str) -> Env {
    home.env_with("CLAUDE_CODE_SESSION_ID", sid)
}

fn lead_and_ui(home: &Home) -> (Value, std::path::PathBuf, std::path::PathBuf) {
    let lead_root = home.folder("lead");
    let ui_root = home.folder("ui");
    let board = board_with(
        "alpha",
        &[
            ("lead", "lead", "active", Some("sid-lead"), Some(&lead_root)),
            ("ui", "thread", "active", Some("sid-ui"), Some(&ui_root)),
        ],
    );
    (board, lead_root, ui_root)
}

#[test]
fn the_thread_holding_the_session_id_is_the_caller_wherever_the_cwd_is() {
    let home = Home::new();
    let (board, ..) = lead_and_ui(&home);
    let who = caller(
        &board,
        home.path(),
        &with_session(&home, "sid-ui"),
        &opts(&[]),
    )
    .unwrap();
    assert_eq!(who.thread, "ui");
    assert_eq!(who.role, Role::Thread);
    assert_eq!(who.actor, "ui");
    assert_eq!(who.real.as_deref(), Some("ui"));
    let lead = caller(
        &board,
        home.path(),
        &with_session(&home, "sid-lead"),
        &opts(&[]),
    )
    .unwrap();
    assert_eq!((lead.thread.as_str(), lead.role), ("lead", Role::Lead));
}

#[test]
fn a_thread_nobody_claimed_is_found_by_the_checkout_it_registered_from() {
    let home = Home::new();
    let root = home.folder("work");
    let board = board_with(
        "alpha",
        &[("worker", "thread", "active", None, Some(&root))],
    );
    let inside = root.join("src").join("deep");
    let found = resolve_thread(&board, &inside, &opts(&[]), Some("sid-new")).unwrap();
    assert_eq!(found, "worker");
}

#[test]
fn a_checkout_another_session_claimed_is_not_ours() {
    let home = Home::new();
    let (board, lead_root, _) = lead_and_ui(&home);
    let err = resolve_thread(&board, &lead_root, &opts(&[]), Some("sid-stranger")).unwrap_err();
    assert_eq!(
        err.to_string(),
        "no thread registered for this session. Run `coord thread register <name>` first, or pass --as <thread>."
    );
}

#[test]
fn the_deepest_root_wins_and_equal_roots_are_refused() {
    let home = Home::new();
    let outer = home.folder("repo");
    let inner = outer.join("packages").join("app");
    std::fs::create_dir_all(&inner).unwrap();
    let board = board_with(
        "alpha",
        &[
            ("wide", "thread", "active", None, Some(&outer)),
            ("narrow", "thread", "active", None, Some(&inner)),
        ],
    );
    assert_eq!(
        resolve_thread(&board, &inner, &opts(&[]), None).unwrap(),
        "narrow"
    );
    assert_eq!(
        resolve_thread(&board, &outer, &opts(&[]), None).unwrap(),
        "wide"
    );

    let tied = board_with(
        "alpha",
        &[
            ("one", "thread", "active", None, Some(&outer)),
            ("two", "thread", "active", None, Some(&outer)),
        ],
    );
    let err = resolve_thread(&tied, &outer, &opts(&[]), None).unwrap_err();
    assert_eq!(
        err.to_string(),
        "several threads share this root (one, two); pass --as <thread>."
    );
}

#[test]
fn a_sibling_folder_with_the_same_prefix_is_not_inside_the_root() {
    let home = Home::new();
    let root = home.folder("app");
    let sibling = home.folder("app-two");
    let board = board_with("alpha", &[("t", "thread", "active", None, Some(&root))]);
    assert!(resolve_thread(&board, &sibling, &opts(&[]), None).is_err());
}

#[test]
fn ended_threads_own_neither_session_nor_checkout() {
    let home = Home::new();
    let root = home.folder("work");
    let board = board_with(
        "alpha",
        &[
            ("done-one", "thread", "done", Some("sid-a"), Some(&root)),
            ("old-one", "thread", "archived", None, Some(&root)),
        ],
    );
    assert!(resolve_thread(&board, &root, &opts(&[]), Some("sid-a")).is_err());
}

#[test]
fn as_names_a_thread_and_an_unknown_name_is_refused() {
    let home = Home::new();
    let (board, ..) = lead_and_ui(&home);
    assert_eq!(
        resolve_thread(&board, home.path(), &opts(&["--as", "ui"]), None).unwrap(),
        "ui"
    );
    let err = caller(&board, home.path(), &home.env(), &opts(&["--as", "nobody"])).unwrap_err();
    assert_eq!(err.to_string(), "no thread named \"nobody\" on this board.");
}

#[test]
fn a_call_with_no_session_may_act_as_any_thread_with_that_threads_authority() {
    let home = Home::new();
    let (board, ..) = lead_and_ui(&home);
    let who = caller(&board, home.path(), &home.env(), &opts(&["--as", "lead"])).unwrap();
    assert_eq!(
        (who.thread.as_str(), who.role, who.actor.as_str(), who.real),
        ("lead", Role::Lead, "lead", None)
    );
}

#[test]
fn a_thread_never_acts_as_another_thread() {
    let home = Home::new();
    let (board, ..) = lead_and_ui(&home);
    let env = with_session(&home, "sid-ui");
    let err = caller(&board, home.path(), &env, &opts(&["--as", "lead"])).unwrap_err();
    assert_eq!(
        err.to_string(),
        "--as lead refused: this session is thread ui, and a thread never acts as another thread."
    );
    let itself = caller(&board, home.path(), &env, &opts(&["--as", "ui"])).unwrap();
    assert_eq!(itself.thread, "ui");
}

#[test]
fn the_lead_acting_as_a_thread_keeps_its_authority_and_is_the_actor() {
    let home = Home::new();
    let (board, ..) = lead_and_ui(&home);
    let env = with_session(&home, "sid-lead");
    let who = caller(&board, home.path(), &env, &opts(&["--as", "ui"])).unwrap();
    assert_eq!(who.thread, "ui");
    assert_eq!(who.role, Role::Lead);
    assert_eq!(who.actor, "lead");
    assert_eq!(who.real.as_deref(), Some("lead"));
}

#[test]
fn me_refreshes_the_heartbeat_of_the_real_caller_never_the_as_thread() {
    let home = Home::new();
    let (mut board, ..) = lead_and_ui(&home);
    let env = with_session(&home, "sid-lead");
    let acted = me(&mut board, home.path(), &env, &opts(&["--as", "ui"])).unwrap();
    assert_eq!(acted, "ui");
    assert!(board["threads"]["lead"]["lastSeen"].is_string());
    assert!(board["threads"]["ui"].get("lastSeen").is_none());
    assert_eq!(board["dirty"], json!({"threads/lead": true}));
    // a revision is not bumped by a heartbeat
    assert!(board["threads"]["lead"].get("rev").is_none());
}

#[test]
fn me_for_a_call_without_a_session_changes_nothing() {
    let home = Home::new();
    let (mut board, ..) = lead_and_ui(&home);
    let before = board.clone();
    me(
        &mut board,
        Path::new("."),
        &home.env(),
        &opts(&["--as", "ui"]),
    )
    .unwrap();
    assert_eq!(board, before);
}
