//! Applies the rules of a route to one answer before the two sides are compared: ignored values
//! are removed, rows of a server's own life are dropped, and arrays that carry no promised order
//! are sorted.
// test-tag: critical

use serde_json::{Map, Value};

use crate::pattern::matches;
use crate::rules::Spec;

/// The value as one string whatever the key order of its objects.
#[must_use]
pub fn canonical(value: &Value) -> String {
    match value {
        Value::Object(map) => {
            let mut keys: Vec<&String> = map.keys().collect();
            keys.sort();
            let parts: Vec<String> = keys
                .into_iter()
                .map(|k| format!("{}:{}", Value::from(k.as_str()), canonical(&map[k])))
                .collect();
            format!("{{{}}}", parts.join(","))
        }
        Value::Array(items) => {
            let parts: Vec<String> = items.iter().map(canonical).collect();
            format!("[{}]", parts.join(","))
        }
        other => other.to_string(),
    }
}

/// The value at a dot path of keys.
fn at_path<'a>(value: &'a Value, dotted: &str) -> Option<&'a Value> {
    dotted.split('.').try_fold(value, |here, key| here.get(key))
}

fn is_dropped(spec: &Spec, path: &[String], row: &Value) -> bool {
    spec.drop_rows
        .iter()
        .filter(|rule| matches(rule.array, path))
        .any(|rule| {
            at_path(row, rule.field)
                .and_then(Value::as_str)
                .is_some_and(|text| rule.prefixes.iter().any(|p| text.starts_with(p)))
        })
}

fn sort_key(row: &Value, field: &str) -> (String, String) {
    let by = if field.is_empty() {
        canonical(row)
    } else {
        row.get(field).map(canonical).unwrap_or_default()
    };
    (by, canonical(row))
}

fn array(items: &[Value], spec: &Spec, path: &mut Vec<String>) -> Value {
    let kept: Vec<&Value> = items
        .iter()
        .filter(|row| !is_dropped(spec, path, row))
        .collect();
    let mut out: Vec<Value> = kept
        .into_iter()
        .enumerate()
        .map(|(i, row)| {
            path.push(i.to_string());
            let one = walk(row, spec, path);
            path.pop();
            one
        })
        .collect();
    if let Some((_, field)) = spec.unordered.iter().find(|(p, _)| matches(p, path)) {
        out.sort_by_cached_key(|row| sort_key(row, field));
    }
    Value::Array(out)
}

fn object(map: &Map<String, Value>, spec: &Spec, path: &mut Vec<String>) -> Value {
    let mut out = Map::new();
    for (key, child) in map {
        path.push(key.clone());
        if !spec.ignore.iter().any(|p| matches(p, path)) {
            out.insert(key.clone(), walk(child, spec, path));
        }
        path.pop();
    }
    Value::Object(out)
}

fn walk(value: &Value, spec: &Spec, path: &mut Vec<String>) -> Value {
    match value {
        Value::Object(map) => object(map, spec, path),
        Value::Array(items) => array(items, spec, path),
        other => other.clone(),
    }
}

/// The answer with the rules of `spec` applied.
#[must_use]
pub fn normalise(value: &Value, spec: &Spec) -> Value {
    walk(value, spec, &mut Vec::new())
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::{canonical, normalise};
    use crate::rules::{RowDrop, Spec};

    static SPEC: Spec = Spec {
        ignore: &["seq", "docs.*.data.at"],
        unordered: &[("docs", "id")],
        drop_rows: &[RowDrop {
            array: "log",
            field: "text",
            prefixes: &["started"],
        }],
    };

    #[test]
    fn canonical_ignores_key_order() {
        assert_eq!(
            canonical(&json!({ "a": 1, "b": [2] })),
            canonical(&json!({ "b": [2], "a": 1 }))
        );
    }

    #[test]
    fn ignored_values_are_removed_at_any_listed_path() {
        let v = json!({ "seq": 4, "docs": [{ "id": "a", "data": { "at": 1, "x": 2 } }] });
        let n = normalise(&v, &SPEC);
        assert_eq!(n, json!({ "docs": [{ "id": "a", "data": { "x": 2 } }] }));
    }

    #[test]
    fn unordered_arrays_are_sorted_by_their_field() {
        let v = json!({ "docs": [{ "id": "b" }, { "id": "a" }] });
        assert_eq!(
            normalise(&v, &SPEC),
            json!({ "docs": [{ "id": "a" }, { "id": "b" }] })
        );
    }

    #[test]
    fn rows_that_start_with_a_dropped_prefix_are_left_out() {
        let v = json!({ "log": [{ "text": "started up" }, { "text": "kept" }] });
        assert_eq!(normalise(&v, &SPEC), json!({ "log": [{ "text": "kept" }] }));
    }
}
