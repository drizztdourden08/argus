//! The board as page documents: every doc of the page collections, the portfolio cards and
//! dependencies when present, `meta/board`, and `launches/<name>` from the run definitions.

use argus_core::clock::now_ms;
use argus_core::{Coll, DocChange};
use serde_json::{Map, Value, json};

use super::PAGE_COLLECTIONS;
use crate::Board;
use crate::blocked::causes;
use crate::json::{display, obj, section, set};
use crate::launchers::{bundle_status, is_standalone, joined_bundle, usage_of};
use crate::names::display_name;

/// A field kept as it is, `null` when missing (`doc.x ?? null`).
fn nn(doc: &Value, key: &str) -> Value {
    doc[key].clone()
}

fn public_proof(p: &Value, proof_url: &dyn Fn(&Value) -> String) -> Value {
    let mut out = Map::new();
    for key in ["n", "caption", "option"] {
        if let Some(v) = p.get(key) {
            out.insert(key.to_owned(), v.clone());
        }
    }
    out.insert("at".into(), nn(p, "at"));
    let url = if crate::json::truthy(p, "file") {
        json!(proof_url(p))
    } else {
        nn(p, "url")
    };
    out.insert("url".into(), url);
    Value::Object(out)
}

fn public_test(doc: &Value, proof_url: &dyn Fn(&Value) -> String) -> Value {
    let proofs: Vec<Value> = doc["proofs"]
        .as_array()
        .into_iter()
        .flatten()
        .map(|p| public_proof(p, proof_url))
        .collect();
    let mut out = doc.clone();
    set(&mut out, "proofs", proofs);
    out
}

/// An entry as the page sees it: the pictures of a question or decision get their `url`; an
/// entry without any is passed on as it is.
fn public_entry(doc: &Value, proof_url: &dyn Fn(&Value) -> String) -> Value {
    let Some(images) = doc["images"].as_array() else {
        return doc.clone();
    };
    let shown: Vec<Value> = images.iter().map(|p| public_proof(p, proof_url)).collect();
    let mut out = doc.clone();
    set(&mut out, "images", shown);
    out
}

/// An item as the page sees it: with `blocked`, why it is blocked, computed from the whole board
/// and never kept on disk, and its `images` with the `url` the page loads them from.
fn public_item(
    doc: &Value,
    id: &str,
    board: &Board,
    proof_url: &dyn Fn(&Value) -> String,
) -> Value {
    let mut out = public_entry(doc, proof_url);
    set(&mut out, "blocked", json!(causes(board, id)));
    out
}

/// A bundle record as the page sees it: every field present, null when unset.
fn public_build(doc: &Value, id: &str, board: &Board) -> Value {
    let tag = match &doc["tag"] {
        Value::Null => json!(id),
        t => t.clone(),
    };
    json!({
        "tag": tag, "status": bundle_status(board, &display(&tag), doc), "title": nn(doc, "title"),
        "note": nn(doc, "note"), "launcher": nn(doc, "launcher"), "cmd": nn(doc, "cmd"),
        "cwd": nn(doc, "cwd"), "thread": nn(doc, "thread"), "createdAt": nn(doc, "createdAt"),
        "updatedAt": nn(doc, "updatedAt"), "createdBy": nn(doc, "createdBy"),
        "closedAt": nn(doc, "closedAt"), "closedBy": nn(doc, "closedBy"),
        "closeNote": nn(doc, "closeNote"),
    })
}

