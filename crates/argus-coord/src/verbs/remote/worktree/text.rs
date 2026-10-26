//! Sizes and ages as the worktree listings print them.

use serde_json::Value;

use crate::verbs::remote::support::text::{num, round};

/// `1536` as `1.5 GB`, `300` as `300 MB`, `0.2` as `under 1 MB`.
pub(super) fn size_text(mb: Option<&Value>) -> String {
    let Some(n) = mb.and_then(Value::as_f64) else {
        return "unknown size".to_owned();
    };
    if n < 1.0 {
        return "under 1 MB".to_owned();
    }
    if n < 1024.0 {
        return format!("{} MB", num(round(n)));
    }
    let gb = round(n / 1024.0 * 10.0) / 10.0;
    if gb.fract() == 0.0 {
        format!("{} GB", num(gb))
    } else {
        format!("{gb:.1} GB")
    }
}

/// `about 1.5 GB` for a size of a megabyte or more, else the plain size text.
pub(super) fn about_text(mb: Option<&Value>) -> String {
    match mb.and_then(Value::as_f64) {
        Some(n) if n >= 1.0 => format!("about {}", size_text(mb)),
        _ => size_text(mb),
    }
}

/// `1 day`, `3 days`.
pub(super) fn days_text(n: f64) -> String {
    let plural = if (n - 1.0).abs() < f64::EPSILON {
        ""
    } else {
        "s"
    };
    format!("{} day{plural}", num(n))
}
