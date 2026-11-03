//! The reason that overrides a rule which refuses (`force` on the op).

use serde_json::Value;

use crate::content::{Context, check};
use crate::error::BoardError;

/// The reason an op carries in `force`, checked as a `reason`. A `force` that is not a text (the
/// flag that skips the revision check) is no reason; a blank one is none either.
pub(crate) fn reason(op: &Value) -> Result<Option<String>, BoardError> {
    let Some(text) = op["force"].as_str() else {
        return Ok(None);
    };
    let checked = check(
        "reason",
        &Value::String(text.to_owned()),
        &Context::default(),
    )?;
    Ok(checked.as_str().map(str::to_owned))
}
