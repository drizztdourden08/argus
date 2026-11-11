//! Folder edits on top of a folder plan, for the sync review: the owner's own moves, renames,
//! deletes and new folders, and undoing parts of the automatic plan the owner skipped. The
//! editor keeps the scanned keys to put a chat back where it was, and tracks the folders it
//! removed and the chats it unfiled, so the app's copies in `dframe-store` lose them too (the
//! plain mirror only ever adds).

use std::collections::{BTreeMap, BTreeSet};

use serde_json::{Map, Value, json};

use super::SidebarKeys;
use super::folders::{
    FolderPlan, FolderStep, assigned_id, chat_key, entry, entry_mut, find_section, folder_id,
    folder_list, folder_name, has_entry, is_manual, mirror_scopes, order_list, set_assignment,
    set_order_list, text,
};

/// The keys being edited, with what the editor removed and changed.
pub struct FolderEditor {
    keys: SidebarKeys,
    scanned: SidebarKeys,
    steps: Vec<FolderStep>,
    removed: BTreeMap<String, BTreeSet<String>>,
    unfiled: BTreeMap<String, BTreeSet<String>>,
    touched: BTreeSet<String>,
}

impl FolderEditor {
    /// An editor over `keys` (usually the keys after the automatic plan); `scanned` is what the
    /// store held when the scan ran.
    #[must_use]
    pub fn new(keys: SidebarKeys, scanned: SidebarKeys) -> Self {
        FolderEditor {
            keys,
            scanned,
            steps: Vec::new(),
            removed: BTreeMap::new(),
            unfiled: BTreeMap::new(),
            touched: BTreeSet::new(),
        }
    }

    /// Whether the scope has folder data in both keys (the app made it a sidebar).
    #[must_use]
    pub fn has_scope(&self, scope: &str) -> bool {
        has_entry(&self.keys.group_scopes, scope) && has_entry(&self.keys.code_sections, scope)
    }

    /// Whether the scope has a folder of that name.
    #[must_use]
    pub fn has_folder(&self, scope: &str, name: &str) -> bool {
        folder_id(&self.keys, scope, name).is_some()
    }

    /// The name of the folder the chat is in, in the scope.
    #[must_use]
    pub fn folder_of(&self, scope: &str, session_id: &str) -> Option<String> {
        chat_folder(&self.keys, scope, session_id)
    }

    /// Files the chat in the folder `name`, or in none; a step is added when it moved.
    ///
    /// # Errors
    /// When the scope has no folder of that name.
    pub fn place(
        &mut self,
        scope: &str,
        session_id: &str,
        name: Option<&str>,
    ) -> Result<(), String> {
        let target = match name {
            Some(n) => Some(
                folder_id(&self.keys, scope, n)
                    .ok_or_else(|| format!("scope {scope} has no folder {n}"))?,
            ),
            None => None,
        };
        if self.file(scope, session_id, target.as_deref(), None) {
            self.steps.push(FolderStep::MoveChat {
                scope: scope.to_owned(),
                session_id: session_id.to_owned(),
                folder_name: name.map(str::to_owned),
            });
        }
        Ok(())
    }

    /// Takes a deleted chat out of its folder, without a step of its own.
    pub fn forget(&mut self, scope: &str, session_id: &str) {
        self.file(scope, session_id, None, None);
    }

    /// Puts the chat back in the folder it had when the scan ran, at its old place in that
    /// folder's order; unfiled when it had none or that folder is gone.
    pub fn revert(&mut self, scope: &str, session_id: &str) {
        let was = assigned_id(&self.scanned, scope, session_id)
            .filter(|id| folder_name(&self.keys, scope, id).is_some());
        let at = was.as_ref().and_then(|id| {
            let key = chat_key(session_id);
            order_list(&self.scanned, scope, id)
                .iter()
                .position(|k| *k == key)
        });
        self.file(scope, session_id, was.as_deref(), at);
    }

    /// Moves the chat's assignment and order entry; true when its folder changed.
    fn file(
        &mut self,
        scope: &str,
        session_id: &str,
        target: Option<&str>,
        at: Option<usize>,
    ) -> bool {
        let current = assigned_id(&self.keys, scope, session_id);
        if current.as_deref() == target {
            return false;
        }
        let key = chat_key(session_id);
        if let Some(old) = &current {
            let rest: Vec<String> = order_list(&self.keys, scope, old)
                .into_iter()
                .filter(|k| *k != key)
                .collect();
            set_order_list(&mut self.keys, scope, old, &rest);
        }
        if let Some(folder) = target {
            set_assignment(&mut self.keys, scope, &key, folder);
            let mut list = order_list(&self.keys, scope, folder);
            if !list.contains(&key) {
                list.insert(at.unwrap_or(list.len()).min(list.len()), key);
            }
            set_order_list(&mut self.keys, scope, folder, &list);
        } else {
            remove_assignment(&mut self.keys, scope, &key);
            self.unfiled
                .entry(scope.to_owned())
                .or_default()
                .insert(key);
        }
        self.touched.insert(scope.to_owned());
        true
    }

