//! A small JSON-schema checker for tool arguments: just the keywords the tool definitions use,
//! so a bad call is refused with a plain message before any service is touched.

use argus_core::js;
use regex::Regex;
use serde_json::Value;

fn type_of(v: &Value) -> &'static str {
    match v {
        Value::Null => "null",
        Value::Array(_) => "array",
        Value::Bool(_) => "boolean",
        Value::String(_) => "string",
        Value::Object(_) => "object",
        Value::Number(n) => {
            if n.is_f64() && n.as_f64().is_some_and(|f| f.fract() != 0.0) {
                "number"
            } else {
                "integer"
            }
        }
    }
}

fn type_ok(wanted: &str, v: &Value) -> bool {
    let actual = type_of(v);
    if wanted == "number" {
        return actual == "number" || actual == "integer";
    }
    actual == wanted
}

fn type_error(wanted: &str, path: &str) -> String {
    if wanted == "integer" {
        format!("{path} must be an integer")
    } else {
        format!("{path} must be a {wanted}")
    }
}

fn enum_error(schema: &Value, value: &Value, path: &str, errors: &mut Vec<String>) {
    let Some(allowed) = schema["enum"].as_array() else {
        return;
    };
    if !allowed.contains(value) {
        let names: Vec<String> = allowed.iter().map(js::string).collect();
        errors.push(format!("{path} must be one of {}", names.join(", ")));
    }
}

fn js_pattern(pattern: &str) -> Option<Regex> {
    Regex::new(&format!("(?-u){pattern}")).ok()
}

fn string_errors(schema: &Value, s: &str, path: &str, errors: &mut Vec<String>) {
    let len = js::len(s) as u64;
    if let Some(min) = schema["minLength"].as_u64().filter(|m| len < *m) {
        errors.push(format!("{path} must be at least {min} characters"));
    }
    if let Some(max) = schema["maxLength"].as_u64().filter(|m| len > *m) {
        errors.push(format!("{path} is {len} characters, the limit is {max}"));
    }
    if let Some(pattern) = schema["pattern"].as_str() {
        if !js_pattern(pattern).is_some_and(|re| re.is_match(s)) {
            errors.push(format!("{path} does not match {pattern}"));
        }
    }
}

fn number_errors(schema: &Value, n: f64, path: &str, errors: &mut Vec<String>) {
    if let Some(min) = schema["minimum"].as_f64().filter(|m| n < *m) {
        errors.push(format!(
            "{path} must be at least {}",
            js::string(&js::number(min))
        ));
    }
    if let Some(max) = schema["maximum"].as_f64().filter(|m| n > *m) {
        errors.push(format!(
            "{path} must be at most {}",
            js::string(&js::number(max))
        ));
    }
}

fn array_errors(schema: &Value, list: &[Value], path: &str, errors: &mut Vec<String>) {
    if let Some(max) = schema["maxItems"]
        .as_u64()
        .filter(|m| list.len() as u64 > *m)
    {
        errors.push(format!("{path} has more than {max} items"));
    }
    if let Some(item) = schema.get("items") {
        for (i, v) in list.iter().enumerate() {
            errors.extend(validate(item, v, &format!("{path}[{i}]")));
        }
    }
}

fn object_errors(
    schema: &Value,
    map: &serde_json::Map<String, Value>,
    path: &str,
    errors: &mut Vec<String>,
) {
    for key in schema["required"].as_array().into_iter().flatten() {
        let key = key.as_str().unwrap_or_default();
        if !map.contains_key(key) {
            errors.push(format!("{path}.{key} is required"));
        }
    }
    for (key, v) in map {
        match schema["properties"].get(key) {
            Some(prop) => errors.extend(validate(prop, v, &format!("{path}.{key}"))),
            None if schema["additionalProperties"] == false => {
                errors.push(format!("{path}.{key} is not a known argument"));
            }
            None => {}
        }
    }
}

/// The problems of `value` against `schema`, empty when it fits.
#[must_use]
pub fn validate(schema: &Value, value: &Value, path: &str) -> Vec<String> {
    let mut errors = Vec::new();
    if let Some(wanted) = schema["type"].as_str() {
        if !type_ok(wanted, value) {
            errors.push(type_error(wanted, path));
            return errors;
        }
    }
    enum_error(schema, value, path, &mut errors);
    match value {
        Value::String(s) => string_errors(schema, s, path, &mut errors),
        Value::Number(n) => number_errors(schema, n.as_f64().unwrap_or(0.0), path, &mut errors),
        Value::Array(list) => array_errors(schema, list, path, &mut errors),
        Value::Object(map) => object_errors(schema, map, path, &mut errors),
        _ => {}
    }
    errors
}
