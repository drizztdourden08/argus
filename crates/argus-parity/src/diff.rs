//! A by-value diff of two JSON answers: object key order never matters, numbers compare as
//! numbers, rows with an identity field (`id`, `key`, `name`) are matched by it, and two
//! timestamps are equal when they are within the tolerance: two strings that both parse as a
//! date and time under any key, or two epoch numbers under a timestamp key (`at`, `ts`, `time`,
//! `since` or a name ending in `At`).

use std::collections::HashSet;

use argus_core::clock::parse_ms;
use serde_json::{Map, Value};

/// Fields that name a row, in the order they are tried.
const IDENTITY: [&str; 3] = ["id", "key", "name"];

/// The smallest epoch value read as milliseconds (the year 1973).
const MIN_EPOCH_MS: i64 = 100_000_000_000;

/// One place where the two sides differ. A side that has nothing there is `None`.
#[derive(Debug, Clone, PartialEq)]
pub struct Change {
    /// The path of the value, `$` for the root.
    pub path: String,
    /// The left value.
    pub left: Option<Value>,
    /// The right value.
    pub right: Option<Value>,
}

struct Walk {
    tolerance_ms: i64,
    out: Vec<Change>,
}

fn is_timestamp_key(key: &str) -> bool {
    matches!(key, "at" | "ts" | "time" | "since") || key.ends_with("At")
}

fn as_ms(value: &Value) -> Option<i64> {
    match value {
        Value::String(text) => parse_ms(text),
        Value::Number(n) => n.as_i64().filter(|ms| *ms >= MIN_EPOCH_MS),
        _ => None,
    }
}

fn same_leaf(left: &Value, right: &Value, key: &str, tolerance_ms: i64) -> bool {
    if left == right {
        return true;
    }
    if let (Value::Number(a), Value::Number(b)) = (left, right) {
        if a.as_f64() == b.as_f64() {
            return true;
        }
    }
    let by_value = left.is_string() && right.is_string();
    if !by_value && !is_timestamp_key(key) {
        return false;
    }
    match (as_ms(left), as_ms(right)) {
        (Some(a), Some(b)) => (a - b).abs() <= tolerance_ms,
        _ => false,
    }
}

fn child(path: &str, key: &str) -> String {
    if path.is_empty() {
        key.to_owned()
    } else {
        format!("{path}.{key}")
    }
}

fn identity_of(items: &[Value]) -> Option<&'static str> {
    IDENTITY.into_iter().find(|field| {
        let names: HashSet<&str> = items
            .iter()
            .filter_map(|row| row.get(*field).and_then(Value::as_str))
            .collect();
        !items.is_empty() && names.len() == items.len()
    })
}

fn label(row: &Value, field: &str) -> String {
    row.get(field)
        .and_then(Value::as_str)
        .unwrap_or_default()
        .to_owned()
}

impl Walk {
    fn put(&mut self, path: &str, left: Option<&Value>, right: Option<&Value>) {
        let path = if path.is_empty() { "$" } else { path };
        self.out.push(Change {
            path: path.to_owned(),
            left: left.cloned(),
            right: right.cloned(),
        });
    }

    fn compare(&mut self, left: &Value, right: &Value, path: &str, key: &str) {
        match (left, right) {
            (Value::Object(a), Value::Object(b)) => self.objects(a, b, path),
            (Value::Array(a), Value::Array(b)) => self.arrays(a, b, path, key),
            _ if same_leaf(left, right, key, self.tolerance_ms) => {}
            _ => self.put(path, Some(left), Some(right)),
        }
    }

    fn objects(&mut self, left: &Map<String, Value>, right: &Map<String, Value>, path: &str) {
        for (k, l) in left {
            match right.get(k) {
                Some(r) => self.compare(l, r, &child(path, k), k),
                None => self.put(&child(path, k), Some(l), None),
            }
        }
        for (k, r) in right.iter().filter(|(k, _)| !left.contains_key(*k)) {
            self.put(&child(path, k), None, Some(r));
        }
    }

    fn arrays(&mut self, left: &[Value], right: &[Value], path: &str, key: &str) {
        let both = identity_of(left).filter(|f| identity_of(right) == Some(*f));
        if let Some(field) = both {
            self.keyed(left, right, path, field);
            return;
        }
        for i in 0..left.len().max(right.len()) {
            let at = format!("{path}[{i}]");
            match (left.get(i), right.get(i)) {
                (Some(l), Some(r)) => self.compare(l, r, &at, key),
                (l, r) => self.put(&at, l, r),
            }
        }
    }

