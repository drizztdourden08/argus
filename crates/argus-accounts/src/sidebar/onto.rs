//! The one-way folder plan: the folders and the filing of the source scope are copied onto the
//! target scope. The target gets the folders it lacks (matched by name), its chats follow the
//! folder the source files them in, and the order inside a folder follows the source. A folder
//! or a chat only the target has is KEPT. The source's own entries are never changed.
//!
//! It runs the two-way plan over just the two scopes with the source made the leader (its chats
//! count as focused last), then drops every step on the source and puts the source's entries
//! back as they were.

use serde_json::{Map, Value};

use super::SidebarKeys;
use super::folders::{FolderPlan, Presence, plan_folders};

/// Plans the folder changes that make `target` follow `source`. Both are scope names
/// (`<account>/<org>`); `presence` says which chats each scope holds, as for [`plan_folders`].
///
/// # Errors
/// When a scope's folder data is not shaped as the app writes it.
pub fn plan_folders_onto(
    keys: &SidebarKeys,
    presence: &Presence,
    (source, target): (&str, &str),
    new_id: &mut dyn FnMut() -> String,
) -> Result<FolderPlan, String> {
    let mut pair = Presence::new();
    for (scope, focus) in [(source, i64::MAX), (target, 0)] {
        if let Some(chats) = presence.get(scope) {
            let led = chats.keys().map(|id| (id.clone(), focus)).collect();
            pair.insert(scope.to_owned(), led);
        }
    }
    let mut plan = plan_folders(keys, &pair, new_id)?;
    plan.steps.retain(|step| step.scope() == target);
    restore_source(&mut plan.keys, keys, source);
    Ok(plan)
}

/// Puts the source's entry of every key back as `before` had it.
fn restore_source(next: &mut SidebarKeys, before: &SidebarKeys, source: &str) {
    put_back(
        &mut next.code_sections,
        &before.code_sections,
        &["value"],
        source,
    );
    put_back(
        &mut next.group_scopes,
        &before.group_scopes,
        &["value"],
        source,
    );
    for part in ["codeSidebarByScope", "customGroupsByScope"] {
        put_back(
            &mut next.dframe_store,
            &before.dframe_store,
            &["state", part],
            source,
        );
    }
}

fn put_back(next: &mut Value, before: &Value, path: &[&str], scope: &str) {
    let original = map_at(before, path).and_then(|m| m.get(scope)).cloned();
    let Some(map) = map_at_mut(next, path) else {
        return;
    };
    match original {
        Some(value) => map.insert(scope.to_owned(), value),
        None => map.remove(scope),
    };
}

fn map_at<'a>(root: &'a Value, path: &[&str]) -> Option<&'a Map<String, Value>> {
    let mut at = root;
    for key in path {
        at = at.get(*key)?;
    }
    at.as_object()
}

fn map_at_mut<'a>(root: &'a mut Value, path: &[&str]) -> Option<&'a mut Map<String, Value>> {
    let mut at = root;
    for key in path {
        at = at.get_mut(*key)?;
    }
    at.as_object_mut()
}

// test-tag: critical
#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    use serde_json::json;

    use super::*;
    use crate::sidebar::FolderStep;

    const A: &str = "acct-a/org-a";
    const B: &str = "acct-b/org-b";

    fn keys() -> SidebarKeys {
        let work = json!({"id": "g-work", "name": "Work", "mode": "code"});
        let mine = json!({"id": "g-mine", "name": "Only here", "mode": "code"});
        let sections = |ids: &[&str]| {
            let list: Vec<Value> = ids
                .iter()
                .map(
                    |id| json!({"id": id, "kind": "manual", "name": id, "order": 0, "members": []}),
                )
                .collect();
            json!({ "sections": list })
        };
        SidebarKeys {
            code_sections: json!({"value": {A: sections(&["g-work"]), B: sections(&["g-mine"])}}),
            group_scopes: json!({"value": {
                A: {"groups": [work], "assignments": {"code:c1": "g-work"},
                    "order": {"g-work": ["code:c1"]}},
                B: {"groups": [mine], "assignments": {}, "order": {}},
            }}),
            dframe_store: json!({"state": {}}),
            encodings: [1, 1, 1],
        }
    }

    fn presence() -> Presence {
        let held = |ids: &[&str]| ids.iter().map(|id| ((*id).to_owned(), 5)).collect();
        BTreeMap::from([(A.to_owned(), held(&["c1"])), (B.to_owned(), held(&["c1"]))])
    }

    #[test]
    fn the_target_follows_the_source_and_keeps_its_own_folder() {
        let before = keys();
        let mut n = 0;
        let mut ids = || {
            n += 1;
            format!("g-new-{n}")
        };
        let plan = plan_folders_onto(&before, &presence(), (A, B), &mut ids).unwrap();
        assert!(plan.steps.iter().all(|s| s.scope() == B));
        assert!(
            plan.steps
                .iter()
                .any(|s| matches!(s, FolderStep::CreateFolder { name, .. } if name == "Work"))
        );
        let target = &plan.keys.group_scopes["value"][B];
        let names: Vec<&str> = target["groups"]
            .as_array()
            .unwrap()
            .iter()
            .filter_map(|g| g["name"].as_str())
            .collect();
        assert!(names.contains(&"Work") && names.contains(&"Only here"));
        assert_eq!(target["assignments"]["code:c1"], "g-new-1");
        assert_eq!(
            plan.keys.group_scopes["value"][A],
            before.group_scopes["value"][A]
        );
        assert_eq!(
            plan.keys.code_sections["value"][A],
            before.code_sections["value"][A]
        );
    }
}
