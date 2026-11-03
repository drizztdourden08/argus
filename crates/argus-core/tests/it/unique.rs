//! Distinct items keep the order of their first appearance.
//! K1: first appearance order.

// test-tag: critical

use argus_core::unique::unique;

#[test]
fn repeats_are_dropped_and_the_first_appearance_keeps_its_place() {
    assert_eq!(unique(["b", "a", "b", "c", "a"]), ["b", "a", "c"]);
}

#[test]
fn nothing_gives_nothing() {
    assert_eq!(unique(Vec::<u8>::new()), Vec::<u8>::new());
}
