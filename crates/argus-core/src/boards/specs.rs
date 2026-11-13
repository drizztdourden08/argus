use std::fs;
use std::path::Path;

use super::*;
use crate::ids::is_reserved;

fn entry(name: &str, aliases: &[&str], role: Option<Role>) -> BoardEntry {
    BoardEntry {
        name: name.to_owned(),
        aliases: aliases.iter().map(|a| (*a).to_owned()).collect(),
        role,
        created_at: "2026-11-12T00:00:00.000Z".to_owned(),
        renamed_at: None,
    }
}

fn table(home: &Path) {
    let mut boards = Boards::default();
    boards
        .boards
        .insert("b4k7qz2".into(), entry("ShardTide", &["geo"], None));
    boards.boards.insert(
        "b2m9xz3".into(),
        entry("Overlord", &["portfolio"], Some(Role::Overlord)),
    );
    save(home, &boards).unwrap();
}

// test-tag: critical
#[test]
fn save_and_load_round_trip() {
    let home = tempfile::tempdir().unwrap();
    assert_eq!(load(home.path()).unwrap().boards.len(), 0);
    table(home.path());
    let back = load(home.path()).unwrap();
    assert_eq!(back.boards.len(), 2);
    assert_eq!(back.boards["b4k7qz2"].aliases, vec!["geo".to_owned()]);
    assert_eq!(back.boards["b2m9xz3"].role, Some(Role::Overlord));
    assert_eq!(fs::read_dir(home.path()).unwrap().count(), 1);
}

// test-tag: critical
#[test]
fn resolve_order_is_id_then_alias_then_folder() {
    let home = tempfile::tempdir().unwrap();
    table(home.path());
    fs::create_dir_all(home.path().join("legacy")).unwrap();
    fs::write(home.path().join("legacy/board.json"), "{}").unwrap();
    assert_eq!(resolve(home.path(), "b4k7qz2").as_deref(), Some("b4k7qz2"));
    assert_eq!(resolve(home.path(), "geo").as_deref(), Some("b4k7qz2"));
    assert_eq!(
        resolve(home.path(), "portfolio").as_deref(),
        Some("b2m9xz3")
    );
    assert_eq!(resolve(home.path(), "legacy").as_deref(), Some("legacy"));
    assert_eq!(resolve(home.path(), "nope"), None);
}

// test-tag: critical
#[test]
fn legacy_home_without_table_resolves_folders_never_portfolio() {
    let home = tempfile::tempdir().unwrap();
    for key in ["rotp", "_portfolio"] {
        fs::create_dir_all(home.path().join(key)).unwrap();
        fs::write(home.path().join(key).join("board.json"), "{}").unwrap();
    }
    assert_eq!(resolve(home.path(), "rotp").as_deref(), Some("rotp"));
    assert_eq!(resolve(home.path(), "_portfolio"), None);
}

// test-tag: critical
#[test]
fn mint_makes_a_unique_formatted_id_and_records_the_alias() {
    let home = tempfile::tempdir().unwrap();
    let mut seen = Vec::new();
    for n in 0..50 {
        let id = mint(home.path(), &format!("key{n}")).unwrap();
        assert_eq!(id.len(), 7);
        assert!(id.starts_with('b'));
        assert!(id.bytes().any(|b| b.is_ascii_digit()));
        assert!(
            id[1..]
                .bytes()
                .all(|b| b.is_ascii_digit() || (b.is_ascii_lowercase() && !b"ilou".contains(&b)))
        );
        assert!(!is_reserved(&id));
        assert!(!seen.contains(&id));
        seen.push(id);
    }
    let boards = load(home.path()).unwrap();
    assert_eq!(boards.boards[&seen[3]].aliases, vec!["key3".to_owned()]);
    assert_eq!(resolve(home.path(), "key3"), Some(seen[3].clone()));
}

// test-tag: critical
#[test]
fn mint_refuses_bad_and_taken_keys() {
    let home = tempfile::tempdir().unwrap();
    table(home.path());
    for key in ["_portfolio", "", "Bad Key", "geo", "b4k7qz2"] {
        assert!(mint(home.path(), key).is_err(), "{key}");
    }
    assert_eq!(load(home.path()).unwrap().boards.len(), 2);
    assert!(is_reserved("overlord") && is_reserved("localpkg"));
}

// test-tag: critical
#[test]
fn rename_checks_bounds_and_keeps_aliases() {
    let home = tempfile::tempdir().unwrap();
    table(home.path());
    rename(home.path(), "b4k7qz2", "  Shard  ").unwrap();
    let boards = load(home.path()).unwrap();
    assert_eq!(boards.boards["b4k7qz2"].name, "Shard");
    assert_eq!(boards.boards["b4k7qz2"].aliases, vec!["geo".to_owned()]);
    assert!(boards.boards["b4k7qz2"].renamed_at.is_some());
    assert_eq!(
        display_name(home.path(), "b4k7qz2").as_deref(),
        Some("Shard")
    );
    rename(home.path(), "b4k7qz2", &"x".repeat(30)).unwrap();
    assert!(rename(home.path(), "b4k7qz2", &"x".repeat(31)).is_err());
    assert!(rename(home.path(), "b4k7qz2", "   ").is_err());
    assert!(rename(home.path(), "bzzzzzz", "Name").is_err());
}

// test-tag: critical
#[test]
fn roles_are_found_both_ways() {
    let home = tempfile::tempdir().unwrap();
    table(home.path());
    assert_eq!(role(home.path(), "b2m9xz3"), Some(Role::Overlord));
    assert_eq!(role(home.path(), "b4k7qz2"), None);
    assert_eq!(
        by_role(home.path(), Role::Overlord).as_deref(),
        Some("b2m9xz3")
    );
    assert_eq!(by_role(home.path(), Role::Argus), None);
}
