//! The `--note` of a verb that records what the Overlord did.

use argus_core::js;

use super::text::collapse;
use crate::cli::exit::CoordError;
use crate::cli::route::Call;

/// The collapsed `--note`, `min` to `max` characters long; the refusal says `ask`, what to write.
///
/// # Errors
/// A note that is missing or out of bounds.
pub fn bounded(
    call: &Call<'_>,
    (min, max): (usize, usize),
    ask: &str,
) -> Result<String, CoordError> {
    let note = call.options.text("note").map(collapse).unwrap_or_default();
    let len = js::len(&note);
    if (min..=max).contains(&len) {
        return Ok(note);
    }
    Err(CoordError::refuse(format!(
        "--note is required, {min} to {max} characters: {ask} (it is {len})."
    )))
}
