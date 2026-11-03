//! The description of any item, typed or not: the rich text rule of the registry (bullets and
//! images, no lone paragraph). Text already stored is never rechecked; only a write is.

use serde_json::Value;

use super::images::numbers;
use super::known::Known;
use crate::Board;
use crate::content::check;
use crate::error::BoardError;

/// A description about to be written, checked against the images the record has (`pictures` is
/// the record's `images` list, `Null` for a new item). Nothing given is an empty description.
pub(crate) fn written(board: &Board, pictures: &Value, v: &Value) -> Result<Value, BoardError> {
    let known = Known::of(board, numbers(pictures));
    let checked = check("description", v, &known.context())?;
    Ok(Value::String(
        checked.as_str().unwrap_or_default().to_owned(),
    ))
}
