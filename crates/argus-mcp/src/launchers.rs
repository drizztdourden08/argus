//! `list_launchers`: the launchers as the Builds tab shows them.

use std::cmp::Ordering;
use std::collections::HashMap;

use serde_json::{Value, json};

use crate::args::flag;
use crate::backend::McpBackend;
use crate::json::{display, nn, text, truthy};
use argus_core::collate;

fn is_master(l: &Value) -> bool {
    text(l, "name") == Some("master")
}

fn pin(x: &Value, y: &Value) -> Ordering {
    is_master(y)
        .cmp(&is_master(x))
        .then_with(|| truthy(y, "current").cmp(&truthy(x, "current")))
        .then_with(|| collate::compare(&display(x, "name"), &display(y, "name"), true))
}

fn plays(backend: &dyn McpBackend, key: &str) -> HashMap<String, Value> {
    backend
        .docs(key, "launches")
        .iter()
        .filter(|l| truthy(l, "launcher"))
        .filter_map(|l| Some((text(l, "launcher")?.to_owned(), nn(l, "name")?.clone())))
        .collect()
}

fn bundles(builds: &[Value], launcher: &Value) -> Vec<Value> {
    builds
        .iter()
        .filter(|b| b.get("launcher").is_some() && b.get("launcher") == launcher.get("name"))
        .map(|b| {
            nn(b, "tag")
                .or_else(|| nn(b, "id"))
                .cloned()
                .unwrap_or(Value::Null)
        })
        .collect()
}

fn launcher_row(l: &Value, plays: &HashMap<String, Value>, builds: &[Value]) -> Value {
    let play = if truthy(l, "retiredAt") {
        Value::Null
    } else {
        text(l, "name")
            .and_then(|n| plays.get(n))
            .cloned()
            .unwrap_or(Value::Null)
    };
    let mut row = l.clone();
    if let Some(map) = row.as_object_mut() {
        map.insert("play".into(), play);
        map.insert("bundles".into(), Value::Array(bundles(builds, l)));
    }
    row
}

/// The launchers, master first, then the current ones, with their Play run name and bundles.
pub fn list_launchers(backend: &dyn McpBackend, key: &str, args: &Value) -> Value {
    let retired = flag(args, "includeRetired");
    let plays = plays(backend, key);
    let builds = backend.docs(key, "builds");
    let mut rows: Vec<Value> = backend
        .docs(key, "launchers")
        .into_iter()
        .filter(|l| retired || !truthy(l, "retiredAt"))
        .collect();
    rows.sort_by(pin);
    let master = rows.iter().any(|l| is_master(l) && !truthy(l, "retiredAt"));
    let launchers: Vec<Value> = rows
        .iter()
        .map(|l| launcher_row(l, &plays, &builds))
        .collect();
    json!({ "launchers": launchers, "master": master })
}
