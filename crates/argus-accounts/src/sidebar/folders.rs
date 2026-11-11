//! Pure folder logic. Given the three sidebar keys and which chats each scope holds, it works
//! out what is missing so every scope shows the same manual folders with the same chats.
//! Folders match by name across scopes (their ids differ). A chat's folder follows the scope
//! where the chat was focused last, and the order inside a folder follows the scope with the
//! newest focus among that folder's chats. System sections (pinned, routines, sessions) are
//! never touched, nothing is deleted, and running the plan on its own output gives no steps.

use std::collections::{BTreeMap, BTreeSet};

use serde_json::{Map, Value, json};

use super::SidebarKeys;

/// The chats each scope holds: scope name, then chat id, to its `lastFocusedAt` in that scope.
pub type Presence = BTreeMap<String, BTreeMap<String, i64>>;

/// One change the plan makes, for the sync log.
#[derive(Debug, Clone, PartialEq)]
pub enum FolderStep {
    /// A manual folder the scope lacked, placed after the folder that precedes it elsewhere.
    CreateFolder {
        scope: String,
        name: String,
        new_id: String,
        /// The section's position in the scope, from 0.
        order: i64,
    },
    /// A chat moved into the folder it has in the scope it was focused in last.
    AssignChat {
        scope: String,
        session_id: String,
        folder_name: String,
    },
    /// The order of the chats inside a folder was rewritten to follow the leading scope.
    ReorderFolder { scope: String, name: String },
    /// The owner moved a chat into a folder, or out of any (`folder_name` none).
    MoveChat {
        scope: String,
        session_id: String,
        folder_name: Option<String>,
    },
    /// The owner renamed a folder.
    RenameFolder {
        scope: String,
        name: String,
        new_name: String,
    },
    /// The owner deleted a folder; its chats are no longer filed.
    DeleteFolder { scope: String, name: String },
}

impl FolderStep {
    /// The scope the step changes.
    #[must_use]
    pub fn scope(&self) -> &str {
        match self {
            Self::CreateFolder { scope, .. }
            | Self::AssignChat { scope, .. }
            | Self::ReorderFolder { scope, .. }
            | Self::MoveChat { scope, .. }
            | Self::RenameFolder { scope, .. }
            | Self::DeleteFolder { scope, .. } => scope,
        }
    }
}

/// The steps and the three keys as they are after them.
#[derive(Debug, Clone, PartialEq)]
pub struct FolderPlan {
    pub steps: Vec<FolderStep>,
    pub keys: SidebarKeys,
}

/// A fresh folder id, `cg-<uuid v4>`.
#[must_use]
pub fn fresh_folder_id() -> String {
    format!("cg-{}", uuid::Uuid::new_v4())
}

/// Plans the folder changes. Only scopes in `presence` that also have an entry in both folder
/// keys take part. `new_id` supplies ids for created folders (tests pass a counter).
///
/// # Errors
/// When a participating scope's folder data is not shaped as the app writes it.
pub fn plan_folders(
    keys: &SidebarKeys,
    presence: &Presence,
    new_id: &mut dyn FnMut() -> String,
) -> Result<FolderPlan, String> {
    let mut next = keys.clone();
    let scopes: Vec<String> = presence
        .keys()
        .filter(|s| has_entry(&next.group_scopes, s) && has_entry(&next.code_sections, s))
        .cloned()
        .collect();
    for scope in &scopes {
        check_shape(&next, scope)?;
    }
    let mut steps = Vec::new();
    create_missing(&mut next, &scopes, new_id, &mut steps)?;
    fill_orders(&mut next, &scopes, presence, &mut steps);
    assign_chats(&mut next, &scopes, presence, &mut steps);
    reorder_folders(&mut next, &scopes, presence, &mut steps);
    mirror_into_store(&mut next, &steps);
    Ok(FolderPlan { steps, keys: next })
}

pub(super) fn has_entry(wrapper: &Value, scope: &str) -> bool {
    wrapper.get("value").and_then(|v| v.get(scope)).is_some()
}

fn check_shape(keys: &SidebarKeys, scope: &str) -> Result<(), String> {
    let groups = entry(&keys.group_scopes, scope).and_then(|e| e.get("groups"));
    if !groups.is_some_and(Value::is_array) {
        return Err(format!("scope {scope}: `groups` is not a list"));
    }
    let sections = entry(&keys.code_sections, scope).and_then(|e| e.get("sections"));
    if !sections.is_some_and(Value::is_array) {
        return Err(format!("scope {scope}: `sections` is not a list"));
    }
    Ok(())
}

pub(super) fn entry<'a>(wrapper: &'a Value, scope: &str) -> Option<&'a Value> {
    wrapper.get("value")?.get(scope)
}

pub(super) fn entry_mut<'a>(wrapper: &'a mut Value, scope: &str) -> Option<&'a mut Value> {
    wrapper.get_mut("value")?.get_mut(scope)
}

