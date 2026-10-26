//! The `search` tool: text search over the docs of one project or of every project.

use std::cmp::Ordering;

use argus_core::js;
use serde_json::{Map, Value, json};

use crate::args::{arg_str, arg_u64};
use crate::backend::McpBackend;
use crate::collate;
use crate::context::CallContext;
use crate::error::ToolError;
use crate::json::{display, nn, truthy};
use crate::project::{clip, require_project};

const COLLS: [&str; 7] = [
    "items", "tests", "entries", "threads", "actions", "agents", "log",
];
const SKIP_KEYS: [&str; 12] = [
    "createdAt",
    "updatedAt",
    "rank",
    "rev",
    "createdBy",
    "updatedBy",
    "session",
    "link",
    "at",
    "ts",
    "url",
    "file",
];

fn is_timestamp(s: &str) -> bool {
    let b = s.as_bytes();
    b.len() >= 11
        && b[..4].iter().all(u8::is_ascii_digit)
        && b[4] == b'-'
        && b[5..7].iter().all(u8::is_ascii_digit)
        && b[7] == b'-'
        && b[8..10].iter().all(u8::is_ascii_digit)
        && b[10] == b'T'
}

fn text_of(v: &Value, out: &mut Vec<String>) {
    match v {
        Value::String(s) if !is_timestamp(s) => out.push(s.clone()),
        Value::Array(list) => list.iter().for_each(|x| text_of(x, out)),
        Value::Object(map) => map
            .iter()
            .filter(|(k, _)| !SKIP_KEYS.contains(&k.as_str()))
            .for_each(|(_, x)| text_of(x, out)),
        _ => {}
    }
}

fn is_search_space(c: char) -> bool {
    (c.is_whitespace() && c != '\u{85}') || c == '\u{feff}'
}

fn title_of(doc: &Value) -> String {
    let found = ["title", "task", "text", "subject", "resource", "id"]
        .iter()
        .find_map(|k| nn(doc, k));
    found.map_or_else(|| "undefined".to_owned(), js::string)
}

fn utf16_slice(s: &str, from: usize, len: usize) -> String {
    let units: Vec<u16> = s.encode_utf16().collect();
    let start = from.min(units.len());
    let end = (from + len).min(units.len());
    String::from_utf16_lossy(&units[start..end])
}

fn snippet_start(low: &str, first: Option<&String>) -> usize {
    let at = first
        .and_then(|t| low.find(t.as_str()))
        .map_or(0, |byte| low[..byte].encode_utf16().count());
    at.saturating_sub(60)
}

fn hit_for(doc: &Value, key: &str, coll: &str, terms: &[String]) -> Option<(usize, Value)> {
    if truthy(doc, "deletedAt") {
        return None;
    }
    let mut parts = Vec::new();
    text_of(doc, &mut parts);
    let text = parts.join(" | ");
    let low = text.to_lowercase();
    if !terms.iter().all(|t| low.contains(t.as_str())) {
        return None;
    }
    let title = title_of(doc);
    let low_title = title.to_lowercase();
    let score = terms
        .iter()
        .map(|t| if low_title.contains(t.as_str()) { 2 } else { 1 })
        .sum();
    let mut hit = Map::new();
    hit.insert("project".into(), json!(key));
    hit.insert("coll".into(), json!(coll));
    if let Some(id) = nn(doc, "id").or_else(|| nn(doc, "subject")) {
        hit.insert("id".into(), id.clone());
    }
    hit.insert("title".into(), json!(js::slice(&title, 120)));
    let start = snippet_start(&low, terms.first());
    hit.insert("snippet".into(), json!(utf16_slice(&text, start, 180)));
    Some((score, Value::Object(hit)))
}

fn hit_order(x: &(usize, Value), y: &(usize, Value)) -> Ordering {
    let cmp = |k: &str| collate::compare(&display(&x.1, k), &display(&y.1, k), false);
    y.0.cmp(&x.0)
        .then_with(|| cmp("project"))
        .then_with(|| cmp("coll"))
        .then_with(|| cmp("id"))
}

fn collections(args: &Value) -> Vec<String> {
    let asked: Vec<String> = args["collections"]
        .as_array()
        .into_iter()
        .flatten()
        .filter_map(|c| c.as_str().map(str::to_owned))
        .collect();
    if asked.is_empty() {
        COLLS.iter().map(|c| (*c).to_owned()).collect()
    } else {
        asked
    }
}

/// `search`: every project unless one is named or the client has a default project.
pub fn search(
    backend: &dyn McpBackend,
    args: &Value,
    ctx: &CallContext,
) -> Result<Value, ToolError> {
    let wanted = args
        .get("project")
        .and_then(Value::as_str)
        .or(ctx.default_project.as_deref())
        .unwrap_or("all");
    let keys = if wanted == "all" {
        backend.project_keys()
    } else {
        vec![require_project(backend, Some(wanted))?]
    };
    let query = arg_str(args, "query").unwrap_or_default().to_lowercase();
    let terms: Vec<String> = query
        .split(is_search_space)
        .filter(|t| !t.is_empty())
        .map(str::to_owned)
        .collect();
    let mut hits = Vec::new();
    for key in &keys {
        for coll in collections(args) {
            for doc in backend.docs(key, &coll) {
                hits.extend(hit_for(&doc, key, &coll, &terms));
            }
        }
    }
    hits.sort_by(hit_order);
    let rows: Vec<Value> = clip(hits, arg_u64(args, "limit"))
        .into_iter()
        .map(|(_, hit)| hit)
        .collect();
    Ok(json!({ "hits": rows }))
}
