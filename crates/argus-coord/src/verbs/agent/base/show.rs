//! What a table prints: a cell as a template literal prints it, and a text clipped as the listing
//! clips it.

use argus_board::json::display;
use argus_core::js;
use serde_json::Value;

/// `String(v ?? '')`: nothing for `null` and for a missing field, else the text of the value.
#[must_use]
pub fn cell(v: &Value) -> String {
    if v.is_null() {
        String::new()
    } else {
        display(v)
    }
}

/// `clipText` for a line the terminal shows: the text cut at `max - 1` UTF-16 units and an
/// ellipsis. A cut inside an emoji leaves half of it, which the terminal prints as the replacement
/// character, so the cut text is decoded the same way.
#[must_use]
pub fn clip(text: &str, max: usize) -> String {
    if js::len(text) <= max {
        return text.to_owned();
    }
    let units: Vec<u16> = text.encode_utf16().take(max.saturating_sub(1)).collect();
    format!("{}\u{2026}", String::from_utf16_lossy(&units))
}
