//! The manual tests of a report grouped by their build tag: the handed build first, the other
//! builds newest first, the tests with no build last.

use std::cmp::Ordering;

use argus_board::json::display;
use serde_json::{Value, json};

use crate::verbs::thread::docs::{field, or_null};

/// Punctuation in the order the root collation of ICU sorts it, before digits and letters.
const PUNCTUATION: &str = "_-,;:!?.'\"()[]{}@*/\\&#%`^+<=>|~$";

#[derive(PartialEq, Eq, PartialOrd, Ord)]
enum Class {
    Space,
    Punct(usize),
    Digit,
    Letter,
}

fn class_of(c: char) -> Class {
    if c.is_whitespace() {
        Class::Space
    } else if c.is_ascii_digit() {
        Class::Digit
    } else if let Some(at) = PUNCTUATION.find(c) {
        Class::Punct(at)
    } else if c.is_alphanumeric() {
        Class::Letter
    } else {
        Class::Punct(PUNCTUATION.len())
    }
}

fn digits(text: &str) -> (&str, &str) {
    let end = text.find(|c: char| !c.is_ascii_digit()).unwrap_or(text.len());
    text.split_at(end)
}

fn compare_numbers(a: &str, b: &str) -> Ordering {
    let (a, b) = (a.trim_start_matches('0'), b.trim_start_matches('0'));
    a.len().cmp(&b.len()).then_with(|| a.cmp(b))
}

fn compare_chars(a: char, b: char) -> Ordering {
    class_of(a)
        .cmp(&class_of(b))
        .then_with(|| a.to_lowercase().cmp(b.to_lowercase()))
}

/// `a.localeCompare(b, undefined, { numeric: true })` for the tags builds carry: digit runs
/// compare as numbers, letters ignore case first (lower case before upper case on a tie), and
/// punctuation sorts before digits, digits before letters.
#[must_use]
pub fn natural_cmp(a: &str, b: &str) -> Ordering {
    let (mut left, mut right) = (a, b);
    loop {
        match (left.chars().next(), right.chars().next()) {
            (None, None) => break,
            (None, Some(_)) => return Ordering::Less,
            (Some(_), None) => return Ordering::Greater,
            (Some(x), Some(y)) if x.is_ascii_digit() && y.is_ascii_digit() => {
                let ((n, rest_left), (m, rest_right)) = (digits(left), digits(right));
                let order = compare_numbers(n, m);
                if order != Ordering::Equal {
                    return order;
                }
                (left, right) = (rest_left, rest_right);
            }
            (Some(x), Some(y)) => {
                let order = compare_chars(x, y);
                if order != Ordering::Equal {
                    return order;
                }
                (left, right) = (&left[x.len_utf8()..], &right[y.len_utf8()..]);
            }
        }
    }
    case_tie(a, b)
}

fn case_tie(a: &str, b: &str) -> Ordering {
    for (x, y) in a.chars().zip(b.chars()) {
        if x != y {
            return if x.is_lowercase() { Ordering::Less } else { Ordering::Greater };
        }
    }
    Ordering::Equal
}

fn rank(build: &Value, handed: &Value) -> u8 {
    if build.is_null() {
        2
    } else if build == handed {
        0
    } else {
        1
    }
}

/// The groups of manual tests by build, as the report lists them.
#[must_use]
pub fn by_build(tests: &[&Value], handed: &Value) -> Vec<Value> {
    let mut groups: Vec<(Value, Vec<Value>)> = Vec::new();
    for t in tests {
        let build = or_null(field(t, "build"));
        let id = field(t, "id").unwrap_or(Value::Null);
        match groups.iter_mut().find(|(b, _)| *b == build) {
            Some((_, ids)) => ids.push(id),
            None => groups.push((build, vec![id])),
        }
    }
    groups.sort_by(|(a, _), (b, _)| {
        rank(a, handed)
            .cmp(&rank(b, handed))
            .then_with(|| natural_cmp(&display(b), &display(a)))
    });
    groups
        .into_iter()
        .map(|(build, ids)| {
            let is_handed = !build.is_null() && build == *handed;
            json!({"build": build, "handed": is_handed, "tests": ids})
        })
        .collect()
}
