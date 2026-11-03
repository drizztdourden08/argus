//! `list_launchers`: the launchers as the Builds tab shows them. A row is the launcher record, so
//! it carries `for` (the item the build serves), `master`, `retiredByItem` (the item whose closing
//! retired it) and `relink` (a build waiting for `coord launcher link`) when they are set.

use std::cmp::Ordering;
use std::collections::HashMap;

use serde_json::{Value, json};

use crate::args::flag;
use crate::backend::McpBackend;
use crate::json::{display, nn, text, truthy};
use argus_core::collate;

/// The master build: the launcher carrying the `master` flag, or the one named `master` while no
/// launcher carries it.
fn is_master(l: &Value, flagged: bool) -> bool {
    if flagged {
        truthy(l, "master")
    } else {
        text(l, "name") == Some("master")
    }
}

fn pin(x: &Value, y: &Value, flagged: bool) -> Ordering {
    is_master(y, flagged)
        .cmp(&is_master(x, flagged))
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
    let all = backend.docs(key, "launchers");
    let flagged = all.iter().any(|l| truthy(l, "master"));
    let mut rows: Vec<Value> = all
        .into_iter()
        .filter(|l| retired || !truthy(l, "retiredAt"))
        .collect();
    rows.sort_by(|x, y| pin(x, y, flagged));
    let master = rows
        .iter()
        .any(|l| is_master(l, flagged) && !truthy(l, "retiredAt"));
    let launchers: Vec<Value> = rows
        .iter()
        .map(|l| launcher_row(l, &plays, &builds))
        .collect();
    json!({ "launchers": launchers, "master": master })
}
