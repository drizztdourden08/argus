//! Output helpers of the coord CLI (`format.mjs`), with JavaScript string widths: lengths,
//! padding and clipping count UTF-16 code units, as `String.prototype.length` and `padEnd` do.

use std::io::Write;

use argus_core::clock::parse_ms;
use argus_core::js;
use serde_json::Value;

pub use argus_board::text::clip_text as clip;

/// `console.log(lines.join('\n'))`: the lines, then a newline. A closed pipe is ignored.
pub fn say(out: &mut dyn Write, lines: &[&str]) {
    writeln!(out, "{}", lines.join("\n")).ok();
}

/// `console.log(JSON.stringify(value, null, 2))`.
pub fn as_json(out: &mut dyn Write, value: &Value) {
    let text = serde_json::to_string_pretty(value).unwrap_or_default();
    say(out, &[&text]);
}

fn is_space(c: char) -> bool {
    (c.is_whitespace() && c != '\u{85}') || c == '\u{feff}'
}

/// `s.padEnd(width)`: spaces to `width` UTF-16 units; a longer text is left alone.
#[must_use]
pub fn pad_end(s: &str, width: usize) -> String {
    let pad = width.saturating_sub(js::len(s));
    format!("{s}{}", " ".repeat(pad))
}

fn widths(rows: &[Vec<String>]) -> Vec<usize> {
    (0..rows[0].len())
        .map(|i| {
            rows.iter()
                .map(|r| r.get(i).map_or(0, |c| js::len(c)))
                .max()
                .unwrap_or(0)
        })
        .collect()
}

/// An aligned table: every column padded to its widest cell, two spaces between columns, each
/// line trimmed at its end. The columns are those of the first row; an empty table is empty.
#[must_use]
pub fn table(rows: &[Vec<String>]) -> String {
    if rows.is_empty() {
        return String::new();
    }
    let widths = widths(rows);
    rows.iter()
        .map(|row| {
            let cells: Vec<String> = row
                .iter()
                .enumerate()
                .map(|(i, c)| pad_end(c, widths.get(i).copied().unwrap_or(0)))
                .collect();
            cells.join("  ").trim_end_matches(is_space).to_owned()
        })
        .collect::<Vec<_>>()
        .join("\n")
}

/// `Math.round` of a non-negative quotient: halves go up.
fn round_div(n: i64, d: i64) -> i64 {
    (2 * n + d) / (2 * d)
}

/// A relative time as the CLI prints it: `never`, `12s ago`, `5m ago`, `3h ago`, `2d ago`.
/// An unparseable time prints `NaNd ago`, as the JavaScript does.
#[must_use]
pub fn ago(iso: Option<&str>, now_ms: i64) -> String {
    let Some(iso) = iso.filter(|s| !s.is_empty()) else {
        return "never".to_owned();
    };
    let Some(then) = parse_ms(iso) else {
        return "NaNd ago".to_owned();
    };
    let s = round_div((now_ms - then).max(0), 1000);
    if s < 60 {
        return format!("{s}s ago");
    }
    let m = round_div(s, 60);
    if m < 60 {
        return format!("{m}m ago");
    }
    let h = round_div(m, 60);
    if h < 48 {
        return format!("{h}h ago");
    }
    format!("{}d ago", round_div(h, 24))
}
