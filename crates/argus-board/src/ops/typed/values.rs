//! The type's own fields (`fields` on the item): the values asked for, checked by the registry.

use serde_json::{Map, Value};

use super::known::Known;
use crate::content::check;
use crate::error::{BoardError, refuse};
use crate::registry::{ItemType, registry};

/// The field values an edit or a create asks for: the entries of `fields`, and each `fields.<id>`.
pub(crate) fn given(set: &Map<String, Value>) -> Result<Map<String, Value>, BoardError> {
    let mut out = Map::new();
    for (key, v) in set {
        if key == "fields" {
            let Some(all) = v.as_object() else {
                return Err(refuse("fields must be an object of field values."));
            };
            out.extend(all.clone());
        } else if let Some(name) = key.strip_prefix("fields.") {
            out.insert(name.to_owned(), v.clone());
        }
    }
    Ok(out)
}

/// Refuses a field the type does not have.
fn own(ty: &ItemType, key: &str) -> Result<(), BoardError> {
    if ty.fields.iter().any(|f| f == key) {
        return Ok(());
    }
    let hint = if ty.fields.is_empty() {
        format!("a {} has no fields of its own", ty.id)
    } else {
        format!("a {} has the fields {}", ty.id, ty.fields.join(", "))
    };
    Err(refuse(format!("{key} is not a field here: {hint}.")))
}

/// The current values with the asked ones checked and applied. A blank value removes the field.
pub(crate) fn merge(
    known: &Known,
    ty: &ItemType,
    current: &Value,
    given: &Map<String, Value>,
) -> Result<Value, BoardError> {
    let mut out = current.as_object().cloned().unwrap_or_default();
    for (key, v) in given {
        own(ty, key)?;
        let checked = check(key, v, &known.context())?;
        if checked.is_null() {
            out.shift_remove(key);
        } else {
            out.insert(key.clone(), checked);
        }
    }
    Ok(Value::Object(out))
}

/// Refuses when a field the type requires (a bug's severity) has no value.
pub(crate) fn require(ty: &ItemType, fields: &Value) -> Result<(), BoardError> {
    for name in &ty.fields {
        let needed = registry().field(name).is_some_and(|f| f.required);
        if needed && fields[name.as_str()].is_null() {
            return Err(refuse(format!(
                "a {} needs {name}: give it as {name}=... in fields.",
                ty.id
            )));
        }
    }
    Ok(())
}
