//! Test support: checks the registry file against `registry/item-types.schema.json`, the schema
//! the web and editors use. It reads the keywords the schema uses: `$ref`, `type`, `required`,
//! `properties`, `additionalProperties`, `items`, `enum`, `minimum` and `minItems`.

use serde_json::Value;

/// The schema file.
const SCHEMA: &str = include_str!("../../registry/item-types.schema.json");

fn type_matches(name: &str, value: &Value) -> bool {
    match name {
        "string" => value.is_string(),
        "integer" => value.is_i64() || value.is_u64(),
        "boolean" => value.is_boolean(),
        "object" => value.is_object(),
        "array" => value.is_array(),
        _ => false,
    }
}

fn resolve<'a>(root: &'a Value, schema: &'a Value) -> &'a Value {
    match schema["$ref"].as_str().and_then(|r| r.strip_prefix('#')) {
        Some(pointer) => root.pointer(pointer).unwrap_or(schema),
        None => schema,
    }
}

fn check_scalars(schema: &Value, value: &Value, at: &str, out: &mut Vec<String>) {
    if let Some(t) = schema["type"].as_str().filter(|t| !type_matches(t, value)) {
        out.push(format!("{at}: expected {t}"));
    }
    if let Some(options) = schema["enum"].as_array().filter(|o| !o.contains(value)) {
        out.push(format!("{at}: {value} is not one of {options:?}"));
    }
    if let (Some(min), Some(n)) = (schema["minimum"].as_i64(), value.as_i64()) {
        if n < min {
            out.push(format!("{at}: {n} is below {min}"));
        }
    }
}

fn check_object(
    root: &Value,
    schema: &Value,
    map: &serde_json::Map<String, Value>,
    at: &str,
    out: &mut Vec<String>,
) {
    for name in schema["required"]
        .as_array()
        .into_iter()
        .flatten()
        .filter_map(Value::as_str)
    {
        if !map.contains_key(name) {
            out.push(format!("{at}: missing {name}"));
        }
    }
    for (key, child) in map {
        let path = format!("{at}/{key}");
        match (&schema["properties"][key], &schema["additionalProperties"]) {
            (Value::Null, Value::Bool(false)) => out.push(format!("{path}: not allowed")),
            (Value::Null, Value::Object(_)) => {
                walk(root, &schema["additionalProperties"], child, &path, out);
            }
            (Value::Null, _) => {}
            (known, _) => walk(root, known, child, &path, out),
        }
    }
}

fn check_array(root: &Value, schema: &Value, items: &[Value], at: &str, out: &mut Vec<String>) {
    if let Some(min) = schema["minItems"]
        .as_u64()
        .filter(|m| (items.len() as u64) < *m)
    {
        out.push(format!("{at}: fewer than {min} items"));
    }
    if schema["items"].is_object() {
        for (i, item) in items.iter().enumerate() {
            walk(root, &schema["items"], item, &format!("{at}/{i}"), out);
        }
    }
}

fn walk(root: &Value, schema: &Value, value: &Value, at: &str, out: &mut Vec<String>) {
    let schema = resolve(root, schema);
    check_scalars(schema, value, at, out);
    match value {
        Value::Object(map) => check_object(root, schema, map, at, out),
        Value::Array(items) => check_array(root, schema, items, at, out),
        _ => {}
    }
}

/// Every way `doc` breaks the schema, one line each.
fn violations(doc: &Value) -> Vec<String> {
    let schema: Value = serde_json::from_str(SCHEMA).unwrap();
    let mut out = Vec::new();
    walk(&schema, &schema, doc, "", &mut out);
    out
}

// test-tag: critical
#[cfg(test)]
mod tests {
    use serde_json::{Value, json};

    use super::violations;
    use crate::registry::load::SOURCE;

    fn shipped() -> Value {
        serde_json::from_str(SOURCE).unwrap()
    }

    #[test]
    fn the_shipped_file_follows_its_schema() {
        assert_eq!(violations(&shipped()), Vec::<String>::new());
    }

    #[test]
    fn a_missing_key_a_stray_key_and_a_wrong_type_are_all_caught() {
        let mut doc = shipped();
        doc.as_object_mut().unwrap().remove("limits");
        doc["colour"] = json!("red");
        doc["types"][0]["card"]["progress"] = json!("pie");
        doc["lanes"] = json!("todo");
        let found = violations(&doc).join("\n");
        assert!(found.contains(": missing limits"), "{found}");
        assert!(found.contains("/colour: not allowed"), "{found}");
        assert!(
            found.contains("/types/0/card/progress: \"pie\" is not one of"),
            "{found}"
        );
        assert!(found.contains("/lanes: expected array"), "{found}");
    }

    #[test]
    fn the_schema_lists_every_type_collection_and_content_kind_the_code_reads() {
        let schema: Value = serde_json::from_str(super::SCHEMA).unwrap();
        let kinds = &schema["$defs"]["field"]["properties"]["content"]["enum"];
        assert_eq!(kinds.as_array().unwrap().len(), 9);
        let collections = &schema["$defs"]["type"]["properties"]["collection"]["enum"];
        assert_eq!(collections, &json!(["items", "tests"]));
    }
}
