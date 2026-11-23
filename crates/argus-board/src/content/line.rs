//! The `line` kind: one trimmed line of plain text, inline code allowed.

use argus_core::js;
use serde_json::Value;

use super::{Rejection, reject, within};
use crate::registry::registry;

/// The trimmed line, refused when it breaks over lines, opens with Markdown block syntax or is
/// over `max`.
pub(super) fn check(field: &str, text: &str, max: Option<usize>) -> Result<Value, Rejection> {
    let kind = &registry().content.line;
    let text = js::trim(text);
    if !kind.newlines && text.contains(['\n', '\r']) {
        return Err(reject("newline", format!("{field} must be one line.")));
    }
    if !kind.markup && kind.block.0.is_match(text) {
        return Err(reject(
            "line-markup",
            format!(
                "{field} is plain text: it cannot start with #, |, >, a code fence or a list marker."
            ),
        ));
    }
    within(field, text, max)?;
    Ok(Value::String(text.to_owned()))
}
