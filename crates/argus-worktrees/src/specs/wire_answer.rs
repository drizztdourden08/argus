//! KEEP: K2 (the key spellings of a job and a removal answer, as the Node dashboard has them).

// test-tag: critical

use crate::jobs::JobDoc;
use crate::request::RemoveOutcome;
use serde_json::{Value, to_value};

#[test]
fn a_job_and_a_removal_answer_spell_the_size_as_node_does() {
    let job = JobDoc::queued("L-1", "row", "X:/a", "lead", "t".into());
    assert_eq!(to_value(&job).unwrap()["sizeMB"], Value::Null);
    let answer = to_value(RemoveOutcome::default()).unwrap();
    assert_eq!(answer["sizeMB"], Value::Null);
    assert!(answer.get("sizeMb").is_none(), "{answer}");
}
