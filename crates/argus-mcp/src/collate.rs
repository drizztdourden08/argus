//! `String.prototype.localeCompare` for the ids and names on a board: punctuation sorts before
//! digits and digits before letters, letters compare without case first and lower case wins a
//! tie. With `numeric`, runs of digits compare by value.

use std::cmp::Ordering;

fn class(c: char) -> u8 {
    if c.is_alphabetic() {
        2
    } else {
        u8::from(c.is_numeric())
    }
}

fn primary(a: char, b: char) -> Ordering {
    class(a).cmp(&class(b)).then_with(|| {
        let (x, y) = (a.to_lowercase().next(), b.to_lowercase().next());
        x.cmp(&y)
    })
}

fn digit_run(chars: &[char]) -> usize {
    chars.iter().take_while(|c| c.is_ascii_digit()).count()
}

fn compare_runs(a: &[char], b: &[char]) -> Ordering {
    let strip = |run: &[char]| -> String {
        let s: String = run.iter().collect();
        s.trim_start_matches('0').to_owned()
    };
    let (x, y) = (strip(a), strip(b));
    x.len().cmp(&y.len()).then_with(|| x.cmp(&y))
}

fn step(a: &[char], b: &[char], numeric: bool) -> (Ordering, usize, usize) {
    if numeric && a[0].is_ascii_digit() && b[0].is_ascii_digit() {
        let (la, lb) = (digit_run(a), digit_run(b));
        return (compare_runs(&a[..la], &b[..lb]), la, lb);
    }
    (primary(a[0], b[0]), 1, 1)
}

fn first_pass(mut a: &[char], mut b: &[char], numeric: bool) -> Ordering {
    while !a.is_empty() && !b.is_empty() {
        let (order, la, lb) = step(a, b, numeric);
        if order != Ordering::Equal {
            return order;
        }
        a = &a[la..];
        b = &b[lb..];
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

/// The order `a.localeCompare(b)` gives, `numeric` standing for `{ numeric: true }`.
#[must_use]
pub fn compare(a: &str, b: &str, numeric: bool) -> Ordering {
    let (x, y): (Vec<char>, Vec<char>) = (a.chars().collect(), b.chars().collect());
    first_pass(&x, &y, numeric).then_with(|| case_pass(&x, &y))
}
