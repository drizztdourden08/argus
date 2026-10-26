//! The JSON a doc, a summary, a job and a removal answer carry: the key spellings and the absent keys of the Node dashboard.

use crate::jobs::JobDoc;
use crate::request::RemoveOutcome;
use crate::testkit::{Fixture, Harness, rule_for};
use serde_json::{Value, to_value};

fn started() -> Harness {
    let fx = Fixture::new();
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

#[test]
fn a_job_and_a_removal_answer_spell_the_size_as_node_does() {
    let job = JobDoc::queued("L-1", "row", "X:/a", "lead", "t".into());
    assert_eq!(to_value(&job).unwrap()["sizeMB"], Value::Null);
    let answer = to_value(RemoveOutcome::default()).unwrap();
    assert_eq!(answer["sizeMB"], Value::Null);
    assert!(answer.get("sizeMb").is_none(), "{answer}");
}
