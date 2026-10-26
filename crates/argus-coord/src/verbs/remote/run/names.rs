//! The shapes `run` takes as arguments: a launch name, a run id, an agent id, a bounded number.

use crate::cli::Options;
use crate::cli::exit::CoordError;
use crate::verbs::remote::support::text::{int_within, is_key, number_of};

fn is_name(text: &str) -> bool {
    is_key(text) && text.chars().count() <= 40
}

fn is_id(prefix: &str, text: &str) -> bool {
    text.strip_prefix(prefix)
        .is_some_and(|digits| !digits.is_empty() && digits.chars().all(|c| c.is_ascii_digit()))
}

/// A launch name: lowercase letters, digits and dashes, 1 to 40 characters.
///
/// # Errors
/// The name is missing or has another shape; `what` names the argument in the refusal.
pub fn need_name(value: Option<&String>, what: &str) -> Result<String, CoordError> {
    match value.filter(|v| is_name(v)) {
        Some(name) => Ok(name.clone()),
        None => Err(CoordError::refuse(format!(
            "{what} must be lowercase letters, digits and dashes, 1 to 40 characters, e.g. build or app-dev."
        ))),
    }
}

/// A run id such as `R-3`.
///
/// # Errors
/// The id is missing or has another shape.
pub fn need_rid(value: Option<&String>) -> Result<String, CoordError> {
    match value.filter(|v| is_id("R-", v)) {
        Some(rid) => Ok(rid.clone()),
        None => Err(CoordError::refuse("give the run id, e.g. R-3.")),
    }
}

/// An agent id such as `A-3`.
#[must_use]
pub fn is_agent_id(text: &str) -> bool {
    is_id("A-", text)
}

/// A whole number option within `min..=max`, `fallback` when the option is not given.
///
/// # Errors
/// The option is a flag, not a whole number, or out of range.
pub fn int_option(
    options: &Options,
    key: &str,
    fallback: i64,
    (min, max): (i64, i64),
) -> Result<i64, CoordError> {
    if options.get(key).is_none() {
        return Ok(fallback);
    }
    let n = options.text(key).map_or(f64::NAN, number_of);
    #[allow(clippy::cast_precision_loss)]
    int_within(n, min as f64, max as f64).ok_or_else(|| {
        CoordError::refuse(format!("--{key} takes a whole number from {min} to {max}."))
    })
}
