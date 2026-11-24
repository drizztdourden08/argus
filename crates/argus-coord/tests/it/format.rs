//! K1: the text formats of the CLI.

// test-tag: critical

use argus_coord::cli::Capture;
use argus_coord::cli::out::{ago, as_json, clip, pad_end, say, table};
use serde_json::json;

fn rows(r: &[&[&str]]) -> Vec<Vec<String>> {
    r.iter()
        .map(|row| row.iter().map(|c| (*c).to_owned()).collect())
        .collect()
}

const NOW: i64 = 1_800_000_000_000;

fn at(secs_ago: i64) -> String {
    argus_core::clock::iso_from_ms(NOW - secs_ago * 1000)
}

#[test]
fn a_table_pads_every_column_to_its_widest_cell_and_trims_each_line() {
    let t = table(&rows(&[
        &["thread", "status", "note"],
        &["ui", "active", ""],
        &["lead", "idle", "waiting"],
    ]));
    assert_eq!(
        t,
        "thread  status  note\nui      active\nlead    idle    waiting"
    );
}

#[test]
fn an_empty_table_is_empty() {
    assert_eq!(table(&[]), "");
}

#[test]
fn widths_count_utf16_units_so_an_emoji_is_two_wide() {
    let t = table(&rows(&[&["a", "b"], &["\u{1F600}", "x"]]));
    assert_eq!(t, "a   b\n\u{1F600}  x");
    assert_eq!(pad_end("\u{1F600}", 4), "\u{1F600}  ");
    assert_eq!(pad_end("long", 2), "long");
}

#[test]
fn a_cell_beyond_the_first_row_is_not_padded() {
    let t = table(&rows(&[&["a"], &["bb", "extra"]]));
    assert_eq!(t, "a\nbb  extra");
}

#[test]
fn clip_cuts_at_a_utf16_unit_and_adds_an_ellipsis() {
    assert_eq!(clip("short", 10), "short");
    assert_eq!(clip("exactly10!", 10), "exactly10!");
    assert_eq!(clip("0123456789ab", 10), "012345678\u{2026}");
    // an emoji is two units: 8 units fit before the cut at 9
    assert_eq!(clip("abcdefgh\u{1F600}tail", 10), "abcdefgh\u{2026}");
}

#[test]
fn ago_prints_never_seconds_minutes_hours_days() {
    assert_eq!(ago(None, NOW), "never");
    assert_eq!(ago(Some(""), NOW), "never");
    assert_eq!(ago(Some(&at(0)), NOW), "0s ago");
    assert_eq!(ago(Some(&at(59)), NOW), "59s ago");
    assert_eq!(ago(Some(&at(60)), NOW), "1m ago");
    assert_eq!(ago(Some(&at(89)), NOW), "1m ago");
    assert_eq!(ago(Some(&at(90)), NOW), "2m ago");
    assert_eq!(ago(Some(&at(3600)), NOW), "1h ago");
    assert_eq!(ago(Some(&at(47 * 3600)), NOW), "47h ago");
    assert_eq!(ago(Some(&at(48 * 3600)), NOW), "2d ago");
    assert_eq!(ago(Some(&at(5 * 86_400)), NOW), "5d ago");
}

#[test]
fn ago_of_the_future_is_zero_and_of_garbage_is_nan_days() {
    assert_eq!(ago(Some(&at(-30)), NOW), "0s ago");
    assert_eq!(ago(Some("not a date"), NOW), "NaNd ago");
}

#[test]
fn say_joins_the_lines_and_ends_with_a_newline() {
    let mut out = Capture::default();
    say(&mut out, &["one", "two", "three"]);
    say(&mut out, &[""]);
    assert_eq!(out.text(), "one\ntwo\nthree\n\n");
}

#[test]
fn as_json_is_two_space_json_in_key_order() {
    let mut out = Capture::default();
    as_json(&mut out, &json!({"b": 1, "a": [true, null]}));
    assert_eq!(
        out.text(),
        "{\n  \"b\": 1,\n  \"a\": [\n    true,\n    null\n  ]\n}\n"
    );
}
