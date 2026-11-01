//! `--idle [D]` of `coord worktree request --finished`: a time of 1h to 30d, 6h when none is given.

use crate::cli::exit::CoordError;
use crate::cli::route::Call;
use crate::cli::{Opt, Val};
use crate::verbs::remote::support::text::amount_and_unit;

const DEFAULT_MINUTES: f64 = 360.0;
const MIN_MINUTES: f64 = 60.0;
const MAX_MINUTES: f64 = 43_200.0;

/// `6h`, `2d`, `90m` or a bare number of hours, in minutes; none when the text is not a time.
fn minutes_of(text: &str) -> Option<f64> {
    let (n, unit) = amount_and_unit(text)?;
    let per = match unit.as_str() {
        "m" => 1.0,
        "" | "h" => 60.0,
        "d" => 1440.0,
        _ => return None,
    };
    Some(n * per)
}

/// The idle time asked for, in minutes. A value that is not a time is a worktree again
/// (`--idle X:/w/a`): it joins the positional words and the time is the default.
///
/// # Errors
/// The time is under 1h or over 30d.
pub(super) fn idle_option(call: &mut Call<'_>) -> Result<Option<f64>, CoordError> {
    let typed = match call.options.get("idle") {
        None => return Ok(None),
        Some(Opt::One(Val::Text(text))) if text != "true" => text.clone(),
        Some(_) => return Ok(Some(DEFAULT_MINUTES)),
    };
    match minutes_of(&typed) {
        None => {
            call.positional.push(typed);
            Ok(Some(DEFAULT_MINUTES))
        }
        Some(m) if !(MIN_MINUTES..=MAX_MINUTES).contains(&m) => {
            Err(CoordError::refuse("--idle goes from 1h to 30d."))
        }
        Some(m) => Ok(Some(m)),
    }
}

// test-tag: critical
#[cfg(test)]
mod tests {
    use super::minutes_of;

    #[test]
    fn a_time_is_minutes_hours_or_days_and_bare_numbers_are_hours() {
        let seen: Vec<Option<f64>> = [
            "90m", "6h", "2d", "6", "1.5h", "6 H", "soon", "X:/w/a", "3w",
        ]
        .map(minutes_of)
        .to_vec();
        assert_eq!(
            seen,
            [
                Some(90.0),
                Some(360.0),
                Some(2880.0),
                Some(360.0),
                Some(90.0),
                Some(360.0),
                None,
                None,
                None
            ]
        );
    }
}
