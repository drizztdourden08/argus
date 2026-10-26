//! Reading options the way the verbs of `coord` do: a value that must be one text, and the
//! JavaScript `String(...)` of any option.

use argus_board::limits::optional_text;
use argus_core::js;
use serde_json::Value;

use crate::cli::exit::CoordError;
use crate::cli::{Opt, Options, Val};

/// A text flag the way the launcher rules take it: not given (`None`), given blank so that it
/// clears the field (`Some(None)`), or set.
#[allow(clippy::option_option)]
pub type Text = Option<Option<String>>;

/// `value(options, key)` of the agent verbs: `None` when absent, the text when it has one, a
/// refusal for a bare flag or a repeated option.
///
/// # Errors
/// `--<key> needs a value.`
pub fn value<'a>(options: &'a Options, key: &str) -> Result<Option<&'a str>, CoordError> {
    match options.get(key) {
        None => Ok(None),
        Some(Opt::One(Val::Text(text))) => Ok(Some(text)),
        Some(_) => Err(CoordError::refuse(format!("--{key} needs a value."))),
    }
}

/// Whether `options[key] !== undefined`.
#[must_use]
pub fn given(options: &Options, key: &str) -> bool {
    options.get(key).is_some()
}

/// Whether the option is a bare flag or was given twice: `=== true || Array.isArray(...)`.
#[must_use]
pub fn not_one_text(options: &Options, key: &str) -> bool {
    matches!(options.get(key), Some(Opt::One(Val::Flag) | Opt::Many(_)))
}

/// Whether the option was given more than once: `Array.isArray(options[key])`.
#[must_use]
pub fn is_list(options: &Options, key: &str) -> bool {
    matches!(options.get(key), Some(Opt::Many(_)))
}

/// `String(options[key])` of a given option.
#[must_use]
pub fn js_string(options: &Options, key: &str) -> String {
    options
        .json(key)
        .map_or_else(|| "undefined".to_owned(), |v| js::string(&v))
}

/// `options[key]` as a JSON value, `null` when absent.
#[must_use]
pub fn json_or_null(options: &Options, key: &str) -> Value {
    options.json(key).unwrap_or(Value::Null)
}

/// One of the positional words, empty when missing (what `typeof v === 'string' ? v : ''` of
/// the limit checks sees).
#[must_use]
pub fn word(words: &[String], at: usize) -> &str {
    words.get(at).map_or("", String::as_str)
}

/// The sub-verb: the first word, taken off the words that stay for it (empty when there is none).
pub fn take_sub(positional: &mut Vec<String>) -> String {
    if positional.is_empty() {
        String::new()
    } else {
        positional.remove(0)
    }
}

/// The refusal of a sub-verb that does not exist.
#[must_use]
pub fn unknown_sub(group: &str, verbs: &str) -> CoordError {
    CoordError::refuse(format!("{group} verb must be one of {verbs}."))
}

/// A text flag that clears with an empty value: `None` when not given, a refusal for a bare flag
/// or a repeated one, else the checked text (`None` inside for a blank one). `clears` says
/// whether the refusal names the `""` that clears it.
///
/// # Errors
/// `--<name> takes one value`, or the limit of `kind`.
pub fn one_text(
    options: &Options,
    name: &str,
    kind: &str,
    clears: bool,
) -> Result<Text, CoordError> {
    if options.get(name).is_none() {
        return Ok(None);
    }
    if not_one_text(options, name) {
        let suffix = if clears { " (\"\" clears it)" } else { "" };
        return Err(CoordError::refuse(format!(
            "--{name} takes one value{suffix}."
        )));
    }
    Ok(Some(optional_text(kind, &json_or_null(options, name))?))
}
