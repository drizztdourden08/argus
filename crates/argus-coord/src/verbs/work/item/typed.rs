//! The typed part of `item add` and `item set`: `--type`, `--parent`, `--assignee`, `--stage` and
//! the own fields. The board checks every value against the registry; this only reads the flags.

use serde_json::{Map, Value, json};

use super::fields;
use crate::verbs::work::run::{Res, Run, bad};

/// The text of a flag that needs one, `None` when the flag is not given.
fn text_flag(r: &Run<'_>, name: &str) -> Res<Option<String>> {
    if let Some(text) = r.text_opt(name) {
        return Ok(Some(text.trim().to_owned()));
    }
    match r.options.get(name) {
        None => Ok(None),
        Some(_) => Err(bad(format!("--{name} needs a value."))),
    }
}

/// An assignee, where `none` clears it.
fn assignee(text: String) -> Value {
    if text == "none" {
        Value::Null
    } else {
        json!(text)
    }
}

/// The typed fields the call names, as the `set` of a create or an edit. Empty for a plain call.
///
/// # Errors
/// Refused for a flag with no value and for a bad `--field`.
pub(super) fn flags(r: &Run<'_>) -> Res<Map<String, Value>> {
    let mut set = Map::new();
    for key in ["type", "parent", "stage"] {
        if let Some(text) = text_flag(r, key)? {
            set.insert(key.to_owned(), json!(text));
        }
    }
    if let Some(text) = text_flag(r, "assignee")? {
        set.insert("assignee".to_owned(), assignee(text));
    }
    for (key, value) in fields::given(r.options)? {
        set.insert(format!("fields.{key}"), value);
    }
    Ok(set)
}
