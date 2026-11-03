//! `--force "<reason>"`: the reason that lets a change past a rule which refuses. The board
//! keeps it as a decision on the item. It never skips the revision check of `--if-rev`.

use serde_json::{Value, json};

use crate::cli::Options;
use crate::verbs::work::run::{Res, bad};

/// The reason of `--force`, `None` without the flag.
///
/// # Errors
/// Refused when the flag has no reason after it.
pub(super) fn reason(options: &Options) -> Res<Option<String>> {
    if let Some(text) = options.text("force") {
        return Ok(Some(text.to_owned()));
    }
    match options.get("force") {
        None => Ok(None),
        Some(_) => Err(bad(
            "--force needs a reason: --force \"<why the rule does not apply>\".",
        )),
    }
}

/// The op with the reason in its `force`.
pub(super) fn attach(mut operation: Value, reason: Option<&str>) -> Value {
    if let Some(text) = reason {
        operation["force"] = json!(text);
    }
    operation
}
