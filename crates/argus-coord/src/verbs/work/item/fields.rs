//! The own fields of a typed item on the command line: `--field k=v` and one named flag per
//! field of the registry (`--severity S2`, `--repro "..."`).

use argus_board::registry::{Collection, ContentType, registry};
use serde_json::{Map, Value, json};

use crate::cli::{Options, Val};
use crate::verbs::work::run::{Res, bad};

/// A flag name and the field it sets.
struct Flag {
    name: String,
    field: &'static str,
}

fn kebab(id: &str) -> String {
    let mut out = String::new();
    for c in id.chars() {
        if c.is_ascii_uppercase() {
            out.push('-');
            out.push(c.to_ascii_lowercase());
        } else {
            out.push(c);
        }
    }
    out
}

/// The fields of the item types that a value can be given for, once each in type order.
fn field_ids() -> Vec<&'static str> {
    let mut ids: Vec<&'static str> = Vec::new();
    let items = registry()
        .types
        .iter()
        .filter(|t| t.collection == Collection::Items);
    for field in items.flat_map(|t| t.fields.iter()) {
        let settable = registry()
            .field(field)
            .is_some_and(|f| f.content != ContentType::Records);
        if settable && !ids.contains(&field.as_str()) {
            ids.push(field.as_str());
        }
    }
    ids
}

fn flags() -> Vec<Flag> {
    let mut out = Vec::new();
    for field in field_ids() {
        out.push(Flag {
            name: field.to_owned(),
            field,
        });
        if kebab(field) != field {
            out.push(Flag {
                name: kebab(field),
                field,
            });
        }
    }
    out
}

fn pair(val: &Val) -> Res<(String, String)> {
    let Val::Text(text) = val else {
        return Err(bad("--field takes key=value."));
    };
    text.split_once('=')
        .map(|(k, v)| (k.trim().to_owned(), v.to_owned()))
        .filter(|(k, _)| !k.is_empty())
        .ok_or_else(|| bad(format!("--field takes key=value, not \"{text}\".")))
}

fn named(options: &Options, flag: &Flag) -> Res<Vec<(String, String)>> {
    if options.get(&flag.name).is_none() {
        return Ok(Vec::new());
    }
    let values = options.many(&flag.name);
    if values.is_empty() {
        return Err(bad(format!("--{} needs a value.", flag.name)));
    }
    values
        .iter()
        .map(|v| match v {
            Val::Text(t) => Ok((flag.field.to_owned(), t.clone())),
            Val::Flag => Err(bad(format!("--{} needs a value.", flag.name))),
        })
        .collect()
}

fn is_list(field: &str) -> bool {
    registry()
        .field(field)
        .is_some_and(|f| f.content == ContentType::Refs)
}

fn put(out: &mut Map<String, Value>, (field, text): (String, String)) {
    if !is_list(&field) {
        out.insert(field, json!(text));
        return;
    }
    let mut list: Vec<Value> = out
        .get(&field)
        .and_then(Value::as_array)
        .cloned()
        .unwrap_or_default();
    list.extend(
        text.split(',')
            .map(str::trim)
            .filter(|t| !t.is_empty())
            .map(|t| json!(t)),
    );
    out.insert(field, Value::Array(list));
}

/// The field values the call gives, by field id. A list field takes ids separated by commas.
///
/// # Errors
/// Refused for a `--field` that is not `key=value` and a field flag with no value.
pub(super) fn given(options: &Options) -> Res<Map<String, Value>> {
    let mut found: Vec<(String, String)> = Vec::new();
    for val in options.many("field") {
        found.push(pair(&val)?);
    }
    if options.get("field").is_some() && found.is_empty() {
        return Err(bad("--field takes key=value."));
    }
    for flag in flags() {
        found.extend(named(options, &flag)?);
    }
    let mut out = Map::new();
    for item in found {
        put(&mut out, item);
    }
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    // test-tag: critical
    #[test]
    fn every_settable_field_has_a_flag_and_camel_case_has_a_kebab_twin() {
        let names: Vec<String> = flags().into_iter().map(|f| f.name).collect();
        assert!(names.contains(&"severity".to_owned()));
        assert!(names.contains(&"duplicateOf".to_owned()));
        assert!(names.contains(&"duplicate-of".to_owned()));
        assert!(!names.contains(&"runs".to_owned()));
        assert!(!names.contains(&"pass".to_owned()));
    }
}
