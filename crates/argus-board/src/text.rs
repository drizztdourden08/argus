//! Text helpers of the coord CLI: clipping and run-name slugs.

use argus_core::js;

/// `clipText`: the text cut to `max` UTF-16 units with an ellipsis when it was longer.
#[must_use]
pub fn clip_text(text: &str, max: usize) -> String {
    if js::len(text) > max {
        format!("{}…", js::slice(text, max.saturating_sub(1)))
    } else {
        text.to_owned()
    }
}

/// `slug`: lower case, runs of other characters become one dash, cut to 60.
#[must_use]
pub fn slug(text: &str) -> String {
    let mut out = String::new();
    let mut dash = false;
    for c in js::trim(text).to_lowercase().chars() {
        if c.is_ascii_lowercase() || c.is_ascii_digit() || "._~:@+-".contains(c) {
            out.push(c);
            dash = false;
        } else if !dash {
            out.push('-');
            dash = true;
        }
    }
    out.trim_matches('-').chars().take(60).collect()
}
