//! Text helpers shared by the policy and the wake producers.

const NOTE_CLIP: usize = 100;

/// Runs of white space become one space and the ends are trimmed.
#[must_use]
pub fn one_line(text: &str) -> String {
    text.split_whitespace().collect::<Vec<_>>().join(" ")
}

/// One line of text cut at `max` characters, with an ellipsis when it was cut.
#[must_use]
pub fn clip(text: &str, max: usize) -> String {
    let line = one_line(text);
    if line.chars().count() > max {
        let head: String = line.chars().take(max.saturating_sub(1)).collect();
        format!("{head}…")
    } else {
        line
    }
}

/// One line cut at `max` characters, with no ellipsis.
#[must_use]
pub fn cut(text: &str, max: usize) -> String {
    one_line(text).chars().take(max).collect()
}

/// The first line of an owner entry, with where to read it whole when it was cut.
#[must_use]
pub fn clip_full(entry_id: &str, text: &str) -> String {
    let line = clip(text, NOTE_CLIP);
    if one_line(text).chars().count() > NOTE_CLIP {
        format!("{line} [full text in {entry_id}: coord inbox]")
    } else {
        line
    }
}