pub(super) fn text<'a>(v: &'a Value, field: &str) -> Option<&'a str> {
    v.get(field).and_then(Value::as_str)
}

/// The scope's manual folders as (id, name), in the order of its groups list.
pub(super) fn folder_list(keys: &SidebarKeys, scope: &str) -> Vec<(String, String)> {
    let groups = entry(&keys.group_scopes, scope).and_then(|e| e.get("groups"));
    groups
        .and_then(Value::as_array)
        .map(|a| {
            a.iter()
                .filter_map(|g| Some((text(g, "id")?.to_owned(), text(g, "name")?.to_owned())))
                .collect()
        })
        .unwrap_or_default()
}

pub(super) fn folder_id(keys: &SidebarKeys, scope: &str, name: &str) -> Option<String> {
    folder_list(keys, scope)
        .into_iter()
        .find(|(_, n)| n == name)
        .map(|(id, _)| id)
}

pub(super) fn folder_name(keys: &SidebarKeys, scope: &str, id: &str) -> Option<String> {
    folder_list(keys, scope)
        .into_iter()
        .find(|(i, _)| i == id)
        .map(|(_, n)| n)
}

pub(super) fn chat_key(session_id: &str) -> String {
    format!("code:{session_id}")
}

/// The id of the folder a chat is in, in a scope.
pub(super) fn assigned_id(keys: &SidebarKeys, scope: &str, session_id: &str) -> Option<String> {
    let assignments = entry(&keys.group_scopes, scope)?.get("assignments")?;
    text(assignments, &chat_key(session_id)).map(str::to_owned)
}

pub(super) fn order_list(keys: &SidebarKeys, scope: &str, folder_id: &str) -> Vec<String> {
    entry(&keys.group_scopes, scope)
        .and_then(|e| e.get("order"))
        .and_then(|o| o.get(folder_id))
        .and_then(Value::as_array)
        .map(|a| {
            a.iter()
                .filter_map(|v| v.as_str().map(str::to_owned))
                .collect()
        })
        .unwrap_or_default()
}

pub(super) fn set_order_list(
    keys: &mut SidebarKeys,
    scope: &str,
    folder_id: &str,
    list: &[String],
) {
    let Some(e) = entry_mut(&mut keys.group_scopes, scope).and_then(Value::as_object_mut) else {
        return;
    };
    let order = e.entry("order").or_insert_with(|| json!({}));
    if let Some(map) = order.as_object_mut() {
        map.insert(folder_id.to_owned(), json!(list));
    }
}

// Step 1: folders a scope lacks.

fn create_missing(
    keys: &mut SidebarKeys,
    scopes: &[String],
    new_id: &mut dyn FnMut() -> String,
    steps: &mut Vec<FolderStep>,
) -> Result<(), String> {
    for target in scopes {
        for source in scopes.iter().filter(|s| *s != target) {
            let mut previous: Option<String> = None;
            for (src_id, name) in folder_list(keys, source) {
                if folder_id(keys, target, &name).is_none() {
                    let id = new_id();
                    let order =
                        insert_folder(keys, (source, &src_id), (target, &id), previous.as_deref())?;
                    steps.push(FolderStep::CreateFolder {
                        scope: target.clone(),
                        name: name.clone(),
                        new_id: id,
                        order,
                    });
                }
                previous = Some(name);
            }
        }
    }
    Ok(())
}

/// Adds a copy of the source folder to the target scope, right after the folder named
/// `previous`; returns the `order` of its new section.
fn insert_folder(
    keys: &mut SidebarKeys,
    (source, src_id): (&str, &str),
    (target, id): (&str, &str),
    previous: Option<&str>,
) -> Result<i64, String> {
    let mut group = find_group(keys, source, src_id)
        .ok_or_else(|| format!("scope {source}: folder {src_id} has no group"))?;
    let mut section = find_section(keys, source, src_id)
        .ok_or_else(|| format!("scope {source}: folder {src_id} has no section"))?;
    let prev_id = previous.and_then(|p| folder_id(keys, target, p));
    group["id"] = json!(id);
    section["id"] = json!(id);
    section["members"] = json!([]);
    insert_group(keys, target, group, prev_id.as_deref());
    Ok(insert_section(keys, target, section, prev_id.as_deref()))
}

fn find_group(keys: &SidebarKeys, scope: &str, id: &str) -> Option<Value> {
    let groups = entry(&keys.group_scopes, scope)?
        .get("groups")?
        .as_array()?;
    groups.iter().find(|g| text(g, "id") == Some(id)).cloned()
}

pub(super) fn find_section(keys: &SidebarKeys, scope: &str, id: &str) -> Option<Value> {
    let sections = entry(&keys.code_sections, scope)?
        .get("sections")?
        .as_array()?;
    sections.iter().find(|s| text(s, "id") == Some(id)).cloned()
}

