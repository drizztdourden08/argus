//! KEEP: K2.
//! The JSON a doc, a summary, a job and a removal answer carry: the key spellings and the absent keys of the Node dashboard.

// test-tag: standard

use crate::testkit::{Fixture, Harness, rule_for};
use serde_json::{Value, to_value};

fn started() -> Harness {
    let fx = Fixture::fake();
    let h = Harness::new(&fx.root, vec![rule_for(&fx)], |_| {});
    h.scan();
    h
}

#[test]
fn the_leftover_part_of_a_row_spells_the_size_as_node_does() {
    let doc = to_value(&started().docs()["plain"]).unwrap();
    assert!(doc["leftover"].get("sizeMB").is_some(), "{doc}");
    assert!(doc["leftover"].get("sizeMb").is_none(), "{doc}");
}

#[test]
fn a_row_with_no_job_has_no_job_key_and_the_other_empty_keys_stay_null() {
    let doc = to_value(&started().docs()["fresh"]).unwrap();
    assert!(doc.get("job").is_none(), "{doc}");
    assert_eq!(doc["leftover"], Value::Null);
    assert_eq!(doc["request"], Value::Null);
}

#[test]
fn the_summary_spells_the_leftover_sizes_as_node_does() {
    let meta = to_value(started().meta()).unwrap();
    assert!(meta.get("leftoverMB").is_some(), "{meta}");
    assert!(meta.get("leftoverMb").is_none(), "{meta}");
    assert!(meta["leftoverOld"].get("sizeMB").is_some(), "{meta}");
    assert!(meta["leftoverOld"].get("sizeMb").is_none(), "{meta}");
}
