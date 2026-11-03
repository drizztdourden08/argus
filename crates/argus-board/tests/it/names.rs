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
    assert_eq!(display_name("rotp", Some("RotP"), Some("Relic")), "RotP");
    assert_eq!(display_name("rotp", None, Some("Relic")), "Relic");
    assert_eq!(display_name("rotp", None, None), "rotp");
    assert_eq!(display_name("portfolio-two", None, None), "portfolio-two");
    assert_eq!(
        display_name("pixelarttool", Some("Bxel"), Some("Pixel art tool")),
        "Bxel"
    );
}

#[test]
fn the_overlords_project_is_overlord_by_default_and_old_names_never_count() {
    assert_eq!(display_name("portfolio", None, None), "Overlord");
    assert_eq!(
        display_name("portfolio", Some("Portfolio"), None),
        "Overlord"
    );
    assert_eq!(
        display_name(
            "portfolio",
            Some("Portfolio"),
            Some("Portfolio (owner to-dos)")
        ),
        "Overlord"
    );
    assert_eq!(
        display_name("portfolio", Some("portfolio"), Some("portfolio")),
        "Overlord"
    );
    assert_eq!(display_name("portfolio", Some("  "), Some("")), "Overlord");
    assert_eq!(
        display_name("portfolio", Some("Portfolio"), Some("Overlord")),
        "Overlord"
    );
    assert_eq!(
        display_name("portfolio", Some("Portfolio"), Some("Control room")),
        "Control room"
    );
    assert_eq!(
        display_name("portfolio", Some("Hub"), Some("Portfolio")),
        "Hub"
    );
    assert_eq!(
        display_name("portfolio", Some("Hub"), Some("Control room")),
        "Control room"
    );
}

#[test]
fn the_overlords_session_title_is_exactly_overlord() {
    assert_eq!(session_name("portfolio", Some("Portfolio")), "Overlord");
    assert_eq!(session_name("portfolio", None), "Overlord");
    assert_eq!(session_name("portfolio", Some("Argus")), "Overlord");
    assert_eq!(session_name("rotp", Some("RotP")), "RotP");
    assert_eq!(session_name("rotp", None), "rotp");
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