fn insert_group(keys: &mut SidebarKeys, scope: &str, group: Value, prev_id: Option<&str>) {
    let Some(list) = entry_mut(&mut keys.group_scopes, scope)
        .and_then(|e| e.get_mut("groups"))
        .and_then(Value::as_array_mut)
    else {
        return;
    };
    let at = prev_id
        .and_then(|p| list.iter().position(|g| text(g, "id") == Some(p)))
        .map_or(0, |i| i + 1);
    list.insert(at, group);
}

pub(super) fn is_manual(section: &Value) -> bool {
    text(section, "kind") == Some("manual")
}

/// Puts the section after the previous folder's section (before the first manual section when
/// there is none), then numbers every section of the scope 0..n in list order, so `order`
/// stays an integer. Returns the new section's number.
fn insert_section(
    keys: &mut SidebarKeys,
    scope: &str,
    section: Value,
    prev_id: Option<&str>,
) -> i64 {
    let Some(list) = entry_mut(&mut keys.code_sections, scope)
        .and_then(|e| e.get_mut("sections"))
        .and_then(Value::as_array_mut)
    else {
        return 0;
    };
    let at = prev_id
        .and_then(|p| list.iter().position(|s| text(s, "id") == Some(p)))
        .map(|i| i + 1)
        .or_else(|| list.iter().position(is_manual))
        .unwrap_or(list.len());
    list.insert(at, section);
    for (n, s) in (0_i64..).zip(list.iter_mut()) {
        s["order"] = json!(n);
    }
    i64::try_from(at).unwrap_or(0)
}

// Step 1b: a chat assigned to a folder but missing from its order list is appended to it.

fn fill_orders(
    keys: &mut SidebarKeys,
    scopes: &[String],
    presence: &Presence,
    steps: &mut Vec<FolderStep>,
) {
    for scope in scopes {
        let Some(here) = presence.get(scope) else {
            continue;
        };
        let mut missing: BTreeMap<String, Vec<String>> = BTreeMap::new();
        for sid in here.keys() {
            let Some(folder) = assigned_id(keys, scope, sid) else {
                continue;
            };
            let key = chat_key(sid);
            if !order_list(keys, scope, &folder).contains(&key) {
                missing.entry(folder).or_default().push(key);
            }
        }
        for (folder, add) in missing {
            let mut list = order_list(keys, scope, &folder);
            list.extend(add);
            set_order_list(keys, scope, &folder, &list);
            if let Some(name) = folder_name(keys, scope, &folder) {
                steps.push(FolderStep::ReorderFolder {
                    scope: scope.clone(),
                    name,
                });
            }
        }
    }
}

// Step 2: a chat's folder follows the scope where it was focused last.

fn assign_chats(
    keys: &mut SidebarKeys,
    scopes: &[String],
    presence: &Presence,
    steps: &mut Vec<FolderStep>,
) {
    let all_ids: BTreeSet<&String> = scopes
        .iter()
        .filter_map(|s| presence.get(s))
        .flat_map(BTreeMap::keys)
        .collect();
    for sid in all_ids {
        let holders: Vec<&String> = scopes
            .iter()
            .filter(|s| presence.get(*s).is_some_and(|m| m.contains_key(sid)))
            .collect();
        let Some((leader, name)) = leading_folder(keys, &holders, presence, sid) else {
            continue;
        };
        for scope in holders.into_iter().filter(|s| **s != leader) {
            move_chat(keys, scope, sid, &name, steps);
        }
    }
}

/// The scope that focused the chat last (a scope where the chat has a folder wins a tie) and
/// the name of the folder the chat has there. None when that scope keeps the chat unfiled.
fn leading_folder(
    keys: &SidebarKeys,
    holders: &[&String],
    presence: &Presence,
    sid: &str,
) -> Option<(String, String)> {
    let rank = |scope: &str| {
        let focus = presence
            .get(scope)
            .and_then(|m| m.get(sid))
            .copied()
            .unwrap_or(0);
        (focus, assigned_id(keys, scope, sid).is_some())
    };
    let mut best: Option<&String> = None;
    for scope in holders {
        if best.is_none_or(|b| rank(scope) > rank(b)) {
            best = Some(scope);
        }
    }
    let leader = best?;
    let id = assigned_id(keys, leader, sid)?;
    let name = folder_name(keys, leader, &id)?;
    Some((leader.clone(), name))
}

fn move_chat(
    keys: &mut SidebarKeys,
    scope: &str,
    sid: &str,
    name: &str,
    steps: &mut Vec<FolderStep>,
) {
    let Some(new_folder) = folder_id(keys, scope, name) else {
        return;
    };
    let old = assigned_id(keys, scope, sid);
    if old.as_deref() == Some(new_folder.as_str()) {
        return;
    }
    let key = chat_key(sid);
    if let Some(old_id) = old {
        let rest: Vec<String> = order_list(keys, scope, &old_id)
            .into_iter()
            .filter(|k| *k != key)
            .collect();
        set_order_list(keys, scope, &old_id, &rest);
    }
    set_assignment(keys, scope, &key, &new_folder);
    let mut list = order_list(keys, scope, &new_folder);
    list.push(key);
    set_order_list(keys, scope, &new_folder, &list);
    steps.push(FolderStep::AssignChat {
        scope: scope.to_owned(),
        session_id: sid.to_owned(),
        folder_name: name.to_owned(),
    });
}

