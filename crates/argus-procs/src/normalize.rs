//! Validation of a rule given as JSON: a clean copy, or a `bad_request` that says what is wrong.

use crate::error::ProcsError;
use crate::rule::{Match, Rule};
use fancy_regex::Regex;
use serde_json::{Map, Value};

pub const NOTE_MAX: usize = 300;
const MATCH_KEYS: [&str; 3] = ["name", "cmd", "path"];
const NUMBER_KEYS: [&str; 5] = [
    "minMemMB",
    "maxMemMB",
    "maxGroupMemMB",
    "maxAgeMin",
    "idleMin",
];
const BOOL_KEYS: [&str; 4] = ["onlyFlagged", "orphan", "hung", "killable"];
const MAX_NUMBER: f64 = 10_000_000.0;
const MAX_IGNORE: usize = 20;
const MAX_REGEX: usize = 300;

type Checked<T> = Result<T, ProcsError>;

fn valid_name(name: &str) -> bool {
    let mut chars = name.chars();
    let first = chars
        .next()
        .is_some_and(|c| c.is_ascii_lowercase() || c.is_ascii_digit());
    first
        && name.len() <= 40
        && chars.all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-')
}

fn check_regex(what: &str, value: &Value) -> Checked<String> {
    let src = value
        .as_str()
        .filter(|s| !s.is_empty() && s.len() <= MAX_REGEX);
    let src = src.ok_or_else(|| {
        ProcsError::bad_request(format!(
            "{what} must be a regex of 1 to {MAX_REGEX} characters"
        ))
    })?;
    Regex::new(&format!("(?i){src}"))
        .map_err(|e| ProcsError::bad_request(format!("{what} is not a valid regex: {e}")))?;
    Ok(src.to_string())
}

fn unset(value: &Value) -> bool {
    value.is_null() || value.as_str() == Some("")
}

fn check_match(input: &Value) -> Checked<Match> {
    let empty = Map::new();
    let map = match input.get("match") {
        None | Some(Value::Null) => &empty,
        Some(Value::Object(map)) => map,
        Some(_) => {
            return Err(ProcsError::bad_request(
                "match must be an object of name / cmd / path regexes",
            ));
        }
    };
    if let Some(key) = map.keys().find(|k| !MATCH_KEYS.contains(&k.as_str())) {
        return Err(ProcsError::bad_request(format!(
            "match.{key} is not known (name, cmd, path)"
        )));
    }
    let pick = |key: &str| -> Checked<Option<String>> {
        match map.get(key) {
            Some(v) if !unset(v) => check_regex(&format!("match.{key}"), v).map(Some),
            _ => Ok(None),
        }
    };
    Ok(Match {
        name: pick("name")?,
        cmd: pick("cmd")?,
        path: pick("path")?,
    })
}

fn check_ignore(input: &Value) -> Checked<Option<Vec<String>>> {
    let list: Vec<&Value> = match input.get("ignore") {
        None | Some(Value::Null) => return Ok(None),
        Some(Value::Array(items)) => items.iter().collect(),
        Some(single) => vec![single],
    };
    if list.len() > MAX_IGNORE {
        return Err(ProcsError::bad_request(format!(
            "ignore takes at most {MAX_IGNORE} regexes"
        )));
    }
    let kept = list
        .into_iter()
        .filter(|v| v.as_str() != Some(""))
        .enumerate()
        .map(|(i, v)| check_regex(&format!("ignore[{i}]"), v))
        .collect::<Checked<Vec<String>>>()?;
    Ok((!kept.is_empty()).then_some(kept))
}

fn number_of(value: &Value) -> Option<f64> {
    match value {
        Value::Number(n) => n.as_f64(),
        Value::String(s) => s.trim().parse().ok(),
        Value::Bool(b) => Some(f64::from(u8::from(*b))),
        _ => None,
    }
}