/// A launcher as the page sees it: every field present, null or false when unset, with what the
/// launcher is (`joined`, `standalone`) and its usage window.
fn public_launcher(doc: &Value, id: &str, board: &Board) -> Value {
    let mut rec = doc.clone();
    crate::json::default_field(&mut rec, "name", id);
    let usage = usage_of(board, &rec, now_ms());
    let joined = joined_bundle(board, &rec).map_or(Value::Null, Value::String);
    json!({
        "name": rec["name"], "title": nn(doc, "title"), "cmd": nn(doc, "cmd"), "cwd": nn(doc, "cwd"),
        "kind": if doc["kind"].is_null() { json!("app") } else { nn(doc, "kind") },
        "branch": nn(doc, "branch"), "commit": nn(doc, "commit"), "note": nn(doc, "note"),
        "current": doc["current"] == true, "window": doc["window"] == true,
        "bundle": nn(doc, "bundle"), "useEvery": nn(doc, "useEvery"), "keptAt": nn(doc, "keptAt"),
        "keptBy": nn(doc, "keptBy"), "keptUntil": nn(doc, "keptUntil"), "joined": joined,
        "standalone": is_standalone(board, &rec),
        "useEveryMs": usage.as_ref().map_or(Value::Null, |u| json!(u.every_ms)),
        "lastUsedAt": usage.as_ref().map_or(Value::Null, |u| json!(u.last_used_at)),
        "stale": usage.as_ref().is_some_and(|u| u.stale),
        "retiredAt": nn(doc, "retiredAt"), "retiredBy": nn(doc, "retiredBy"),
        "retireNote": nn(doc, "retireNote"), "createdAt": nn(doc, "createdAt"),
        "createdBy": nn(doc, "createdBy"), "updatedAt": nn(doc, "updatedAt"), "by": nn(doc, "by"),
    })
}

fn public_doc(
    coll: Coll,
    (id, doc): (&str, &Value),
    board: &Board,
    proof_url: &dyn Fn(&Value) -> String,
) -> Value {
    match coll {
        Coll::Items => public_item(doc, id, board, proof_url),
        Coll::Tests => public_test(doc, proof_url),
        Coll::Entries => public_entry(doc, proof_url),
        Coll::Builds => public_build(doc, id, board),
        Coll::Launchers => public_launcher(doc, id, board),
        _ => doc.clone(),
    }
}

/// A run definition as the page sees it: everything but the env values (names stay).
fn public_launch(name: &str, def: &Value) -> Value {
    let env: Map<String, Value> = obj(&def["env"])
        .keys()
        .map(|k| (k.clone(), Value::String(String::new())))
        .collect();
    let mut out = def.clone();
    set(&mut out, "name", name);
    set(&mut out, "env", Value::Object(env));
    out
}

fn board_meta(board: &Board) -> Value {
    let counters = if board["counters"].is_null() {
        json!({})
    } else {
        board["counters"].clone()
    };
    json!({
        "project": board["project"],
        "name": display_name(&display(&board["project"]), board["name"].as_str(), None),
        "updatedAt": nn(board, "updatedAt"), "counters": counters,
        "handedBuild": nn(board, "handedBuild"), "handedAt": nn(board, "handedAt"),
    })
}

/// Every page document of a board, as sets. The caller diffs them against what it stored (a hash
/// per doc): what is missing from this list is deleted there. `proof_url` is called only for a
/// proof kept on disk (one with a `file`).
#[must_use]
pub fn export_docs(board: &Board, proof_url: &dyn Fn(&Value) -> String) -> Vec<DocChange> {
    let mut docs = Vec::new();
    for coll in PAGE_COLLECTIONS {
        for (id, doc) in section(board, coll.as_str()) {
            let data = public_doc(coll, (id, doc), board, proof_url);
            docs.push(DocChange::set(coll, id.clone(), data));
        }
    }
    for (id, doc) in section(board, "cards") {
        docs.push(DocChange::set(Coll::Cards, id.clone(), doc.clone()));
    }
    for (id, doc) in section(board, "deps") {
        docs.push(DocChange::set(Coll::Deps, id.clone(), doc.clone()));
    }
    docs.push(DocChange::set(Coll::Meta, "board", board_meta(board)));
    for (name, def) in section(board, "runDefs") {
        docs.push(DocChange::set(
            Coll::Launches,
            name.clone(),
            public_launch(name, def),
        ));
    }
    docs
}
