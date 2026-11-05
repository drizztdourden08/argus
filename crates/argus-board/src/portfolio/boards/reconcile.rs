//! Keeping the board items in step with the project list: every project has exactly one item, an
//! item whose project left the list is archived, and one whose project is back is unarchived with
//! its id kept.

use std::collections::{BTreeMap, BTreeSet};
use std::path::Path;

use argus_core::clock::now_iso;
use serde_json::{Map, Value, json};

use super::COLL;
use crate::Board;
use crate::json::{display, obj, set, truthy, unset};
use crate::portfolio::registry::project_keys;
use crate::store::{create_doc, stamp};

/// The actor recorded for changes the reconcile makes by itself.
const ACTOR: &str = "portfolio";

/// One change the reconcile would make.
enum Step {
    Create(String),
    Archive(String),
    Unarchive(String),
}

/// The item id of each project key, and whether that item is archived.
fn items_by_key(p: &Board) -> BTreeMap<String, (String, bool)> {
    let mut out = BTreeMap::new();
    for (id, item) in obj(&p[COLL]) {
        out.entry(display(&item["key"]))
            .or_insert_with(|| (id.clone(), truthy(item, "archivedAt")));
    }
    out
}

fn plan(home: &Path, p: &Board) -> Vec<Step> {
    let keys = project_keys(home, p);
    let items = items_by_key(p);
    let live: BTreeSet<&String> = keys.iter().collect();
    let mut steps = Vec::new();
    for key in &keys {
        match items.get(key) {
            None => steps.push(Step::Create(key.clone())),
            Some((id, true)) => steps.push(Step::Unarchive(id.clone())),
            Some((_, false)) => {}
        }
    }
    for (key, (id, archived)) in &items {
        if !archived && !live.contains(key) {
            steps.push(Step::Archive(id.clone()));
        }
    }
    steps
}

/// Whether a reconcile would change anything. Reads only, so it can run without the lock.
#[must_use]
pub fn needs_reconcile(home: &Path, p: &Board) -> bool {
    !plan(home, p).is_empty()
}

fn goal_of(p: &Board, key: &str) -> String {
    p["projects"][key]["goal"]
        .as_str()
        .unwrap_or_default()
        .to_owned()
}

fn create(p: &mut Board, key: &str, actor: &str) {
    let fields: Map<String, Value> = Map::from_iter([
        ("key".to_owned(), json!(key)),
        ("goal".to_owned(), json!(goal_of(p, key))),
        ("note".to_owned(), json!("")),
    ]);
    // The id comes from the BD counter, so the collection cannot already hold it.
    let _ = create_doc(p, COLL, "BD", fields, actor);
}

fn archive(p: &mut Board, id: &str, actor: &str) {
    if let Some(item) = p.get_mut(COLL).and_then(|c| c.get_mut(id)) {
        set(item, "archivedAt", now_iso());
        set(item, "archivedBy", actor);
    }
    let _ = stamp(p, COLL, id, actor);
}

fn unarchive(p: &mut Board, id: &str, actor: &str) {
    if let Some(item) = p.get_mut(COLL).and_then(|c| c.get_mut(id)) {
        unset(item, "archivedAt");
        unset(item, "archivedBy");
    }
    let _ = stamp(p, COLL, id, actor);
}

/// Reconciles as `actor`. Returns whether it changed anything.
pub(super) fn reconcile_as(home: &Path, p: &mut Board, actor: &str) -> bool {
    let steps = plan(home, p);
    let changed = !steps.is_empty();
    for step in steps {
        match step {
            Step::Create(key) => create(p, &key, actor),
            Step::Archive(id) => archive(p, &id, actor),
            Step::Unarchive(id) => unarchive(p, &id, actor),
        }
    }
    changed
}

/// Gives every project of the list a board item, archives an item whose project left the list
/// and unarchives one whose project is back, keeping its `BD-n`. Returns whether it changed
/// anything.
pub fn reconcile(home: &Path, p: &mut Board) -> bool {
    reconcile_as(home, p, ACTOR)
}