    /// Adds a manual folder at the end of the scope's folders, with the integer `order` after
    /// the last section. Its section copies the look of another manual folder of the scope.
    ///
    /// # Errors
    /// When the scope already has a folder of that name.
    pub fn create(&mut self, scope: &str, name: &str, id: &str) -> Result<(), String> {
        if self.has_folder(scope, name) {
            return Err(format!("a folder named {name} already exists"));
        }
        let template = folder_list(&self.keys, scope)
            .first()
            .and_then(|(fid, _)| find_section(&self.keys, scope, fid));
        let Some(sections) = entry_mut(&mut self.keys.code_sections, scope)
            .and_then(|e| e.get_mut("sections"))
            .and_then(Value::as_array_mut)
        else {
            return Err(format!("scope {scope} has no sidebar sections"));
        };
        let order = sections
            .iter()
            .filter_map(|s| s.get("order").and_then(Value::as_i64))
            .max()
            .map_or(0, |m| m + 1);
        let mut section =
            template.unwrap_or_else(|| json!({"kind": "manual", "v": 2, "collapsed": false}));
        section["id"] = json!(id);
        section["name"] = json!(name);
        section["order"] = json!(order);
        section["members"] = json!([]);
        sections.push(section);
        if let Some(groups) = entry_mut(&mut self.keys.group_scopes, scope)
            .and_then(|e| e.get_mut("groups"))
            .and_then(Value::as_array_mut)
        {
            groups.push(json!({"id": id, "name": name, "mode": "code"}));
        }
        self.touched.insert(scope.to_owned());
        self.steps.push(FolderStep::CreateFolder {
            scope: scope.to_owned(),
            name: name.to_owned(),
            new_id: id.to_owned(),
            order,
        });
        Ok(())
    }

    /// Renames a folder in its group and its section.
    ///
    /// # Errors
    /// When the folder is unknown or the new name is taken.
    pub fn rename(&mut self, scope: &str, name: &str, new_name: &str) -> Result<(), String> {
        let id =
            folder_id(&self.keys, scope, name).ok_or_else(|| format!("unknown folder {name}"))?;
        if self.has_folder(scope, new_name) {
            return Err(format!("a folder named {new_name} already exists"));
        }
        for list in [
            groups_mut(&mut self.keys.group_scopes, scope, "groups"),
            groups_mut(&mut self.keys.code_sections, scope, "sections"),
        ]
        .into_iter()
        .flatten()
        {
            for item in list
                .iter_mut()
                .filter(|i| text(i, "id") == Some(id.as_str()))
            {
                item["name"] = json!(new_name);
            }
        }
        self.touched.insert(scope.to_owned());
        self.steps.push(FolderStep::RenameFolder {
            scope: scope.to_owned(),
            name: name.to_owned(),
            new_name: new_name.to_owned(),
        });
        Ok(())
    }

    /// Deletes a folder: its group, its section and its order list go, its chats are unfiled.
    ///
    /// # Errors
    /// When the folder is unknown.
    pub fn delete(&mut self, scope: &str, name: &str) -> Result<(), String> {
        let id =
            folder_id(&self.keys, scope, name).ok_or_else(|| format!("unknown folder {name}"))?;
        self.remove(scope, &id);
        self.steps.push(FolderStep::DeleteFolder {
            scope: scope.to_owned(),
            name: name.to_owned(),
        });
        Ok(())
    }

    /// Takes back a folder the automatic plan created: the folder goes and the chats it filed
    /// there go back where they were.
    pub fn drop_created(&mut self, scope: &str, id: &str) {
        let filed: Vec<String> = assigned_to(&self.keys, scope, id);
        self.remove(scope, id);
        for key in filed {
            if let Some(session_id) = key.strip_prefix("code:") {
                self.revert(scope, session_id);
            }
        }
    }