pub(super) fn set_assignment(keys: &mut SidebarKeys, scope: &str, key: &str, folder_id: &str) {
    let Some(e) = entry_mut(&mut keys.group_scopes, scope).and_then(Value::as_object_mut) else {
        return;
    };
    let map = e.entry("assignments").or_insert_with(|| json!({}));
    if let Some(m) = map.as_object_mut() {
        m.insert(key.to_owned(), json!(folder_id));
    }
}

// Step 3: the order inside a folder follows the scope with the newest focus in it.

fn reorder_folders(
    keys: &mut SidebarKeys,
    scopes: &[String],
    presence: &Presence,
    steps: &mut Vec<FolderStep>,
) {
    let names: BTreeSet<String> = scopes
        .iter()
        .flat_map(|s| folder_list(keys, s))
        .map(|(_, n)| n)
        .collect();
    for name in names {
        let Some(owner) = folder_owner(keys, scopes, presence, &name) else {
            continue;
        };
        let Some(owner_id) = folder_id(keys, &owner, &name) else {
            continue;
        };
        let lead = order_list(keys, &owner, &owner_id);
        for scope in scopes.iter().filter(|s| **s != owner) {
            follow_order(keys, scope, &name, &lead, steps);
        }
    }
}

/// The scope in which the newest-focused chat of the folder was focused.
fn folder_owner(
    keys: &SidebarKeys,
    scopes: &[String],
    presence: &Presence,
    name: &str,
) -> Option<String> {
    let mut best: Option<(i64, &String)> = None;
    for scope in scopes {
        let Some(id) = folder_id(keys, scope, name) else {
            continue;
        };
        let newest = newest_focus(keys, scope, presence, &id);
        if newest.is_some_and(|n| best.is_none_or(|(b, _)| n > b)) {
            best = newest.map(|n| (n, scope));
        }
    }
    best.map(|(_, s)| s.clone())
}

fn newest_focus(
    keys: &SidebarKeys,
    scope: &str,
    presence: &Presence,
    folder_id: &str,
) -> Option<i64> {
    let assignments = entry(&keys.group_scopes, scope)?
        .get("assignments")?
        .as_object()?;
    let focus = presence.get(scope)?;
    assignments
        .iter()
        .filter(|(_, v)| v.as_str() == Some(folder_id))
        .filter_map(|(k, _)| focus.get(k.strip_prefix("code:")?).copied())
        .max()
}

/// Rewrites the scope's order list of the folder so the chats the leader lists come first, in
/// the leader's order; the rest keep their place after them.
fn follow_order(
    keys: &mut SidebarKeys,
    scope: &str,
    name: &str,
    lead: &[String],
    steps: &mut Vec<FolderStep>,
) {
    let Some(id) = folder_id(keys, scope, name) else {
        return;
    };
    let current = order_list(keys, scope, &id);
    let mine: BTreeSet<&String> = current.iter().collect();
    let mut wanted: Vec<String> = lead.iter().filter(|k| mine.contains(k)).cloned().collect();
    let placed: BTreeSet<String> = wanted.iter().cloned().collect();
    wanted.extend(current.iter().filter(|k| !placed.contains(*k)).cloned());
    if wanted != current {
        set_order_list(keys, scope, &id, &wanted);
        steps.push(FolderStep::ReorderFolder {
            scope: scope.to_owned(),
            name: name.to_owned(),
        });
    }
}

// The app's own copies, kept equal for every scope the plan changed: `customGroupsByScope`
// holds the groups, `codeSidebarByScope` the sections with the members of each folder.

fn mirror_into_store(keys: &mut SidebarKeys, steps: &[FolderStep]) {
    let changed: BTreeSet<&str> = steps.iter().map(FolderStep::scope).collect();
    mirror_scopes(keys, &changed);
}

/// Makes the app's copies in `dframe-store` follow the two folder keys for each scope given.
pub(super) fn mirror_scopes(keys: &mut SidebarKeys, changed: &BTreeSet<&str>) {
    for &scope in changed {
        let Some(groups) = entry(&keys.group_scopes, scope).cloned() else {
            continue;
        };
        let sections = entry(&keys.code_sections, scope)
            .and_then(|e| e.get("sections"))
            .and_then(Value::as_array)
            .cloned()
            .unwrap_or_default();
        let Some(state) = keys
            .dframe_store
            .get_mut("state")
            .and_then(Value::as_object_mut)
        else {
            continue;
        };
        mirror_sections(state, scope, &sections, &groups);
        let by_scope = state
            .entry("customGroupsByScope")
            .or_insert_with(|| Value::Object(Map::new()));
        if let Some(map) = by_scope.as_object_mut() {
            let merged = merge_groups(map.get(scope), &groups);
            map.insert(scope.to_owned(), merged);
        }
    }
}

