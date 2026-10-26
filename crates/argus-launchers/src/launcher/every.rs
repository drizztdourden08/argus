//! The usage window of a standalone launcher (`--use-every`): a whole number of hours (h),
//! days (d), weeks (w) or months of 30 days (mo), from 1 hour to 365 days.

use std::sync::LazyLock;

use regex::Regex;

use super::record::RuleError;
use crate::time::DAY_MS;

pub const EVERY_LEGACY: &str = "7d";
pub const EVERY_HELP: &str = "write it like 12h, 3d, 2w or 1mo (1mo = 30 days), from 1h to 365d";
const HOUR_MS: i64 = 3_600_000;
const EVERY_MIN_MS: i64 = HOUR_MS;
const EVERY_MAX_MS: i64 = 365 * DAY_MS;

static EVERY_RE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new("^([1-9][0-9]{0,4})(h|d|w|mo)$").unwrap());

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Every {
    pub text: String,
    pub ms: i64,
}

fn unit_ms(unit: &str) -> i64 {
    match unit {
        "h" => HOUR_MS,
        "d" => DAY_MS,
        "w" => 7 * DAY_MS,
        _ => 30 * DAY_MS,
    }
}

/// `"7d"` as `{ text: "7d", ms }`; anything else is a plain sentence.
pub fn parse_every(value: &str) -> Result<Every, RuleError> {
    let raw = value.trim();
    let text = raw.to_lowercase();
    let captures = EVERY_RE.captures(&text).ok_or_else(|| {
        RuleError::new(format!(
            "--use-every \"{raw}\" is not a duration: {EVERY_HELP}."
        ))
    })?;
    let count: i64 = captures[1].parse().unwrap_or(0);
    let ms = count * unit_ms(&captures[2]);
    if !(EVERY_MIN_MS..=EVERY_MAX_MS).contains(&ms) {
        return Err(RuleError::new(format!(
            "--use-every {text} is out of range: {EVERY_HELP}."
        )));
    }
    Ok(Every { text, ms })
}

/// `40m` under an hour, `9h` under two days, else `9d`.
pub fn duration_text(ms: i64) -> String {
    let minutes = ((ms + 30_000) / 60_000).max(1);
    if minutes < 60 {
        return format!("{minutes}m");
    }
    let hours = (minutes + 30) / 60;
    if hours < 48 {
        format!("{hours}h")
    } else {
        format!("{}d", (hours + 12) / 24)
    }
}
