//! K1: the text limits.

// test-tag: critical

use argus_board::limits::{
    limit, optional_text, require_context, require_str, soft_context, text_list,
};
use serde_json::json;

#[test]
fn a_text_is_trimmed_and_then_refused_when_empty() {
    assert_eq!(require_str("title", "  hello \n").unwrap(), "hello");
    let err = require_str("title", " \t ").unwrap_err().to_string();
    assert_eq!(err, "title text is required.");
}

#[test]
fn a_text_over_the_limit_is_refused_with_its_length_and_the_limit() {
    let long = "x".repeat(81);
    let err = require_str("title", &long).unwrap_err().to_string();
    assert_eq!(err, "title is 81 chars, the limit is 80. Shorten it.");
    assert!(require_str("title", &"x".repeat(80)).is_ok());
}

#[test]
fn the_length_counts_utf16_units_so_an_emoji_counts_twice() {
    let forty = "\u{1F600}".repeat(20);
    assert!(require_str("name", &forty).is_ok());
    let over = format!("{forty}x");
    let err = require_str("name", &over).unwrap_err().to_string();
    assert_eq!(err, "name is 41 chars, the limit is 40. Shorten it.");
}

#[test]
fn optional_text_is_none_for_blank_and_checked_otherwise() {
    assert_eq!(optional_text("note", &json!("   ")).unwrap(), None);
    assert_eq!(optional_text("note", &json!(null)).unwrap(), None);
    assert_eq!(
        optional_text("note", &json!(" hi ")).unwrap().as_deref(),
        Some("hi")
    );
    assert!(optional_text("note", &json!("x".repeat(101))).is_err());
}

#[test]
fn a_list_checks_each_entry_and_holds_at_most_twenty() {
    let ok = text_list("ac", &json!(["a", " b "]), 20).unwrap();
    assert_eq!(ok, ["a", "b"]);
    let many = json!((0..21).map(|n| format!("c{n}")).collect::<Vec<_>>());
    let err = text_list("ac", &many, 20).unwrap_err().to_string();
    assert_eq!(err, "21 ac entries, the limit is 20.");
    let blank = text_list("ac", &json!(["a", ""]), 20)
        .unwrap_err()
        .to_string();
    assert_eq!(blank, "ac text is required.");
    assert_eq!(
        text_list("ac", &json!(null), 20).unwrap(),
        Vec::<String>::new()
    );
}

#[test]
fn a_missing_or_short_context_is_refused_with_the_teaching_text() {
    let missing = require_context(&json!(null)).unwrap_err().to_string();
    assert!(missing.starts_with("--context is required: in 20 to 600 chars"));
    assert!(missing.contains("Example: --context"));
    let short = require_context(&json!("too short"))
        .unwrap_err()
        .to_string();
    assert!(short.starts_with("--context is 9 chars, the minimum is 20. --context is required"));
    let fine = "x".repeat(20);
    assert_eq!(require_context(&json!(fine.clone())).unwrap(), fine);
    assert!(require_context(&json!("x".repeat(601))).is_err());
}

#[test]
fn soft_context_is_clipped_never_refused() {
    assert_eq!(soft_context(&json!("  ")), None);
    assert_eq!(soft_context(&json!(7)), None);
    let clipped = soft_context(&json!("y".repeat(700))).unwrap();
    assert_eq!(clipped.len(), 600);
}
