//! The names the owner sees: the main coordinator thread is the Overlord, and the Overlord's own
//! project (key `portfolio`) is shown as the Overlord. Only the displayed names change; the ids
//! stay.
//! K1: the display names.

// test-tag: critical

use argus_board::names::{display_name, session_name};
use argus_board::portfolio::summarize;
use serde_json::json;

#[test]
fn other_projects_are_their_board_name_else_the_registry_name_else_the_key() {
    let home = tempfile::tempdir().expect("temp dir");
    assert_eq!(display_name(home.path(), "rotp", Some("RotP"), Some("Relic")), "RotP");
    assert_eq!(display_name(home.path(), "rotp", None, Some("Relic")), "Relic");
    assert_eq!(display_name(home.path(), "rotp", None, None), "rotp");
    assert_eq!(display_name(home.path(), "portfolio-two", None, None), "portfolio-two");
    assert_eq!(
        display_name(home.path(), "pixelarttool", Some("Bxel"), Some("Pixel art tool")),
        "Bxel"
    );
}

#[test]
fn the_overlords_project_is_overlord_by_default_and_old_names_never_count() {
    let home = tempfile::tempdir().expect("temp dir");
    assert_eq!(display_name(home.path(), "portfolio", None, None), "Overlord");
    assert_eq!(
        display_name(home.path(), "portfolio", Some("Portfolio"), None),
        "Overlord"
    );
    assert_eq!(
        display_name(
            home.path(),
            "portfolio",
            Some("Portfolio"),
            Some("Portfolio (owner to-dos)")
        ),
        "Overlord"
    );
    assert_eq!(
        display_name(home.path(), "portfolio", Some("portfolio"), Some("portfolio")),
        "Overlord"
    );
    assert_eq!(display_name(home.path(), "portfolio", Some("  "), Some("")), "Overlord");
    assert_eq!(
        display_name(home.path(), "portfolio", Some("Portfolio"), Some("Overlord")),
        "Overlord"
    );
    assert_eq!(
        display_name(home.path(), "portfolio", Some("Portfolio"), Some("Control room")),
        "Control room"
    );
    assert_eq!(
        display_name(home.path(), "portfolio", Some("Hub"), Some("Portfolio")),
        "Hub"
    );
    assert_eq!(
        display_name(home.path(), "portfolio", Some("Hub"), Some("Control room")),
        "Control room"
    );
}

#[test]
fn the_overlords_session_title_is_exactly_overlord() {
    let home = tempfile::tempdir().expect("temp dir");
    assert_eq!(session_name(home.path(), "portfolio", Some("Portfolio")), "Overlord");
    assert_eq!(session_name(home.path(), "portfolio", None), "Overlord");
    assert_eq!(session_name(home.path(), "portfolio", Some("Argus")), "Overlord");
    assert_eq!(session_name(home.path(), "rotp", Some("RotP")), "RotP");
    assert_eq!(session_name(home.path(), "rotp", None), "rotp");
}

#[test]
fn the_card_of_key_portfolio_is_overlord_also_without_a_board_and_other_cards_keep_their_names() {
    let home = tempfile::tempdir().expect("temp dir");
    let reg = json!({"projects": {
        "portfolio": {"key": "portfolio", "name": "Portfolio (owner to-dos)", "root": "X:\\ai-config"},
        "rotp": {"key": "rotp", "name": "RotP", "root": null},
    }});
    let unstarted = summarize(home.path(), "portfolio", &reg, None);
    assert_eq!(unstarted["name"], "Overlord");
    assert_eq!(unstarted["url"], "/p/portfolio/");
    assert_eq!(summarize(home.path(), "rotp", &reg, None)["name"], "RotP");
    let board = json!({
        "project": "portfolio", "name": "Portfolio", "threads": {}, "entries": {}, "tests": {},
        "items": {}, "actions": {}, "log": {}, "agents": {}, "updatedAt": null,
    });
    let card = summarize(home.path(), "portfolio", &reg, Some(&board));
    assert_eq!(card["key"], "portfolio");
    assert_eq!(card["name"], "Overlord");
    let named_reg =
        json!({"projects": {"portfolio": {"key": "portfolio", "name": "Control room"}}});
    let named = summarize(home.path(), "portfolio", &named_reg, Some(&board));
    assert_eq!(named["name"], "Control room");
}

const TABLE: &str = r#"{ "version": 1, "state": "active", "boards": {
  "b4k7qz2": { "name": "ShardTide", "aliases": ["geo-survivors"], "role": null },
  "b9m2xv3": { "name": "Control room", "aliases": ["portfolio"], "role": "overlord" },
  "b2c5hd8": { "name": "Argus app", "aliases": ["argus"], "role": "argus" }
} }"#;

#[test]
fn roles_come_from_the_legacy_folders_without_a_table_and_from_the_table_with_one() {
    use argus_board::roles::{id_of, is_argus, is_overlord};
    use argus_core::boards::Role;
    let home = tempfile::tempdir().expect("temp dir");
    assert_eq!(id_of(home.path(), Role::Overlord), None);
    for key in ["portfolio", "argus", "rotp"] {
        std::fs::create_dir_all(home.path().join(key)).expect("folder");
        std::fs::write(home.path().join(key).join("board.json"), "{}").expect("board");
    }
    assert_eq!(
        id_of(home.path(), Role::Overlord).as_deref(),
        Some("portfolio")
    );
    assert_eq!(id_of(home.path(), Role::Argus).as_deref(), Some("argus"));
    assert!(is_overlord(home.path(), "portfolio") && !is_overlord(home.path(), "rotp"));
    std::fs::write(home.path().join("boards.json"), TABLE).expect("table");
    assert_eq!(
        id_of(home.path(), Role::Overlord).as_deref(),
        Some("b9m2xv3")
    );
    assert_eq!(id_of(home.path(), Role::Argus).as_deref(), Some("b2c5hd8"));
    assert!(is_overlord(home.path(), "b9m2xv3") && is_overlord(home.path(), "portfolio"));
    assert!(is_argus(home.path(), "b2c5hd8") && !is_argus(home.path(), "b4k7qz2"));
    assert!(!is_overlord(home.path(), "b4k7qz2"));
}

#[test]
fn a_renamed_display_name_is_read_from_the_table_through_the_id_and_the_alias() {
    let home = tempfile::tempdir().expect("temp dir");
    std::fs::write(home.path().join("boards.json"), TABLE).expect("table");
    let shown = |id: &str| display_name(home.path(), id, Some("old board name"), Some("old"));
    assert_eq!(shown("b4k7qz2"), "ShardTide");
    assert_eq!(shown("geo-survivors"), "ShardTide");
    assert_eq!(shown("b9m2xv3"), "Control room");
    assert_eq!(session_name(home.path(), "b9m2xv3", None), "Overlord");
    assert_eq!(session_name(home.path(), "b4k7qz2", Some("x")), "ShardTide");
}