/// Rewrites the scope's `codeSidebarByScope` sections from the code sections: the same list
/// and order, each folder's members following its order list. Other fields of an existing
/// section stay. A scope the app has no entry for is left alone.
fn mirror_sections(
    state: &mut Map<String, Value>,
    scope: &str,
    sections: &[Value],
    groups: &Value,
) {
    let Some(entry) = state
        .get_mut("codeSidebarByScope")
        .and_then(|m| m.get_mut(scope))
        .and_then(Value::as_object_mut)
    else {
        return;
    };
    let old: Vec<Value> = entry
        .get("sections")
        .and_then(Value::as_array)
        .cloned()
        .unwrap_or_default();
    let mut mirrored: Vec<Value> = sections
        .iter()
        .map(|section| {
            let id = text(section, "id").unwrap_or_default();
            let previous = old.iter().find(|o| text(o, "id") == Some(id));
            let mut out = previous.unwrap_or(section).clone();
            out["order"] = section.get("order").cloned().unwrap_or(Value::Null);
            if is_manual(section) {
                out["name"] = section.get("name").cloned().unwrap_or(Value::Null);
                out["members"] = json!(members_of(id, previous, groups));
            }
            out
        })
        .collect();
    for kept in old {
        let id = text(&kept, "id").unwrap_or_default();
        if !sections.iter().any(|s| text(s, "id") == Some(id)) {
            mirrored.push(kept);
        }
    }
    entry.insert("sections".to_owned(), Value::Array(mirrored));
}

/// The app's `customGroupsByScope` entry with the planned groups laid over it: the planned
/// groups come first, then the app's groups the plan does not know; assignments and order
/// lists the app holds stay unless the plan sets them.
fn merge_groups(app: Option<&Value>, planned: &Value) -> Value {
    let Some(app) = app.filter(|a| a.is_object()) else {
        return planned.clone();
    };
    let mut out = app.clone();
    let list = |v: &Value| v.get("groups").and_then(Value::as_array).cloned();
    let mut groups = list(planned).unwrap_or_default();
    for old in list(app).unwrap_or_default() {
        let id = text(&old, "id").unwrap_or_default();
        if !groups.iter().any(|g| text(g, "id") == Some(id)) {
            groups.push(old);
        }
    }
    out["groups"] = Value::Array(groups);
    let mut assignments = app
        .get("assignments")
        .and_then(Value::as_object)
        .cloned()
        .unwrap_or_default();
    for (key, folder) in planned
        .get("assignments")
        .and_then(Value::as_object)
        .into_iter()
        .flatten()
    {
        assignments.insert(key.clone(), folder.clone());
    }
    let mut order = app
        .get("order")
        .and_then(Value::as_object)
        .cloned()
        .unwrap_or_default();
    for (folder, wanted) in planned
        .get("order")
        .and_then(Value::as_object)
        .into_iter()
        .flatten()
    {
        let mut list = wanted.as_array().cloned().unwrap_or_default();
        let extras = order.get(folder).and_then(Value::as_array).cloned();
        for old in extras.into_iter().flatten() {
            let elsewhere = old
                .as_str()
                .and_then(|k| assignments.get(k))
                .and_then(Value::as_str)
                .is_some_and(|f| f != folder);
            if !elsewhere && !list.contains(&old) {
                list.push(old);
            }
        }
        order.insert(folder.clone(), Value::Array(list));
    }
    out["assignments"] = Value::Object(assignments);
    out["order"] = Value::Object(order);
    out
}

/// The folder's order list, then the members the app listed that are neither in it nor
/// assigned to another folder.
fn members_of(id: &str, previous: Option<&Value>, groups: &Value) -> Vec<String> {
    let strings = |v: Option<&Value>| -> Vec<String> {
        v.and_then(Value::as_array)
            .map(|a| {
                a.iter()
                    .filter_map(|m| m.as_str().map(str::to_owned))
                    .collect()
            })
            .unwrap_or_default()
    };
    let mut members = strings(groups.get("order").and_then(|o| o.get(id)));
    let assignments = groups.get("assignments");
    for old in strings(previous.and_then(|p| p.get("members"))) {
        let elsewhere = assignments
            .and_then(|a| text(a, &old))
            .is_some_and(|f| f != id);
        if !elsewhere && !members.contains(&old) {
            members.push(old);
        }
    }
    members
}

// test-tag: critical
#[cfg(test)]
mod tests {
    use super::*;

    const A: &str = "acct-a/org-a";
    const B: &str = "acct-b/org-b";

    fn section(id: &str, kind: &str, name: &str, order: i64) -> Value {
        json!({"id": id, "kind": kind, "v": 2, "name": name, "order": order, "members": []})
    }

