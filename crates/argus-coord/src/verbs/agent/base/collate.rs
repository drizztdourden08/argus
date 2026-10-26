//! `a.localeCompare(b, undefined, { numeric: true })` for the ids, tags and names of a board:
//! white space and punctuation sort before digits and digits before letters, letters compare
//! without case first, a run of digits compares by value, and lower case wins a tie.

use std::cmp::Ordering;

/// The punctuation of the root collation, in its order.
const PUNCTUATION: &str = " _-,;:!?.'\"()[]{}@*/\\&#%`^+<=>|~$";

type Weight = (u8, usize, char);

fn weight(c: char) -> Weight {
    if let Some(at) = PUNCTUATION.find(c) {
        return (0, at, ' ');
    }
    if c.is_alphabetic() {
        return (2, 0, c.to_lowercase().next().unwrap_or(c));
    }
    (u8::from(!c.is_whitespace()), 0, c)
}

fn digits(chars: &[char]) -> usize {
    chars.iter().take_while(|c| c.is_ascii_digit()).count()
}

fn runs(a: &[char], b: &[char]) -> Ordering {
    let strip = |run: &[char]| {
        run.iter()
            .collect::<String>()
            .trim_start_matches('0')
            .to_owned()
    };
    let (x, y) = (strip(a), strip(b));
    x.len().cmp(&y.len()).then_with(|| x.cmp(&y))
}

fn first_pass(mut a: &[char], mut b: &[char]) -> Ordering {
    while !a.is_empty() && !b.is_empty() {
        let (la, lb) = (digits(a), digits(b));
        let (order, step_a, step_b) = if la > 0 && lb > 0 {
            (runs(&a[..la], &b[..lb]), la, lb)
        } else {
            (weight(a[0]).cmp(&weight(b[0])), 1, 1)
        };
        if order != Ordering::Equal {
            return order;
        }
        a = &a[step_a..];
        b = &b[step_b..];
    }
    a.len().cmp(&b.len())
}

fn case_pass(a: &[char], b: &[char]) -> Ordering {
    match a.iter().zip(b).find(|(x, y)| x != y) {
        Some((x, _)) if x.is_lowercase() => Ordering::Less,
        Some(_) => Ordering::Greater,
        None => Ordering::Equal,
    }
}

/// The order `a.localeCompare(b, undefined, { numeric: true })` gives.
#[must_use]
pub fn compare(a: &str, b: &str) -> Ordering {
    let (x, y): (Vec<char>, Vec<char>) = (a.chars().collect(), b.chars().collect());
    first_pass(&x, &y).then_with(|| case_pass(&x, &y))
}
