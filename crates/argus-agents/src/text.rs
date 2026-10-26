//! Small text rules: one-line clips, and how well two descriptions of one dispatch fit each other.

use std::collections::HashSet;

const WORD_SKIP: [&str; 27] = [
    "the", "and", "for", "with", "from", "into", "that", "this", "then", "all", "use", "new", "of",
    "to", "in", "on", "a", "an", "is", "it", "or", "as", "at", "by", "be", "no", "vs",
];

/// The text on one line, trimmed and cut to `max` characters.
#[must_use]
pub fn clip(value: &str, max: usize) -> String {
    let line = value.split_whitespace().collect::<Vec<_>>().join(" ");
    line.chars().take(max).collect()
}

/// The text trimmed and cut to `max` characters, line breaks kept.
#[must_use]
pub fn cut(value: &str, max: usize) -> String {
    value.trim().chars().take(max).collect()
}

/// The lowercase letter and digit words of a text, without the filler words.
#[must_use]
pub fn words_of(text: &str) -> HashSet<String> {
    let lower = text.to_lowercase();
    lower
        .split(|c: char| !c.is_ascii_alphanumeric())
        .filter(|w| w.len() > 1 && !WORD_SKIP.contains(w))
        .map(str::to_owned)
        .collect()
}

/// How many words two texts share.
#[must_use]
pub fn overlap(a: &str, b: &str) -> usize {
    let wa = words_of(a);
    words_of(b).iter().filter(|w| wa.contains(*w)).count()
}

/// The candidate that fits `text` best, the first one winning a tie, or `None`. It needs at least
/// `min` shared words. A text with no words of its own matches only when there is one candidate.
pub fn best_fit<'a, T>(
    text: &str,
    candidates: &'a [T],
    text_of: impl Fn(&T) -> String,
    min: usize,
) -> Option<&'a T> {
    if words_of(text).is_empty() {
        return (candidates.len() == 1).then(|| &candidates[0]);
    }
    let mut best = None;
    let mut top = min.saturating_sub(1);
    for candidate in candidates {
        let n = overlap(text, &text_of(candidate));
        if n > top {
            best = Some(candidate);
            top = n;
        }
    }
    best
}

/// Whether the model a record asked for is the one its transcript showed; either may be longer.
#[must_use]
pub fn same_model(asked: &str, used: &str) -> bool {
    let a = asked.to_lowercase();
    let u = used.to_lowercase();
    u.contains(&a) || a.contains(&u)
}