    fn remove(&mut self, scope: &str, id: &str) {
        for key in assigned_to(&self.keys, scope, id) {
            remove_assignment(&mut self.keys, scope, &key);
            self.unfiled
                .entry(scope.to_owned())
                .or_default()
                .insert(key);
        }
        if let Some(list) = groups_mut(&mut self.keys.group_scopes, scope, "groups") {
            list.retain(|g| text(g, "id") != Some(id));
        }
        if let Some(list) = groups_mut(&mut self.keys.code_sections, scope, "sections") {
            list.retain(|s| text(s, "id") != Some(id));
        }
        if let Some(order) = entry_mut(&mut self.keys.group_scopes, scope)
            .and_then(|e| e.get_mut("order"))
            .and_then(Value::as_object_mut)
        {
            order.remove(id);
        }
        self.removed
            .entry(scope.to_owned())
            .or_default()
            .insert(id.to_owned());
        self.touched.insert(scope.to_owned());
    }

    /// The plan: the automatic steps that still hold, then the editor's own, and the keys with
    /// the app's copies following them. With no step left the keys are the scanned ones, so
    /// nothing is written.
    #[must_use]
    pub fn finish(mut self, auto: &[FolderStep]) -> FolderPlan {
        let mut steps: Vec<FolderStep> = auto
            .iter()
            .filter(|s| self.still_holds(s))
            .cloned()
            .collect();
        steps.append(&mut self.steps);
        if steps.is_empty() {
            return FolderPlan {
                steps,
                keys: self.scanned,
            };
        }
        let touched: BTreeSet<&str> = self.touched.iter().map(String::as_str).collect();
        mirror_scopes(&mut self.keys, &touched);
        for scope in self.touched.clone() {
            self.clean_store(&scope);
        }
        FolderPlan {
            steps,
            keys: self.keys,
        }
    }

    fn still_holds(&self, step: &FolderStep) -> bool {
        match step {
            FolderStep::CreateFolder { scope, new_id, .. } => {
                folder_name(&self.keys, scope, new_id).is_some()
            }
            FolderStep::AssignChat {
                scope,
                session_id,
                folder_name,
            } => self.folder_of(scope, session_id).as_ref() == Some(folder_name),
            FolderStep::ReorderFolder { scope, name } => self.has_folder(scope, name),
            _ => true,
        }
    }

    /// Takes the removed folders and the unfiled chats out of the app's copies of the scope.
    fn clean_store(&mut self, scope: &str) {
        let removed = self.removed.get(scope).cloned().unwrap_or_default();
        let unfiled = self.unfiled.get(scope).cloned().unwrap_or_default();
        let assigned: BTreeMap<String, Option<String>> = unfiled
            .iter()
            .map(|k| (k.clone(), assigned_key(&self.keys, scope, k)))
            .collect();
        // A key may stay listed only under the folder it is assigned to now.
        let stays = |key: &str, folder: &str| {
            assigned
                .get(key)
                .is_none_or(|now| now.as_deref() == Some(folder))
        };
        let Some(state) = self
            .keys
            .dframe_store
            .get_mut("state")
            .and_then(Value::as_object_mut)
        else {
            return;
        };
        if let Some(groups) = state
            .get_mut("customGroupsByScope")
            .and_then(|m| m.get_mut(scope))
            .and_then(Value::as_object_mut)
        {
            clean_groups(groups, &removed, &stays);
        }
        if let Some(sections) = state
            .get_mut("codeSidebarByScope")
            .and_then(|m| m.get_mut(scope))
            .and_then(|e| e.get_mut("sections"))
            .and_then(Value::as_array_mut)
        {
            sections.retain(|s| text(s, "id").is_none_or(|id| !removed.contains(id)));
            for section in sections.iter_mut().filter(|s| is_manual(s)) {
                let id = text(section, "id").unwrap_or_default().to_owned();
                if let Some(members) = section.get_mut("members").and_then(Value::as_array_mut) {
                    members.retain(|m| m.as_str().is_none_or(|k| stays(k, &id)));
                }
            }
        }
    }
}

/// The name of the folder a chat is in, in a scope of the keys.
#[must_use]
pub fn chat_folder(keys: &SidebarKeys, scope: &str, session_id: &str) -> Option<String> {
    let id = assigned_id(keys, scope, session_id)?;
    folder_name(keys, scope, &id)
}

fn clean_groups(
    groups: &mut Map<String, Value>,
    removed: &BTreeSet<String>,
    stays: &dyn Fn(&str, &str) -> bool,
) {
    if let Some(list) = groups.get_mut("groups").and_then(Value::as_array_mut) {
        list.retain(|g| text(g, "id").is_none_or(|id| !removed.contains(id)));
    }
    if let Some(map) = groups.get_mut("assignments").and_then(Value::as_object_mut) {
        map.retain(|key, folder| {
            let folder = folder.as_str().unwrap_or_default();
            !removed.contains(folder) && stays(key, folder)
        });
    }
    if let Some(order) = groups.get_mut("order").and_then(Value::as_object_mut) {
        order.retain(|id, _| !removed.contains(id));
        for (id, list) in order.iter_mut() {
            if let Some(items) = list.as_array_mut() {
                items.retain(|k| k.as_str().is_none_or(|k| stays(k, id)));
            }
        }
    }
}