    /// folders: (id, name), in order; chats: (session id, folder id or none, in order list).
    fn scope_data(folders: &[(&str, &str)], chats: &[(&str, Option<&str>)]) -> (Value, Value) {
        let mut sections = vec![section("pinned", "pinned", "Pinned", 0)];
        let mut groups = Vec::new();
        for (i, (id, name)) in folders.iter().enumerate() {
            let order = i64::try_from(i).unwrap() + 1;
            sections.push(section(id, "manual", name, order));
            groups.push(json!({"id": id, "name": name, "mode": "code"}));
        }
        sections.push(section("sessions", "sessions", "Sessions", 99));
        let mut assignments = Map::new();
        let mut order: Map<String, Value> = Map::new();
        for (sid, folder) in chats {
            if let Some(f) = folder {
                assignments.insert(format!("code:{sid}"), json!(f));
                order
                    .entry((*f).to_owned())
                    .or_insert_with(|| json!([]))
                    .as_array_mut()
                    .unwrap()
                    .push(json!(format!("code:{sid}")));
            }
        }
        (
            json!({"sections": sections}),
            json!({"groups": groups, "assignments": assignments, "order": order}),
        )
    }

    /// The app's `codeSidebarByScope` entry: the sections with each folder's members filled in.
    fn app_sections(code: &Value, groups: &Value) -> Value {
        let mut sections = code["sections"].clone();
        for s in sections.as_array_mut().unwrap() {
            if is_manual(s) {
                s["members"] = groups["order"][s["id"].as_str().unwrap()].clone();
            }
        }
        json!({"migratedFrom": 1, "sections": sections})
    }

    fn keys(scopes: &[(&str, (Value, Value))]) -> SidebarKeys {
        let mut code = Map::new();
        let mut groups = Map::new();
        let mut custom = Map::new();
        let mut sidebar = Map::new();
        for (scope, (c, g)) in scopes {
            code.insert((*scope).into(), c.clone());
            groups.insert((*scope).into(), g.clone());
            custom.insert((*scope).into(), g.clone());
            sidebar.insert((*scope).into(), app_sections(c, g));
        }
        SidebarKeys {
            code_sections: json!({"value": code, "tabId": "", "timestamp": 1}),
            group_scopes: json!({"value": groups, "tabId": "", "timestamp": 1}),
            dframe_store: json!({"state": {"customGroupsByScope": custom, "codeSidebarByScope": sidebar}, "version": 3}),
            encodings: [1, 1, 1],
        }
    }

    fn presence(a: &[(&str, i64)], b: &[(&str, i64)]) -> Presence {
        let map = |l: &[(&str, i64)]| l.iter().map(|(s, t)| ((*s).to_owned(), *t)).collect();
        BTreeMap::from([(A.to_owned(), map(a)), (B.to_owned(), map(b))])
    }

    fn counter() -> impl FnMut() -> String {
        let mut n = 0;
        move || {
            n += 1;
            format!("cg-new-{n}")
        }
    }

    fn run(k: &SidebarKeys, p: &Presence) -> FolderPlan {
        plan_folders(k, p, &mut counter()).unwrap()
    }

    #[test]
    fn merging_keeps_what_the_app_holds_and_the_plan_does_not_touch() {
        let app = json!({
            "groups": [{"id": "cg-x", "name": "Mine", "mode": "code"}],
            "assignments": {"code:keep": "cg-x", "code:moved": "cg-x"},
            "order": {"cg-x": ["code:keep", "code:moved"], "cg-new": ["code:late"]},
            "extra": 1
        });
        let planned = json!({
            "groups": [{"id": "cg-new", "name": "New", "mode": "code"}],
            "assignments": {"code:moved": "cg-new"},
            "order": {"cg-new": ["code:moved"]}
        });
        let merged = merge_groups(Some(&app), &planned);
        assert_eq!(merged["groups"].as_array().unwrap().len(), 2);
        assert_eq!(merged["groups"][0]["id"], "cg-new");
        assert_eq!(merged["assignments"]["code:keep"], "cg-x");
        assert_eq!(merged["assignments"]["code:moved"], "cg-new");
        assert_eq!(
            merged["order"]["cg-new"],
            json!(["code:moved", "code:late"])
        );
        assert_eq!(merged["order"]["cg-x"], json!(["code:keep", "code:moved"]));
        assert_eq!(merged["extra"], 1);
        assert_eq!(merge_groups(None, &planned), planned);
    }

    #[test]
    fn a_section_only_the_app_has_survives_the_mirror() {
        let mut state = serde_json::from_value::<Map<String, Value>>(json!({
            "codeSidebarByScope": {A: {"sections": [
                {"id": "app-only", "kind": "manual", "order": 7, "members": ["code:z"]}
            ]}}
        }))
        .unwrap();
        let sections = [section("pinned", "pinned", "Pinned", 0)];
        mirror_sections(&mut state, A, &sections, &json!({}));
        let out = state["codeSidebarByScope"][A]["sections"]
            .as_array()
            .unwrap();
        assert_eq!(out.len(), 2);
        assert_eq!(out[1]["id"], "app-only");
    }

