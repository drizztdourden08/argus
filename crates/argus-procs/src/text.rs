//! Small text helpers shared by the rules, the guard and the alerts.

use crate::num::round1;

/// Backslashes become forward slashes, so a rule never needs one.
#[must_use]
pub fn slashed(text: &str) -> String {
    text.replace('\\', "/")
}

/// Executable name without `.exe`, lower case.
#[must_use]
pub fn base_name(name: &str) -> String {
    let lower = name.to_lowercase();
    lower.strip_suffix(".exe").unwrap_or(&lower).to_string()
}

/// `1.9 GB` from 1.0 GB up, else `640 MB`.
#[must_use]
pub fn mem_text(mb: i64) -> String {
    if mb >= 1024 {
        format!("{} GB", round1(crate::num::to_f64(mb) / 1024.0))
    } else {
        format!("{mb} MB")
    }
}

/// `5h` under two days, else `2d`; `40m` under an hour.
#[must_use]
pub fn age_text(ms: i64) -> String {
    let minutes = ((ms + 30_000).div_euclid(60_000)).max(1);
    if minutes < 60 {
        return format!("{minutes}m");
    }
    let hours = (minutes + 30).div_euclid(60);
    if hours < 48 {
        format!("{hours}h")
    } else {
        format!("{}d", (hours + 12).div_euclid(24))
    }
}

/// The command line on one line, clipped to `max` characters.
#[must_use]
pub fn one_line(text: &str, max: usize) -> String {
    text.split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
        .chars()
        .take(max)
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn memory_reads_in_mb_then_gb() {
        assert_eq!(mem_text(1946), "1.9 GB");
        assert_eq!(mem_text(640), "640 MB");
        assert_eq!(mem_text(2048), "2 GB");
    }

    #[test]
    fn ages_read_short() {
        assert_eq!(age_text(10 * 60_000), "10m");
        assert_eq!(age_text(0), "1m");
        assert_eq!(age_text(60 * 60_000), "1h");
        assert_eq!(age_text(3 * 86_400_000), "3d");
    }

    #[test]
    fn names_and_paths_normalise() {
        assert_eq!(base_name("Node.EXE"), "node");
        assert_eq!(slashed("C:\\a\\b"), "C:/a/b");
        assert_eq!(one_line("  a \t b\n c ", 3), "a b");
    }
}
