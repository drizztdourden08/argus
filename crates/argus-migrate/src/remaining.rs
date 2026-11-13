//! The dry run's review list: after the whitelist rewrite, every string (and object key) that
//! still equals a board key, by JSON path shape. Most are actor names, log channels and thread
//! ids, which stay on purpose; the rest are for the Overlord to read before the cutover.

use std::collections::{BTreeMap, BTreeSet};
use std::path::Path;

use argus_core::ids::PORTFOLIO;
use serde_json::Value;

use crate::error::Result;
use crate::jsonio;
use crate::map::{self, IdMap};
use crate::plan::{Plan, board_file};
use crate::rewrite::{board, portfolio};

/// Path shape to count, per file.
pub type Remaining = BTreeMap<String, BTreeMap<String, usize>>;

fn is_item_id(s: &str) -> bool {
    s.split_once('-').is_some_and(|(head, tail)| {
        (1..=3).contains(&head.len())
            && head.bytes().all(|b| b.is_ascii_uppercase())
            && !tail.is_empty()
            && tail.bytes().all(|b| b.is_ascii_digit())
    })
}

fn shape(path: &[String], keys: &BTreeSet<&str>) -> String {
    let parts: Vec<&str> = path
        .iter()
        .map(|seg| {
            if keys.contains(seg.as_str()) {
                "<key>"
            } else if is_item_id(seg) || seg.bytes().all(|b| b.is_ascii_digit()) {
                "*"
            } else {
                seg.as_str()
            }
        })
        .collect();
    parts.join(".")
}

fn collect(
    v: &Value,
    keys: &BTreeSet<&str>,
    path: &mut Vec<String>,
    out: &mut BTreeMap<String, usize>,
) {
    match v {
        Value::String(s) if keys.contains(s.as_str()) => {
            *out.entry(shape(path, keys)).or_insert(0) += 1;
        }
        Value::Array(items) => {
            for (i, item) in items.iter().enumerate() {
                path.push(i.to_string());
                collect(item, keys, path, out);
                path.pop();
            }
        }
        Value::Object(fields) => {
            for (name, item) in fields {
                path.push(name.clone());
                if keys.contains(name.as_str()) {
                    *out.entry(shape(path, keys)).or_insert(0) += 1;
                }
                collect(item, keys, path, out);
                path.pop();
            }
        }
        _ => {}
    }
}

fn scan_file(
    file: &Path,
    label: String,
    keys: &BTreeSet<&str>,
    rewrite: impl FnOnce(&mut Value),
    out: &mut Remaining,
) -> Result<()> {
    if !file.is_file() {
        return Ok(());
    }
    let mut value = jsonio::read(file)?;
    rewrite(&mut value);
    let mut shapes = BTreeMap::new();
    collect(&value, keys, &mut Vec::new(), &mut shapes);
    if !shapes.is_empty() {
        out.insert(label, shapes);
    }
    Ok(())
}

/// Scans every board and the portfolio, rewritten in memory.
pub fn scan(home: &Path, plan: &Plan) -> Result<Remaining> {
    let ids: IdMap = map::forward(plan);
    let keys: BTreeSet<&str> = plan.boards.keys().map(String::as_str).collect();
    let mut out = Remaining::new();
    for (key, planned) in &plan.boards {
        let folder = if home.join(key).is_dir() { key } else { &planned.id };
        let label = format!("{key}/board.json");
        let file = board_file(home, folder);
        scan_file(&file, label, &keys, |v| {
            board::rewrite(v, &ids);
        }, &mut out)?;
    }
    let file = board_file(home, PORTFOLIO);
    let label = format!("{PORTFOLIO}/board.json");
    scan_file(&file, label, &keys, |v| {
        portfolio::rewrite(v, &ids);
    }, &mut out)?;
    Ok(out)
}