    #[test]
    fn a_missing_folder_is_created_after_its_predecessor() {
        let a = scope_data(
            &[("cg-a1", "Alpha"), ("cg-a2", "Beta"), ("cg-a3", "Gamma")],
            &[],
        );
        let b = scope_data(&[("cg-b1", "Alpha"), ("cg-b3", "Gamma")], &[]);
        let k = keys(&[(A, a), (B, b)]);
        let plan = run(&k, &presence(&[], &[]));
        assert_eq!(
            plan.steps,
            vec![FolderStep::CreateFolder {
                scope: B.into(),
                name: "Beta".into(),
                new_id: "cg-new-1".into(),
                order: 2,
            }]
        );
        let names: Vec<String> = folder_list(&plan.keys, B)
            .into_iter()
            .map(|(_, n)| n)
            .collect();
        assert_eq!(names, ["Alpha", "Beta", "Gamma"]);
        let ids: Vec<&str> = plan.keys.code_sections["value"][B]["sections"]
            .as_array()
            .unwrap()
            .iter()
            .map(|s| s["id"].as_str().unwrap())
            .collect();
        assert_eq!(ids, ["pinned", "cg-b1", "cg-new-1", "cg-b3", "sessions"]);
        let orders: Vec<&Value> = plan.keys.code_sections["value"][B]["sections"]
            .as_array()
            .unwrap()
            .iter()
            .map(|s| &s["order"])
            .collect();
        assert_eq!(orders, [0, 1, 2, 3, 4]);
        assert!(orders.iter().all(|o| o.is_i64()));
        let mirrored: Vec<&str> =
            plan.keys.dframe_store["state"]["codeSidebarByScope"][B]["sections"]
                .as_array()
                .unwrap()
                .iter()
                .map(|s| s["id"].as_str().unwrap())
                .collect();
        assert_eq!(mirrored, ids);
    }

    #[test]
    fn chats_missing_from_an_order_list_are_appended() {
        let a = scope_data(&[("cg-a1", "Alpha")], &[("local_1", Some("cg-a1"))]);
        let b = scope_data(&[("cg-b1", "Alpha")], &[("local_1", Some("cg-b1"))]);
        let mut k = keys(&[(A, a), (B, b)]);
        k.group_scopes["value"][B]["assignments"]["code:local_2"] = json!("cg-b1");
        let plan = run(
            &k,
            &presence(&[("local_1", 1)], &[("local_1", 1), ("local_2", 2)]),
        );
        assert_eq!(
            plan.steps,
            vec![FolderStep::ReorderFolder {
                scope: B.into(),
                name: "Alpha".into()
            }]
        );
        let order = &plan.keys.group_scopes["value"][B]["order"]["cg-b1"];
        assert_eq!(order, &json!(["code:local_1", "code:local_2"]));
        let members =
            &plan.keys.dframe_store["state"]["codeSidebarByScope"][B]["sections"][1]["members"];
        assert_eq!(members, order);
        assert!(
            run(
                &plan.keys,
                &presence(&[("local_1", 1)], &[("local_1", 1), ("local_2", 2)])
            )
            .steps
            .is_empty()
        );
    }

    #[test]
    fn a_folder_only_one_side_has_is_created_on_the_other() {
        let a = scope_data(&[("cg-a1", "Only A")], &[]);
        let b = scope_data(&[("cg-b1", "Only B")], &[]);
        let plan = run(&keys(&[(A, a), (B, b)]), &presence(&[], &[]));
        assert_eq!(plan.steps.len(), 2);
        assert!(folder_id(&plan.keys, A, "Only B").is_some());
        assert!(folder_id(&plan.keys, B, "Only A").is_some());
        assert!(folder_id(&plan.keys, A, "Only A").is_some());
    }

    #[test]
    fn the_newest_focused_scope_decides_a_chats_folder() {
        let a = scope_data(
            &[("cg-a1", "Alpha"), ("cg-a2", "Beta")],
            &[("local_1", Some("cg-a1"))],
        );
        let b = scope_data(
            &[("cg-b1", "Alpha"), ("cg-b2", "Beta")],
            &[("local_1", Some("cg-b2"))],
        );
        let k = keys(&[(A, a), (B, b)]);
        let plan = run(&k, &presence(&[("local_1", 100)], &[("local_1", 200)]));
        assert_eq!(
            plan.steps,
            vec![FolderStep::AssignChat {
                scope: A.into(),
                session_id: "local_1".into(),
                folder_name: "Beta".into(),
            }]
        );
        let assignments = &plan.keys.group_scopes["value"][A]["assignments"];
        assert_eq!(assignments["code:local_1"], "cg-a2");
        let order = &plan.keys.group_scopes["value"][A]["order"];
        assert_eq!(order["cg-a1"], json!([]));
        assert_eq!(order["cg-a2"], json!(["code:local_1"]));
        let mirrored = &plan.keys.dframe_store["state"]["customGroupsByScope"][A];
        assert_eq!(mirrored, &plan.keys.group_scopes["value"][A]);
        let untouched = &plan.keys.dframe_store["state"]["customGroupsByScope"][B];
        assert_eq!(untouched, &k.group_scopes["value"][B]);
    }

