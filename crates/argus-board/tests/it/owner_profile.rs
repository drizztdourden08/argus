//! The owner's profile: a `profile` post on the portfolio sets the avatar, name and initials the
//! Argus page wears, refused with a reason when a field is unfit, never handed down to a project.
//! K1, K3: the checks, the audit line and the export.

// test-tag: critical

use crate::common::{exported, home_with_alpha as home, portfolio_of};
use argus_board::portfolio::import_portfolio_entries;
use argus_board::store::read_board;
use argus_core::Coll;
use serde_json::{Value, json};
use std::sync::atomic::{AtomicUsize, Ordering};
use tempfile::TempDir;

const PNG: &str = "data:image/png;base64,iVBORw0KGgoA";
const SVG: &str =
    "data:image/svg+xml;base64,PHN2ZyB4bWxucz0iaHR0cDovL3d3dy53My5vcmcvMjAwMC9zdmciLz4=";
const SVG_SCRIPT: &str = "data:image/svg+xml;base64,PHN2ZyB4bWxucz0iaHR0cDovL3d3dy53My5vcmcvMjAwMC9zdmciPjxzY3JpcHQ+eDwvc2NyaXB0Pjwvc3ZnPg==";
static POSTS: AtomicUsize = AtomicUsize::new(0);

fn portfolio() -> Value {
    portfolio_of(json!({}))
}

/// Posts a profile; returns its status and the reason when refused.
fn profile(home: &TempDir, p: &mut Value, fields: Value) -> (String, Option<String>) {
    let id = format!("O-{}", POSTS.fetch_add(1, Ordering::SeqCst) + 1);
    let mut entry = json!({"id": id, "kind": "profile", "author": "human"});
    for (k, v) in fields.as_object().expect("fields") {
        entry[k] = v.clone();
    }
    let out = import_portfolio_entries(home.path(), p, vec![entry]).expect("imported");
    let reason = p["entries"][&id]["reason"].as_str().map(str::to_owned);
    (out.results[0].1.clone(), reason)
}

fn applied(home: &TempDir, p: &mut Value, fields: Value) {
    let (status, reason) = profile(home, p, fields);
    assert_eq!(status, "applied", "{reason:?}");
}

fn refused(home: &TempDir, p: &mut Value, fields: Value) -> String {
    let (status, reason) = profile(home, p, fields);
    assert_eq!(status, "refused");
    reason.expect("a reason")
}

fn owner_meta(p: &Value) -> Value {
    exported(p, Coll::Meta, "board")["owner"].clone()
}

#[test]
fn a_brand_initials_or_a_clean_image_is_taken_as_the_avatar() {
    let (h, mut p) = (home(), portfolio());
    for avatar in ["initials", "brand:argus", PNG, SVG] {
        applied(&h, &mut p, json!({"avatar": avatar}));
        assert_eq!(p["owner"]["avatar"], avatar);
    }
}

#[test]
fn an_unknown_brand_and_other_text_are_refused_and_nothing_changes() {
    let (h, mut p) = (home(), portfolio());
    assert!(refused(&h, &mut p, json!({"avatar": "brand:nope"})).contains("brand"));
    assert!(refused(&h, &mut p, json!({"avatar": "https://x.test/a.png"})).contains("avatar"));
    assert!(refused(&h, &mut p, json!({"avatar": 7})).contains("avatar"));
    assert!(p["owner"].is_null());
}

#[test]
fn an_svg_that_runs_code_and_a_broken_png_are_refused() {
    let (h, mut p) = (home(), portfolio());
    assert!(refused(&h, &mut p, json!({"avatar": SVG_SCRIPT})).contains("script"));
    let bad = "data:image/png;base64,@@@@";
    assert!(refused(&h, &mut p, json!({"avatar": bad})).contains("base64"));
    assert!(p["owner"].is_null());
}

#[test]
fn the_name_is_trimmed_and_held_to_one_to_forty_characters() {
    let (h, mut p) = (home(), portfolio());
    applied(&h, &mut p, json!({"name": "  Drizzt  "}));
    assert_eq!(p["owner"]["name"], "Drizzt");
    applied(&h, &mut p, json!({"name": "x".repeat(40)}));
    assert!(refused(&h, &mut p, json!({"name": "x".repeat(41)})).contains("name"));
    assert!(refused(&h, &mut p, json!({"name": "   "})).contains("name"));
    assert_eq!(p["owner"]["name"], "x".repeat(40));
}

#[test]
fn initials_are_two_upper_case_letters_or_digits() {
    let (h, mut p) = (home(), portfolio());
    applied(&h, &mut p, json!({"initials": "D7"}));
    for bad in ["d7", "D", "DRI", "D-", ""] {
        assert!(refused(&h, &mut p, json!({"initials": bad})).contains("initials"));
    }
    assert_eq!(p["owner"]["initials"], "D7");
}

#[test]
fn a_field_left_out_is_unchanged_and_null_restores_the_default() {
    let (h, mut p) = (home(), portfolio());
    let all = json!({"name": "Dri", "initials": "DR", "avatar": "brand:brock"});
    applied(&h, &mut p, all);
    applied(&h, &mut p, json!({"name": "Drizzt"}));
    assert_eq!(p["owner"]["initials"], "DR");
    applied(&h, &mut p, json!({"name": null, "avatar": null}));
    assert_eq!(p["owner"], json!({"initials": "DR"}));
    applied(&h, &mut p, json!({"initials": null}));
    assert!(p["owner"].is_null());
    assert!(owner_meta(&p).is_null());
}

#[test]
fn a_profile_is_not_handed_down_as_a_message() {
    let (h, mut p) = (home(), portfolio());
    applied(&h, &mut p, json!({"name": "Dri"}));
    let board = read_board(h.path(), "alpha").expect("read").expect("board");
    let entries = board["entries"].as_object();
    assert!(entries.is_none_or(|m| m.values().all(|e| e["via"] != "portfolio")));
    let inbox = board["inbox"].as_object();
    assert!(inbox.is_none_or(|m| m.values().all(|v| v.as_array().is_none_or(Vec::is_empty))));
}

#[test]
fn each_applied_profile_leaves_one_audit_line_and_a_refusal_none() {
    let (h, mut p) = (home(), portfolio());
    applied(&h, &mut p, json!({"name": "Dri", "initials": "DR"}));
    refused(&h, &mut p, json!({"initials": "x"}));
    let lines = p["log"]["portfolio"]["entries"].as_array().expect("log");
    assert_eq!(lines.len(), 1);
    let text = lines[0]["text"].as_str().expect("text");
    assert!(text.contains("profile") && text.contains("name") && text.contains("initials"));
}

#[test]
fn the_export_carries_the_owner_and_the_image_url_changes_with_the_image() {
    let (h, mut p) = (home(), portfolio());
    assert!(owner_meta(&p).is_null());
    applied(&h, &mut p, json!({"name": "Dri", "avatar": PNG}));
    let first = owner_meta(&p);
    assert_eq!(first["name"], "Dri");
    let url = first["avatar"].as_str().expect("url").to_owned();
    assert!(url.starts_with("/api/owner/avatar?v="));
    assert_eq!(url.len(), "/api/owner/avatar?v=".len() + 8);
    applied(&h, &mut p, json!({"avatar": SVG}));
    assert_ne!(owner_meta(&p)["avatar"], url);
    applied(&h, &mut p, json!({"avatar": "brand:argus"}));
    assert_eq!(owner_meta(&p)["avatar"], "brand:argus");
}
