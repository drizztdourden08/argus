//! K1: the normalising rules and the judgement of the differential harness.

// test-tag: critical

use std::collections::BTreeMap;

use crate::support::differ::verdict::Seen;
use crate::support::differ::{Normaliser, Verdict, judge};
use crate::support::latency::percentile;

fn norm(stable: &[&str]) -> Normaliser {
    Normaliser::new(&[], stable.iter().map(|s| (*s).to_owned()).collect())
}

fn seen(code: i32, stdout: &str, files: &[(&str, &str)]) -> Seen {
    Seen {
        code: Some(code),
        stdout: stdout.to_owned(),
        stderr: String::new(),
        files: files
            .iter()
            .map(|(k, v)| ((*k).to_owned(), (*v).to_owned()))
            .collect::<BTreeMap<_, _>>(),
    }
}

#[test]
fn times_stamps_ids_and_relative_times_are_normalised() {
    let text = "at 2026-10-26T15:49:13.648Z and 2026-10-26T15:49:13Z, \
                backup board.v1.20261017T193800Z.json, id m-1790000000123-ab12 and \
                m-1790000000123-I-3, seen 12s ago, 5m ago, 3h ago, 2d ago, \
                {\"pid\": 4242, \"at\": 1790000000123}";
    let want = "at <TS> and <TS>, backup board.v1.<STAMP>.json, id m-<MS>-<RND> and \
                m-<MS>-I-3, seen <AGO>, <AGO>, <AGO>, <AGO>, \
                {\"pid\": <PID>, \"at\": <EPOCH_MS>}";
    assert_eq!(norm(&[]).text(text), want);
}

#[test]
fn a_relative_time_absorbs_the_padding_of_its_column() {
    let n = norm(&[]);
    let pad_end = n.text(
        "a   9s ago   x
b  10s ago  x
",
    );
    assert_eq!(
        pad_end,
        "a <AGO> x
b <AGO> x
"
    );
    let pad_start = n.text(
        "   9s ago
  10s ago
 9s ago",
    );
    assert_eq!(
        pad_start,
        "<AGO>
<AGO>
<AGO>"
    );
    assert_eq!(
        n.text(
            "id  9s ago  
id 10s ago 
"
        ),
        n.text(
            "id 10s ago
id  9s ago
"
        )
    );
    assert_eq!(n.text("a9s ago"), "a9s ago");
}

#[test]
fn a_fixed_session_id_stays_visible_and_a_generated_one_is_replaced() {
    let fixed = "11111111-1111-4111-8111-111111111111";
    let other = "9f1c2d3e-4a5b-4c6d-8e7f-0123456789ab";
    let text = format!("{fixed} {other}");
    assert_eq!(norm(&[fixed]).text(&text), format!("{fixed} <UUID>"));
}

#[test]
fn plain_text_numbers_and_words_are_left_alone() {
    let text = "I-12 todo 2026 items, 1790000000123456 digits, 40 mamas, x1790000000123";
    assert_eq!(norm(&[]).text(text), text);
}

#[test]
fn a_home_path_is_normalised_in_every_spelling() {
    let root = std::path::Path::new("C:\\Temp\\.tmpAB12");
    let norm = Normaliser::new(&[root], Vec::new());
    let text = "C:\\Temp\\.tmpAB12\\coord C:/Temp/.tmpAB12/coord C:\\\\Temp\\\\.tmpAB12\\\\coord";
    assert_eq!(
        norm.text(text),
        "<HOME>\\coord <HOME>/coord <HOME>\\\\coord"
    );
}

#[test]
fn not_ported_is_its_own_verdict_and_never_a_mismatch() {
    let node = seen(0, "listing\n", &[("coord/alpha/board.json", "{}")]);
    let rust = seen(120, "", &[]);
    let verdict = judge(&node, &rust);
    assert_eq!(verdict, Verdict::NotPorted);
    assert_eq!(verdict.to_string(), "rust: not ported (120)");
}

#[test]
fn a_difference_in_code_stream_or_file_is_a_mismatch_that_names_it() {
    let node = seen(
        0,
        "a\nb\n",
        &[("coord/alpha/board.json", "1"), ("only-node", "x")],
    );
    let rust = seen(
        1,
        "a\nc\n",
        &[("coord/alpha/board.json", "2"), ("only-rust", "y")],
    );
    let Verdict::Mismatch(lines) = judge(&node, &rust) else {
        panic!("expected a mismatch")
    };
    let all = lines.join("\n");
    for part in [
        "exit code: node Some(0), rust Some(1)",
        "stdout differs (line 2",
        "board.json differs",
        "only-node: only in node",
        "only-rust: only in rust",
    ] {
        assert!(all.contains(part), "missing {part:?} in:\n{all}");
    }
}

#[test]
fn percentiles_are_nearest_rank() {
    let ms: Vec<_> = (1..=100).map(std::time::Duration::from_millis).collect();
    assert_eq!(percentile(&ms, 50).as_millis(), 50);
    assert_eq!(percentile(&ms, 95).as_millis(), 95);
    assert_eq!(percentile(&ms[..1], 95).as_millis(), 1);
}
