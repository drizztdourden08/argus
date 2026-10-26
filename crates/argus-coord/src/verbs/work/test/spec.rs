//! Every field a spec file or the flags can carry on a test, checked against the limits
//! (`applySpec` and `specFromOptions`).

use std::fs;
use std::io::ErrorKind;
use std::path::Path;

use argus_board::json::{as_integer, display, set};
use argus_board::limits::{optional_text, require_text, text_list};
use serde_json::{Map, Value, json};

use crate::cli::Val;
use crate::cli::resolve::resolve;
use crate::verbs::work::run::{Res, Run, bad};

/// How one field of a spec is checked and turned into what the test stores.
type Check = fn(&Value, &Value) -> Res<Value>;

const FIELDS: [(&str, Check); 13] = [
    ("title", |v, _| Ok(json!(require_text("title", v)?))),
    ("covers", |v, _| optional("covers", v)),
    ("mode", mode),
    ("run", |v, _| optional("command", v)),
    ("context", |v, _| optional("context", v)),
    ("changed", |v, _| list("changed", v, 8)),
    ("setup", setup),
    ("steps", |v, _| list("step", v, 15)),
    ("pass", |v, _| list("expect", v, 8)),
    ("fail", |v, _| list("expect", v, 8)),
    ("notes", |v, _| list("testnote", v, 5)),
    ("ac", acceptance),
    ("build", |v, _| optional("build", v)),
];

const NOT_A_LIST: &str = "(intermediate value).map is not a function";

fn optional(kind: &str, v: &Value) -> Res<Value> {
    Ok(optional_text(kind, v)?.map_or(Value::Null, Value::from))
}

fn items_of(v: &Value) -> Res<Vec<Value>> {
    match v {
        Value::Null => Ok(Vec::new()),
        Value::Array(items) => Ok(items.clone()),
        _ => Err(bad(NOT_A_LIST)),
    }
}

fn list(kind: &str, v: &Value, max: usize) -> Res<Value> {
    items_of(v)?;
    Ok(json!(text_list(kind, v, max)?))
}

fn mode(v: &Value, _item: &Value) -> Res<Value> {
    match v.as_str().filter(|m| ["auto", "manual"].contains(m)) {
        Some(_) => Ok(v.clone()),
        None => Err(bad("mode must be auto or manual.")),
    }
}

fn setup(v: &Value, _item: &Value) -> Res<Value> {
    let lines: Res<Vec<Value>> = items_of(v)?
        .iter()
        .map(|s| {
            let label = require_text("label", &s["label"])?;
            Ok(json!({"label": label, "text": require_text("setup", &s["text"])?}))
        })
        .collect();
    Ok(Value::Array(lines?))
}

/// The numbers of the criteria a test proves, each checked against the item.
pub(super) fn ac_numbers(item: &Value, v: &Value) -> Res<Vec<i64>> {
    let values = match v {
        Value::Array(values) => values,
        Value::Null => return Err(bad("Cannot read properties of null (reading 'map')")),
        _ => return Err(bad("values.map is not a function")),
    };
    let count = item["ac"].as_array().map_or(0, Vec::len);
    values
        .iter()
        .map(|given| {
            let n = as_integer(Some(given));
            match n.filter(|n| usize::try_from(*n).is_ok_and(|n| (1..=count).contains(&n))) {
                Some(n) => Ok(n),
                None => Err(bad(format!("{} has no AC{}.", display(&item["id"]), display(given)))),
            }
        })
        .collect()
}

fn acceptance(v: &Value, item: &Value) -> Res<Value> {
    Ok(json!(ac_numbers(item, v)?))
}

/// Applies every field the spec carries, in the order of the fields, to the test.
pub(super) fn apply_spec(test: &mut Value, item: &Value, spec: &Map<String, Value>) -> Res<()> {
    for (key, check) in FIELDS {
        if let Some(given) = spec.get(key) {
            set(test, key, check(given, item)?);
        }
    }
    Ok(())
}

fn read_file(path: &Path) -> Res<Map<String, Value>> {
    let text = fs::read_to_string(path).map_err(|e| match e.kind() {
        ErrorKind::NotFound => bad(format!(
            "ENOENT: no such file or directory, open '{}'",
            path.display()
        )),
        _ => bad(e.to_string()),
    })?;
    match serde_json::from_str::<Value>(&text) {
        Ok(Value::Object(spec)) => Ok(spec),
        Ok(_) => Err(bad("the spec file must hold a JSON object.")),
        Err(e) => Err(bad(e.to_string())),
    }
}

fn texts(r: &Run<'_>, name: &str) -> Value {
    Value::Array(r.options.many(name).iter().map(Val::to_json).collect())
}

/// The spec the call carries: the `--file` first, then the flags over it.
pub(super) fn spec_from_options(r: &Run<'_>) -> Res<Map<String, Value>> {
    let mut spec = match r.text_opt("file") {
        Some(file) => read_file(&resolve(&r.inv.cwd, Path::new(file)))?,
        None => Map::new(),
    };
    for key in ["title", "covers", "mode", "run", "context"] {
        if let Some(text) = r.text_opt(key) {
            spec.insert(key.to_owned(), json!(text));
        }
    }
    for (name, key) in [("changed", "changed"), ("note", "notes"), ("ac", "ac")] {
        if !r.options.many(name).is_empty() {
            spec.insert(key.to_owned(), texts(r, name));
        }
    }
    if r.options.json("build") == Some(json!(true)) {
        return Err(bad("--build needs a tag (--build \"\" or --clear-build removes it)."));
    }
    if let Some(tag) = r.text_opt("build") {
        spec.insert("build".into(), json!(tag));
    }
    if r.options.flag("clear-build") {
        spec.insert("build".into(), json!(""));
    }
    Ok(spec)
}
