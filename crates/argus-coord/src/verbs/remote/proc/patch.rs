//! The options of `coord proc rule set` as the patch the dashboard applies: a value `none` (or
//! the empty text) removes a field.

use argus_core::js;
use serde_json::{Map, Value, json};

use crate::cli::exit::CoordError;
use crate::cli::{Opt, Options, Val};
use crate::verbs::remote::support::text::{amount_and_unit, number_of};

fn val_text(val: &Val) -> String {
    match val {
        Val::Flag => "true".to_owned(),
        Val::Text(text) => text.clone(),
    }
}

/// `String(options[name])`.
fn typed(options: &Options, name: &str) -> Option<String> {
    Some(match options.get(name)? {
        Opt::One(val) => val_text(val),
        Opt::Many(list) => list.iter().map(val_text).collect::<Vec<_>>().join(","),
    })
}

/// A text where `none` and the empty text mean "remove it".
fn text_or_null(text: &str) -> Value {
    if text == "none" || text.is_empty() {
        Value::Null
    } else {
        json!(text)
    }
}

/// A duration such as `30m`, `6h` or `2d` (a bare number is minutes), in minutes.
pub(super) fn minutes(typed: &str, what: &str) -> Result<Value, CoordError> {
    let refusal = || {
        CoordError::refuse(format!(
            "{what} takes a duration such as 30m, 6h or 2d (got \"{typed}\")."
        ))
    };
    let (n, unit) = amount_and_unit(typed).ok_or_else(refusal)?;
    let factor = match unit.as_str() {
        "" | "m" | "min" => 1.0,
        "h" => 60.0,
        "d" => 1440.0,
        _ => return Err(refusal()),
    };
    Ok(js::number(n * factor))
}

fn match_part(options: &Options, rule: &mut Map<String, Value>) {
    let mut found = Map::new();
    for (option, key) in [
        ("name-match", "name"),
        ("cmd-match", "cmd"),
        ("path-match", "path"),
    ] {
        if matches!(options.get(option), Some(Opt::One(Val::Flag)) | None) {
            continue;
        }
        if let Some(text) = typed(options, option) {
            found.insert(key.to_owned(), text_or_null(&text));
        }
    }
    if !found.is_empty() {
        rule.insert("match".to_owned(), Value::Object(found));
    }
}

fn ignore_part(options: &Options, rule: &mut Map<String, Value>) {
    let list = options.many("ignore");
    if list.is_empty() {
        return;
    }
    let only_none = matches!(list.as_slice(), [Val::Text(t)] if t == "none");
    let value = if only_none {
        Value::Null
    } else {
        Value::Array(list.iter().map(Val::to_json).collect())
    };
    rule.insert("ignore".to_owned(), value);
}

fn memory_part(options: &Options, rule: &mut Map<String, Value>) -> Result<(), CoordError> {
    for (option, key) in [
        ("min-mem", "minMemMB"),
        ("max-mem", "maxMemMB"),
        ("max-group-mem", "maxGroupMemMB"),
    ] {
        let Some(text) = typed(options, option) else {
            continue;
        };
        let value = if text == "none" {
            Value::Null
        } else if !text.is_empty() && text.chars().all(|c| c.is_ascii_digit()) {
            js::number(number_of(&text))
        } else {
            return Err(CoordError::refuse(format!(
                "--{option} takes megabytes (a whole number) or none."
            )));
        };
        rule.insert(key.to_owned(), value);
    }
    Ok(())
}

fn time_part(options: &Options, rule: &mut Map<String, Value>) -> Result<(), CoordError> {
    for (option, key) in [("max-age", "maxAgeMin"), ("idle", "idleMin")] {
        let Some(text) = typed(options, option) else {
            continue;
        };
        let value = if text == "none" {
            Value::Null
        } else {
            minutes(&text, &format!("--{option}"))?
        };
        rule.insert(key.to_owned(), value);
    }
    Ok(())
}

fn switch_part(options: &Options, rule: &mut Map<String, Value>) {
    for (on, off, key) in [
        ("orphan", "no-orphan", "orphan"),
        ("hung", "no-hung", "hung"),
        ("killable", "not-killable", "killable"),
        ("only-flagged", "all-listed", "onlyFlagged"),
    ] {
        if options.flag(on) {
            rule.insert(key.to_owned(), json!(true));
        }
        if options.flag(off) {
            rule.insert(key.to_owned(), json!(false));
        }
    }
}

/// The patch the options describe, an empty one when none of them names a field.
///
/// # Errors
/// A memory that is not a whole number of megabytes, or a duration that is not one.
pub(super) fn patch_of(options: &Options) -> Result<Map<String, Value>, CoordError> {
    let mut rule = Map::new();
    match_part(options, &mut rule);
    ignore_part(options, &mut rule);
    memory_part(options, &mut rule)?;
    time_part(options, &mut rule)?;
    switch_part(options, &mut rule);
    if let Some(note) = typed(options, "note") {
        rule.insert("note".to_owned(), text_or_null(&note));
    }
    Ok(rule)
}

/// A duration option as `proc kill-rule --older-than` reads it.
///
/// # Errors
/// The option is not a duration.
pub(super) fn older_than(options: &Options) -> Result<Option<Value>, CoordError> {
    typed(options, "older-than")
        .map(|text| minutes(&text, "--older-than"))
        .transpose()
}
