//! The input of a hook: one JSON value on stdin, and the folder the hook is about.

use std::io::Read;
use std::path::{Path, PathBuf};

use argus_core::js;
use serde_json::{Map, Value};

use crate::cli::resolve::resolve;

/// The JSON on stdin. An unreadable or empty input counts as `{}`; a `null` is `None`, since
/// Node fails on it and the hook then does nothing.
pub fn read(stdin: &mut dyn Read) -> Option<Value> {
    let mut bytes = Vec::new();
    stdin.read_to_end(&mut bytes).ok();
    let text = String::from_utf8_lossy(&bytes);
    match serde_json::from_str::<Value>(&text) {
        Ok(Value::Null) => None,
        Ok(value) => Some(value),
        Err(_) => Some(Value::Object(Map::new())),
    }
}

/// The folder of the hook: its `cwd` when given, else the working directory of the call.
/// `None` when that folder does not exist (or the `cwd` is not a path).
pub fn folder(input: &Value, base: &Path) -> Option<PathBuf> {
    let dir = match &input["cwd"] {
        v if !js::truthy(v) => base.to_path_buf(),
        Value::String(s) => resolve(base, Path::new(s)),
        _ => return None,
    };
    dir.exists().then_some(dir)
}

/// `clip` of `hook.mjs`: a text trimmed and cut to `max` UTF-16 units; anything else is empty.
pub fn clip(value: &Value, max: usize) -> String {
    value
        .as_str()
        .map(|s| js::slice(js::trim(s), max).to_owned())
        .unwrap_or_default()
}

/// `clip(value, max) || null`: the clipped text, `null` when it is empty.
pub fn clip_or_null(value: &Value, max: usize) -> Value {
    let text = clip(value, max);
    if text.is_empty() {
        Value::Null
    } else {
        Value::String(text)
    }
}
