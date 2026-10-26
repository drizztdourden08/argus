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
    let end = text
        .find(|c: char| !c.is_ascii_digit())
        .unwrap_or(text.len());
    text.split_at(end)
}

fn compare_numbers(one: &str, other: &str) -> Ordering {
    let (one, other) = (one.trim_start_matches('0'), other.trim_start_matches('0'));
    one.len().cmp(&other.len()).then_with(|| one.cmp(other))
}

fn compare_chars(first: char, second: char) -> Ordering {
    class_of(first)
        .cmp(&class_of(second))
        .then_with(|| first.to_lowercase().cmp(second.to_lowercase()))
}

/// Compares the heads of two texts: two digit runs as numbers, else one character each. Equal
/// heads leave the rest of both texts to compare.
fn step<'t>(left: &'t str, right: &'t str) -> Result<(&'t str, &'t str), Ordering> {
    let (first, second) = (left.chars().next(), right.chars().next());
    let (Some(first), Some(second)) = (first, second) else {
        return Err(first.is_some().cmp(&second.is_some()));
    };
    if first.is_ascii_digit() && second.is_ascii_digit() {
        let ((one, rest_one), (other, rest_other)) = (digits(left), digits(right));
        return match compare_numbers(one, other) {
            Ordering::Equal => Ok((rest_one, rest_other)),
            order => Err(order),
        };
    }
    match compare_chars(first, second) {
        Ordering::Equal => Ok((&left[first.len_utf8()..], &right[second.len_utf8()..])),
        order => Err(order),
    }
}

fn case_tie(left: &str, right: &str) -> Ordering {
    for (ours, theirs) in left.chars().zip(right.chars()) {
        if ours != theirs {
            return if ours.is_lowercase() {
                Ordering::Less
            } else {
                Ordering::Greater
            };
        }
    }
    Ordering::Equal
}

/// `a.localeCompare(b, undefined, { numeric: true })` for the tags builds carry: digit runs
/// compare as numbers, letters ignore case first (lower case before upper case on a tie), and
/// punctuation sorts before digits, digits before letters.
#[must_use]
pub fn natural_cmp(left: &str, right: &str) -> Ordering {
    let (mut rest_left, mut rest_right) = (left, right);
    loop {
        if rest_left.is_empty() && rest_right.is_empty() {
            return case_tie(left, right);
        }
        match step(rest_left, rest_right) {
            Ok(rest) => (rest_left, rest_right) = rest,
            Err(order) => return order,
        }
    }
}

fn rank(build: &Value, handed: &Value) -> u8 {
    if build.is_null() {
        2
    } else {
        u8::from(build != handed)
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
    groups.sort_by(|(first, _), (second, _)| {
        rank(first, handed)
            .cmp(&rank(second, handed))
            .then_with(|| natural_cmp(&display(second), &display(first)))
    });
    groups
        .into_iter()
        .map(|(build, ids)| {
            let is_handed = !build.is_null() && build == *handed;
            json!({"build": build, "handed": is_handed, "tests": ids})
        })
        .collect()
}
