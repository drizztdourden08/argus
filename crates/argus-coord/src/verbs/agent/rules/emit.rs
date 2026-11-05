//! The records the rules changed, as the JSON `coord` writes them: the keys in the order the Node
//! code creates and fills them, so a new record and a changed one are the same bytes.

use argus_launchers::RunDef;
use argus_launchers::launcher::Launcher;
use serde_json::{Map, Value, json};

fn put(map: &mut Map<String, Value>, key: &str, value: impl Into<Value>) {
    map.insert(key.to_owned(), value.into());
}

fn put_some(map: &mut Map<String, Value>, key: &str, value: Option<&String>) {
    if let Some(v) = value {
        put(map, key, v.as_str());
    }
}

/// A launcher record: the literal of `writeLauncher`, then what the flags add, in the order
/// `window`, `useEvery`, the keep marks, `retiredByBundle`, and the item links (`for`, `master`,
/// `retiredByItem`, `relink`) when they are set. A keep always leaves `keptNote`
/// (`null` without a note).
#[must_use]
pub fn launcher_map(l: &Launcher) -> Map<String, Value> {
    let mut m = Map::new();
    put(&mut m, "name", l.name.as_str());
    put(&mut m, "title", l.title.clone());
    put(&mut m, "cmd", l.cmd.clone());
    put(&mut m, "cwd", l.cwd.clone());
    put(&mut m, "kind", l.kind.as_str());
    put(&mut m, "branch", l.branch.clone());
    put(&mut m, "commit", l.commit.clone());
    put(&mut m, "note", l.note.clone());
    put(&mut m, "current", l.current);
    put(&mut m, "retiredAt", l.retired_at.clone());
    put(&mut m, "retiredBy", l.retired_by.clone());
    put(&mut m, "retireNote", l.retire_note.clone());
    put(&mut m, "createdAt", l.created_at.as_str());
    put(&mut m, "createdBy", l.created_by.clone());
    put(&mut m, "updatedAt", l.updated_at.as_str());
    put(&mut m, "by", l.by.clone());
    put_some(&mut m, "bundle", l.bundle.as_ref());
    if l.window {
        put(&mut m, "window", true);
    }
    put_some(&mut m, "useEvery", l.use_every.as_ref());
    if l.kept_at.is_some() {
        put(&mut m, "keptAt", l.kept_at.clone());
        put(&mut m, "keptBy", l.kept_by.clone());
        put(&mut m, "keptNote", l.kept_note.clone());
        put_some(&mut m, "keptUntil", l.kept_until.as_ref());
    }
    put_some(&mut m, "retiredByBundle", l.retired_by_bundle.as_ref());
    put_some(&mut m, "for", l.for_item.as_ref());
    if l.link.master {
        put(&mut m, "master", true);
    }
    put_some(&mut m, "retiredByItem", l.retired_by_item.as_ref());
    if l.link.relink {
        put(&mut m, "relink", true);
    }
    m
}

/// A Play definition: the literal of `syncLauncherPlay`.
#[must_use]
pub fn def_map(d: &RunDef) -> Map<String, Value> {
    let Value::Object(map) = json!({
        "name": d.name,
        "label": d.label,
        "launcher": d.launcher,
        "cmd": d.cmd,
        "cwd": d.cwd,
        "kind": d.kind.as_str(),
        "env": {},
        "hand": d.hand,
        "visible": d.visible,
        "window": d.window,
        "timeoutSec": d.timeout_sec,
        "parallel": d.parallel,
        "definedBy": d.defined_by,
        "definedAt": d.defined_at,
        "rev": d.rev,
    }) else {
        unreachable!("an object literal")
    };
    map
}