fn groups_mut<'a>(wrapper: &'a mut Value, scope: &str, field: &str) -> Option<&'a mut Vec<Value>> {
    entry_mut(wrapper, scope)?.get_mut(field)?.as_array_mut()
}

fn assigned_key(keys: &SidebarKeys, scope: &str, key: &str) -> Option<String> {
    entry(&keys.group_scopes, scope)?
        .get("assignments")?
        .get(key)?
        .as_str()
        .map(str::to_owned)
}

/// The chat keys assigned to a folder.
fn assigned_to(keys: &SidebarKeys, scope: &str, id: &str) -> Vec<String> {
    entry(&keys.group_scopes, scope)
        .and_then(|e| e.get("assignments"))
        .and_then(Value::as_object)
        .map(|m| {
            m.iter()
                .filter(|(_, f)| f.as_str() == Some(id))
                .map(|(k, _)| k.clone())
                .collect()
        })
        .unwrap_or_default()
}

fn remove_assignment(keys: &mut SidebarKeys, scope: &str, key: &str) {
    if let Some(map) = entry_mut(&mut keys.group_scopes, scope)
        .and_then(|e| e.get_mut("assignments"))
        .and_then(Value::as_object_mut)
    {
        map.remove(key);
    }
}

// test-tag: critical
#[cfg(test)]
mod tests {
    use super::*;

    const S: &str = "acct-a/org-a";

    fn keys() -> SidebarKeys {
        let sections = json!([
            {"id": "pinned", "kind": "pinned", "order": 0, "members": []},
            {"id": "cg-1", "kind": "manual", "v": 2, "name": "One", "order": 1, "sortBy": "recent",
             "members": ["code:local_1", "code:local_2"]},
            {"id": "cg-2", "kind": "manual", "v": 2, "name": "Two", "order": 2, "members": []},
            {"id": "sessions", "kind": "sessions", "order": 3, "members": []},
        ]);
        let groups = json!({
            "groups": [{"id": "cg-1", "name": "One", "mode": "code"},
                       {"id": "cg-2", "name": "Two", "mode": "code"}],
            "assignments": {"code:local_1": "cg-1", "code:local_2": "cg-1"},
            "order": {"cg-1": ["code:local_1", "code:local_2"], "cg-2": []},
        });
        SidebarKeys {
            code_sections: json!({"value": {S: {"sections": sections.clone()}}, "tabId": "", "timestamp": 1}),
            group_scopes: json!({"value": {S: groups.clone()}, "tabId": "", "timestamp": 1}),
            dframe_store: json!({"state": {
                "customGroupsByScope": {S: groups},
                "codeSidebarByScope": {S: {"sections": sections}},
            }, "version": 3}),
            encodings: [1, 1, 1],
        }
    }

    fn editor() -> FolderEditor {
        FolderEditor::new(keys(), keys())
    }

    fn store_groups(plan: &FolderPlan) -> &Value {
        &plan.keys.dframe_store["state"]["customGroupsByScope"][S]
    }

    fn store_sections(plan: &FolderPlan) -> &Vec<Value> {
        plan.keys.dframe_store["state"]["codeSidebarByScope"][S]["sections"]
            .as_array()
            .unwrap()
    }

    #[test]
    fn a_moved_chat_leaves_its_old_folder_everywhere() {
        let mut ed = editor();
        ed.place(S, "local_1", Some("Two")).unwrap();
        let plan = ed.finish(&[]);
        let groups = &plan.keys.group_scopes["value"][S];
        assert_eq!(groups["assignments"]["code:local_1"], "cg-2");
        assert_eq!(groups["order"]["cg-1"], json!(["code:local_2"]));
        assert_eq!(groups["order"]["cg-2"], json!(["code:local_1"]));
        let one = store_sections(&plan)
            .iter()
            .find(|s| s["id"] == "cg-1")
            .unwrap();
        assert_eq!(one["members"], json!(["code:local_2"]));
        assert!(
            matches!(&plan.steps[0], FolderStep::MoveChat { folder_name: Some(n), .. } if n == "Two")
        );
    }

