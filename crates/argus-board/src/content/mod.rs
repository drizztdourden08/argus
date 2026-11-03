//! The content checker: one registry field, one value, one verdict. Every op that writes a field
//! of an item calls [`check`]; the web runs the same rules from the same registry and the shared
//! case file (`registry/content-cases.json`), so the two cannot drift. The rules are line based
//! on purpose (regular expressions per line, no Markdown parser) so both sides answer alike.

mod line;
mod plain;
mod reference;
mod rich;

#[cfg(test)]
mod cases;

use argus_core::js;
use serde_json::Value;

use crate::error::{BoardError, refuse};
use crate::registry::{ContentType, Field, registry};

/// What the checker may know about the record a value belongs to.
#[derive(Debug, Default, Clone, Copy)]
pub struct Context<'a> {
    /// The numbers of the images the record has (`![caption](img:n)` may name these).
    pub images: &'a [u32],
    /// The ids on the board; when set, a ref must be one of them.
    pub ids: Option<&'a [String]>,
    /// The threads of the board; when set, a person must be one of them.
    pub threads: Option<&'a [String]>,
}

/// A refusal: a stable key that both implementations agree on, and the message for the user.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Rejection {
    /// The stable key (`rich-lone-paragraph`), the same in the case file and the web.
    pub key: &'static str,
    /// What to tell the user, with what to do instead.
    pub message: String,
}

/// A refusal with its key and message.
pub(crate) fn reject(key: &'static str, message: impl Into<String>) -> Rejection {
    Rejection {
        key,
        message: message.into(),
    }
}

/// The checked value of `field`: trimmed text, the list of checked entries, or `Null` for an
/// empty value of a field that is not required. Existence checks (a ref on the board, a thread)
/// run only when `cx` carries the ids or threads.
///
/// # Errors
/// [`BoardError::Refused`] with the message of the rule the value breaks.
pub fn check(field: &str, value: &Value, cx: &Context<'_>) -> Result<Value, BoardError> {
    check_keyed(field, value, cx).map_err(|r| refuse(r.message))
}

/// [`check`] with the refusal kept whole, key included: what the case file runner compares.
///
/// # Errors
/// A [`Rejection`] naming the rule the value breaks.
pub fn check_keyed(field: &str, value: &Value, cx: &Context<'_>) -> Result<Value, Rejection> {
    let Some(spec) = registry().field(field) else {
        return Err(reject(
            "unknown-field",
            format!("{field} is not a field of the item registry."),
        ));
    };
    if is_blank(value) {
        return if spec.required {
            Err(reject("required", format!("{field} is required.")))
        } else {
            Ok(Value::Null)
        };
    }
    if spec.list.is_some() || spec.content == ContentType::Refs {
        list(field, spec, value, cx)
    } else {
        one(field, spec, value, cx)
    }
}

/// Nothing was given: null, a text of white space, or an empty list.
fn is_blank(value: &Value) -> bool {
    match value {
        Value::Null => true,
        Value::String(s) => js::trim(s).is_empty(),
        Value::Array(items) => items.is_empty(),
        _ => false,
    }
}

/// The text of a value, refused when it is not one.
fn text_of<'a>(field: &str, value: &'a Value) -> Result<&'a str, Rejection> {
    value
        .as_str()
        .ok_or_else(|| reject("not-text", format!("{field} must be a text.")))
}

/// Refuses a text longer than `max` UTF-16 units, as every other length check of the board does.
pub(crate) fn within(field: &str, text: &str, max: Option<usize>) -> Result<(), Rejection> {
    let len = js::len(text);
    match max {
        Some(max) if len > max => Err(reject(
            "too-long",
            format!("{field} is {len} chars, the limit is {max}. Shorten it."),
        )),
        _ => Ok(()),
    }
}

/// One value of a field that is not a list.
fn one(field: &str, spec: &Field, value: &Value, cx: &Context<'_>) -> Result<Value, Rejection> {
    let text = || text_of(field, value);
    match spec.content {
        ContentType::Line => line::check(field, spec, text()?),
        ContentType::Slug => plain::slug(field, spec, text()?),
        ContentType::Rich => rich::check(field, spec, text()?, cx),
        ContentType::Enum => plain::choice(field, spec, text()?),
        ContentType::Date => plain::date(field, text()?),
        ContentType::Ref | ContentType::Refs => reference::id(field, spec, text()?, cx),
        ContentType::Person => reference::person(field, spec, text()?, cx),
        ContentType::Records => records(field, value),
    }
}

/// A list of values, each checked like a single one, at most the field's `list` entries.
fn list(field: &str, spec: &Field, value: &Value, cx: &Context<'_>) -> Result<Value, Rejection> {
    let Value::Array(items) = value else {
        return Err(reject("not-a-list", format!("{field} must be a list.")));
    };
    if let Some(max) = spec.list.filter(|max| items.len() > *max) {
        return Err(reject(
            "list-too-long",
            format!("{} {field} entries, the limit is {max}.", items.len()),
        ));
    }
    let mut checked = Vec::with_capacity(items.len());
    for item in items {
        if is_blank(item) {
            return Err(reject("required", format!("{field} has an empty entry.")));
        }
        checked.push(one(field, spec, item, cx)?);
    }
    Ok(Value::Array(checked))
}

/// A list of records kept by an op: only the shape is checked here.
fn records(field: &str, value: &Value) -> Result<Value, Rejection> {
    let Value::Array(items) = value else {
        return Err(reject("not-a-list", format!("{field} must be a list.")));
    };
    if items.iter().any(|i| !i.is_object()) {
        return Err(reject(
            "bad-record",
            format!("{field} holds records (objects)."),
        ));
    }
    Ok(value.clone())
}
