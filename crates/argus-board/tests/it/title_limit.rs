//! The board's item title limit: `titleMax` set by the `title-max` op, 80 when none is set, and
//! the range the op accepts. I-220.

// test-tag: critical

use crate::common::{alpha, applied, refused};
use argus_board::ops::{OpResult, Who, apply_op};
use serde_json::{Value, json};

fn as_owner(board: &mut Value, op: &Value) -> OpResult {
    apply_op(board, op, &Who::human(), None)
}

fn titled(board: &mut Value, len: usize) -> OpResult {
    let op = json!({"kind": "create", "target": {"c": "items"}, "set": {"title": "t".repeat(len)}});
    as_owner(board, &op)
}

#[test]
fn a_board_title_limit_of_40_refuses_41_chars_and_accepts_40() {
    let mut b = alpha();
    applied(&as_owner(
        &mut b,
        &json!({"kind": "title-max", "titleMax": 40}),
    ));
    assert_eq!(
        refused(&titled(&mut b, 41)),
        "title is 41 chars, the limit is 40. Shorten it."
    );
    applied(&titled(&mut b, 40));
}

#[test]
fn a_board_without_a_title_limit_keeps_80() {
    let mut b = alpha();
    applied(&titled(&mut b, 80));
    assert_eq!(
        refused(&titled(&mut b, 81)),
        "title is 81 chars, the limit is 80. Shorten it."
    );
}

#[test]
fn the_title_limit_must_be_from_20_to_200() {
    let mut b = alpha();
    for bad in [json!(19), json!(201), json!("40"), json!(40.5)] {
        let op = json!({"kind": "title-max", "titleMax": bad});
        refused(&as_owner(&mut b, &op));
    }
}