    #[test]
    fn a_chat_in_one_scope_only_is_left_alone() {
        let a = scope_data(&[("cg-a1", "Alpha")], &[("local_1", Some("cg-a1"))]);
        let b = scope_data(&[("cg-b1", "Alpha")], &[]);
        let plan = run(&keys(&[(A, a), (B, b)]), &presence(&[("local_1", 5)], &[]));
        assert!(plan.steps.is_empty());
    }

    #[test]
    fn an_archived_chat_keeps_its_assignment() {
        let a = scope_data(&[("cg-a1", "Alpha")], &[("local_old", Some("cg-a1"))]);
        let b = scope_data(&[("cg-b1", "Alpha")], &[]);
        let plan = run(
            &keys(&[(A, a), (B, b)]),
            &presence(&[("local_old", 1)], &[("local_old", 1)]),
        );
        assert_eq!(
            plan.keys.group_scopes["value"][A]["assignments"]["code:local_old"],
            "cg-a1"
        );
        assert_eq!(
            plan.keys.group_scopes["value"][B]["assignments"]["code:local_old"],
            "cg-b1"
        );
    }

    #[test]
    fn the_order_in_a_folder_follows_the_leading_scope() {
        let a = scope_data(
            &[("cg-a1", "Alpha")],
            &[("local_1", Some("cg-a1")), ("local_2", Some("cg-a1"))],
        );
        let b = scope_data(
            &[("cg-b1", "Alpha")],
            &[("local_2", Some("cg-b1")), ("local_1", Some("cg-b1"))],
        );
        let k = keys(&[(A, a), (B, b)]);
        let plan = run(
            &k,
            &presence(
                &[("local_1", 10), ("local_2", 20)],
                &[("local_1", 90), ("local_2", 5)],
            ),
        );
        assert_eq!(
            plan.steps,
            vec![FolderStep::ReorderFolder {
                scope: A.into(),
                name: "Alpha".into()
            }]
        );
        let order = &plan.keys.group_scopes["value"][A]["order"]["cg-a1"];
        assert_eq!(order, &json!(["code:local_2", "code:local_1"]));
    }

    #[test]
    fn system_sections_are_untouched() {
        let a = scope_data(&[("cg-a1", "Alpha"), ("cg-a2", "Beta")], &[]);
        let b = scope_data(&[("cg-b1", "Alpha")], &[]);
        let k = keys(&[(A, a), (B, b)]);
        let plan = run(&k, &presence(&[], &[]));
        let system = |keys: &SidebarKeys| -> Vec<Value> {
            keys.code_sections["value"][B]["sections"]
                .as_array()
                .unwrap()
                .iter()
                .filter(|s| !is_manual(s))
                .map(|s| {
                    let mut s = s.clone();
                    s.as_object_mut().unwrap().remove("order");
                    s
                })
                .collect()
        };
        assert_eq!(system(&plan.keys), system(&k));
        assert_eq!(
            plan.keys.code_sections["timestamp"],
            k.code_sections["timestamp"]
        );
    }

    #[test]
    fn the_plan_is_idempotent() {
        let a = scope_data(
            &[("cg-a1", "Alpha"), ("cg-a2", "Beta")],
            &[
                ("local_1", Some("cg-a1")),
                ("local_2", Some("cg-a2")),
                ("local_3", Some("cg-a2")),
            ],
        );
        let b = scope_data(
            &[("cg-b1", "Alpha"), ("cg-b9", "Other")],
            &[
                ("local_1", Some("cg-b9")),
                ("local_3", Some("cg-b1")),
                ("local_2", None),
            ],
        );
        let p = presence(
            &[("local_1", 10), ("local_2", 30), ("local_3", 5)],
            &[("local_1", 50), ("local_2", 20), ("local_3", 40)],
        );
        let first = run(&keys(&[(A, a), (B, b)]), &p);
        assert!(!first.steps.is_empty());
        let second = run(&first.keys, &p);
        assert!(second.steps.is_empty());
        assert_eq!(second.keys, first.keys);
    }

    #[test]
    fn a_scope_missing_from_the_keys_is_skipped() {
        let a = scope_data(&[("cg-a1", "Alpha")], &[]);
        let k = keys(&[(A, a)]);
        let plan = run(&k, &presence(&[], &[]));
        assert!(plan.steps.is_empty());
    }

    #[test]
    fn odd_shapes_are_an_error() {
        let a = scope_data(&[("cg-a1", "Alpha")], &[]);
        let b = scope_data(&[], &[]);
        let mut k = keys(&[(A, a), (B, b)]);
        k.group_scopes["value"][B]["groups"] = json!({});
        assert!(plan_folders(&k, &presence(&[], &[]), &mut counter()).is_err());
    }

    #[test]
    fn fresh_ids_look_like_the_apps() {
        let id = fresh_folder_id();
        assert!(id.starts_with("cg-"));
        assert_eq!(id.len(), 3 + 36);
    }
}