    #[test]
    fn an_unfiled_chat_is_gone_from_the_apps_copies_too() {
        let mut ed = editor();
        ed.place(S, "local_1", None).unwrap();
        let plan = ed.finish(&[]);
        assert!(
            plan.keys.group_scopes["value"][S]["assignments"]
                .get("code:local_1")
                .is_none()
        );
        assert!(
            store_groups(&plan)["assignments"]
                .get("code:local_1")
                .is_none()
        );
        assert_eq!(
            store_groups(&plan)["order"]["cg-1"],
            json!(["code:local_2"])
        );
        let one = store_sections(&plan)
            .iter()
            .find(|s| s["id"] == "cg-1")
            .unwrap();
        assert_eq!(one["members"], json!(["code:local_2"]));
    }

    #[test]
    fn a_deleted_folder_unfiles_its_chats_and_leaves_every_copy() {
        let mut ed = editor();
        ed.delete(S, "One").unwrap();
        let plan = ed.finish(&[]);
        let groups = &plan.keys.group_scopes["value"][S];
        assert_eq!(groups["groups"].as_array().unwrap().len(), 1);
        assert_eq!(groups["assignments"], json!({}));
        assert!(groups["order"].get("cg-1").is_none());
        let sections = plan.keys.code_sections["value"][S]["sections"]
            .as_array()
            .unwrap();
        assert!(sections.iter().all(|s| s["id"] != "cg-1"));
        assert!(
            store_groups(&plan)["groups"]
                .as_array()
                .unwrap()
                .iter()
                .all(|g| g["id"] != "cg-1")
        );
        assert_eq!(store_groups(&plan)["assignments"], json!({}));
        assert!(store_sections(&plan).iter().all(|s| s["id"] != "cg-1"));
        assert!(matches!(&plan.steps[0], FolderStep::DeleteFolder { name, .. } if name == "One"));
    }

    #[test]
    fn a_renamed_folder_keeps_its_id_and_chats() {
        let mut ed = editor();
        ed.rename(S, "One", "First").unwrap();
        assert!(ed.rename(S, "First", "Two").is_err(), "the name is taken");
        assert!(ed.rename(S, "Nope", "X").is_err());
        let plan = ed.finish(&[]);
        assert_eq!(
            plan.keys.group_scopes["value"][S]["groups"][0]["name"],
            "First"
        );
        assert_eq!(
            plan.keys.code_sections["value"][S]["sections"][1]["name"],
            "First"
        );
        assert_eq!(store_sections(&plan)[1]["name"], "First");
        assert_eq!(
            plan.keys.group_scopes["value"][S]["assignments"]["code:local_1"],
            "cg-1"
        );
    }

    #[test]
    fn a_new_folder_goes_last_with_an_integer_order() {
        let mut ed = editor();
        ed.create(S, "Three", "cg-new").unwrap();
        assert!(ed.create(S, "One", "cg-x").is_err());
        let plan = ed.finish(&[]);
        let sections = plan.keys.code_sections["value"][S]["sections"]
            .as_array()
            .unwrap();
        let last = sections.last().unwrap();
        assert_eq!(last["id"], "cg-new");
        assert_eq!(last["order"], 4);
        assert_eq!(last["kind"], "manual");
        assert_eq!(last["sortBy"], "recent", "the look of the scope's folders");
        assert_eq!(last["members"], json!([]));
        let groups = plan.keys.group_scopes["value"][S]["groups"]
            .as_array()
            .unwrap();
        assert_eq!(groups.last().unwrap()["name"], "Three");
        assert!(
            store_groups(&plan)["groups"]
                .as_array()
                .unwrap()
                .iter()
                .any(|g| g["id"] == "cg-new")
        );
    }

    #[test]
    fn revert_puts_a_chat_back_at_its_old_place() {
        let mut moved = keys();
        let groups = &mut moved.group_scopes["value"][S];
        groups["assignments"]["code:local_1"] = json!("cg-2");
        groups["order"]["cg-1"] = json!(["code:local_2"]);
        groups["order"]["cg-2"] = json!(["code:local_1"]);
        let mut ed = FolderEditor::new(moved, keys());
        ed.revert(S, "local_1");
        assert_eq!(
            ed.keys.group_scopes["value"][S],
            keys().group_scopes["value"][S]
        );
    }

    #[test]
    fn nothing_left_to_do_writes_nothing() {
        let mut ed = editor();
        ed.place(S, "local_1", Some("One")).unwrap();
        let plan = ed.finish(&[]);
        assert!(plan.steps.is_empty());
        assert_eq!(plan.keys, keys());
    }
}