    fn keyed(&mut self, left: &[Value], right: &[Value], path: &str, field: &str) {
        for l in left {
            let name = label(l, field);
            let at = format!("{path}[{name}]");
            match right.iter().find(|r| label(r, field) == name) {
                Some(r) => self.compare(l, r, &at, field),
                None => self.put(&at, Some(l), None),
            }
        }
        for r in right {
            let name = label(r, field);
            if !left.iter().any(|l| label(l, field) == name) {
                self.put(&format!("{path}[{name}]"), None, Some(r));
            }
        }
    }
}

/// Every place where `left` and `right` differ, in the order of the left value.
#[must_use]
pub fn diff(left: &Value, right: &Value, tolerance_ms: i64) -> Vec<Change> {
    let mut walk = Walk {
        tolerance_ms,
        out: Vec::new(),
    };
    walk.compare(left, right, "", "");
    walk.out
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::diff;

    #[test]
    fn equal_values_have_no_change_whatever_the_key_order() {
        let a = json!({ "a": 1, "b": { "c": [1, 2] } });
        let b = json!({ "b": { "c": [1, 2] }, "a": 1.0 });
        assert_eq!(diff(&a, &b, 0).len(), 0);
    }

    #[test]
    fn a_changed_leaf_is_reported_at_its_path() {
        let c = diff(&json!({ "a": { "b": 1 } }), &json!({ "a": { "b": 2 } }), 0);
        assert_eq!(c.len(), 1);
        assert_eq!(c[0].path, "a.b");
        assert_eq!(c[0].left, Some(json!(1)));
        assert_eq!(c[0].right, Some(json!(2)));
    }

    #[test]
    fn a_key_on_one_side_only_is_a_change_with_nothing_on_the_other() {
        let c = diff(&json!({ "a": 1 }), &json!({ "a": 1, "b": 2 }), 0);
        assert_eq!(c.len(), 1);
        assert_eq!((c[0].path.as_str(), c[0].left.is_none()), ("b", true));
    }

    #[test]
    fn rows_with_an_id_are_matched_by_it() {
        let a = json!({ "docs": [{ "id": "x", "v": 1 }, { "id": "y", "v": 2 }] });
        let b = json!({ "docs": [{ "id": "y", "v": 2 }] });
        let c = diff(&a, &b, 0);
        assert_eq!(c.len(), 1);
        assert_eq!(c[0].path, "docs[x]");
    }

    #[test]
    fn rows_without_an_identity_are_matched_by_index() {
        let c = diff(&json!([1, 2, 3]), &json!([1, 9]), 0);
        let paths: Vec<&str> = c.iter().map(|c| c.path.as_str()).collect();
        assert_eq!(paths, ["[1]", "[2]"]);
    }

    #[test]
    fn timestamps_within_the_tolerance_are_equal() {
        let a = json!({ "createdAt": "2026-10-26T10:00:00.000Z", "n": 1_790_000_000_000_i64 });
        let b = json!({ "createdAt": "2026-10-26T10:00:01.500Z", "n": 1_790_000_000_000_i64 });
        assert_eq!(diff(&a, &b, 2000).len(), 0);
        assert_eq!(diff(&a, &b, 1000).len(), 1);
    }

    #[test]
    fn date_strings_are_timestamps_under_any_key() {
        let a = json!({ "lastSeen": "2026-10-26T10:00:00.000Z" });
        let b = json!({ "lastSeen": "2026-10-26T10:00:00.200Z" });
        assert_eq!(diff(&a, &b, 1000).len(), 0);
        assert_eq!(
            diff(&json!({ "t": "abc" }), &json!({ "t": "abd" }), 1000).len(),
            1
        );
    }

    #[test]
    fn epoch_numbers_under_a_timestamp_key_use_the_tolerance() {
        let a = json!({ "at": 1_790_000_000_000_i64 });
        let b = json!({ "at": 1_790_000_000_900_i64 });
        assert_eq!(diff(&a, &b, 1000).len(), 0);
        let other = json!({ "count": 1_790_000_000_900_i64 });
        assert_eq!(
            diff(&json!({ "count": 1_790_000_000_000_i64 }), &other, 1000).len(),
            1
        );
    }

    #[test]
    fn a_different_type_is_a_change() {
        assert_eq!(diff(&json!({ "a": "1" }), &json!({ "a": 1 }), 0).len(), 1);
        assert_eq!(diff(&json!(null), &json!({}), 0)[0].path, "$");
    }
}