fn check_number(input: &Value, key: &str) -> Checked<Option<f64>> {
    let Some(value) = input.get(key).filter(|v| !unset(v)) else {
        return Ok(None);
    };
    let n = number_of(value)
        .filter(|n| n.is_finite() && (0.0..=MAX_NUMBER).contains(n))
        .ok_or_else(|| ProcsError::bad_request(format!("{key} must be a number of 0 or more")))?;
    Ok((n > 0.0).then(|| (n * 100.0).round() / 100.0))
}

fn check_bool(input: &Value, key: &str) -> Checked<Option<bool>> {
    match input.get(key) {
        None | Some(Value::Null) => Ok(None),
        Some(Value::Bool(b)) => Ok(Some(*b)),
        Some(_) => Err(ProcsError::bad_request(format!(
            "{key} must be true or false"
        ))),
    }
}

fn check_note(input: &Value) -> Checked<Option<String>> {
    let Some(value) = input.get("note").filter(|v| !unset(v)) else {
        return Ok(None);
    };
    let text = value
        .as_str()
        .map_or_else(|| value.to_string(), str::to_string);
    let note = text.split_whitespace().collect::<Vec<_>>().join(" ");
    if note.chars().count() > NOTE_MAX {
        return Err(ProcsError::bad_request(format!(
            "note is at most {NOTE_MAX} characters"
        )));
    }
    Ok(Some(note))
}

/// A clean copy of a rule, or a `bad_request` that says what is wrong.
///
/// # Errors
/// A `bad_request` naming the first field that is not valid.
pub fn normalize_rule(input: &Value) -> Checked<Rule> {
    if !input.is_object() {
        return Err(ProcsError::bad_request("a rule must be an object"));
    }
    let name = input.get("name").and_then(Value::as_str).unwrap_or("");
    if !valid_name(name) {
        return Err(ProcsError::bad_request(format!(
            "rule name \"{name}\" must be lower case letters, digits and dashes (1 to 40)"
        )));
    }
    let n = |i: usize| check_number(input, NUMBER_KEYS[i]);
    let b = |i: usize| check_bool(input, BOOL_KEYS[i]);
    Ok(Rule {
        name: name.to_string(),
        matcher: check_match(input)?,
        ignore: check_ignore(input)?,
        min_mem_mb: n(0)?,
        max_mem_mb: n(1)?,
        max_group_mem_mb: n(2)?,
        max_age_min: n(3)?,
        idle_min: n(4)?,
        only_flagged: b(0)?,
        orphan: b(1)?,
        hung: b(2)?,
        killable: b(3)?,
        note: check_note(input)?,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn message(input: &Value) -> String {
        normalize_rule(input).unwrap_err().message().to_string()
    }

    #[test]
    fn a_rule_is_validated_name_regexes_numbers_booleans_note() {
        assert!(message(&json!({"name": "Bad Name"})).contains("lower case"));
        assert!(
            message(&json!({"name": "x", "match": {"cmd": "("}}))
                .contains("match.cmd is not a valid regex")
        );
        assert!(
            message(&json!({"name": "x", "match": {"exe": "a"}}))
                .contains("match.exe is not known")
        );
        assert!(
            message(&json!({"name": "x", "maxMemMB": -1})).contains("maxMemMB must be a number")
        );
        assert!(
            message(&json!({"name": "x", "killable": "yes"}))
                .contains("killable must be true or false")
        );
        assert!(
            message(&json!({"name": "x", "ignore": ["["]}))
                .contains("ignore[0] is not a valid regex")
        );
        assert!(message(&json!("nope")).contains("a rule must be an object"));
    }

    #[test]
    fn a_valid_rule_comes_out_clean() {
        let rule = normalize_rule(&json!({
            "name": "x", "match": {"name": "a", "cmd": ""}, "maxMemMB": "500",
            "idleMin": 0, "orphan": true, "note": "  two   words "
        }))
        .unwrap();
        assert_eq!(
            serde_json::to_value(&rule).unwrap(),
            json!({"name": "x", "match": {"name": "a"}, "maxMemMB": 500, "orphan": true, "note": "two words"})
        );
    }
}
