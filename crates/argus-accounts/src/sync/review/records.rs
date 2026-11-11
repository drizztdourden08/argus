//! The chat records every scope should hold after a reviewed sync: the automatic plan's records
//! first, then the owner's choices and edits laid over them. The catalog changes are the
//! difference between these records and the scan, so with no choice the changes are exactly the
//! automatic plan's.

use std::collections::{BTreeMap, BTreeSet};

use serde_json::{Value, json};

use crate::catalog::merge::{LINEAGE_KEYS, advance_history, archive_vote, set_archived};
use crate::catalog::plan::{Change, ChangeKind, Plan};
use crate::ports::Catalog;
use crate::sync::job::{Scan, parent_of};

const INDEX_FILE: &str = "archived-sessions.idx";
/// The lineage keys that make a chat a sub-thread of another.
const LINK_KEYS: [&str; 3] = ["spawnedFrom", "dispatchParentId", "dispatchParentOrigin"];

/// The wanted records of every scope, in the order of the scan's scopes.
pub struct Records<'a> {
    scan: &'a Scan,
    want: Vec<BTreeMap<String, Value>>,
    deleted: Vec<BTreeSet<String>>,
}

impl<'a> Records<'a> {
    /// The records once the automatic plan is written.
    #[must_use]
    pub fn from_auto(scan: &'a Scan, plan: &Plan) -> Self {
        let mut want: Vec<BTreeMap<String, Value>> = scan
            .scopes
            .iter()
            .map(|s| {
                s.chats
                    .iter()
                    .map(|(id, c)| (id.clone(), c.record.clone()))
                    .collect()
            })
            .collect();
        for change in plan
            .changes
            .iter()
            .filter(|c| matches!(c.kind, ChangeKind::CopySession | ChangeKind::UpdateSession))
        {
            if let Some(i) = scan.scopes.iter().position(|s| s.scope == change.scope) {
                want[i].insert(change.session_id.clone(), change.data.clone());
            }
        }
        let deleted = vec![BTreeSet::new(); scan.scopes.len()];
        Records {
            scan,
            want,
            deleted,
        }
    }

    /// The scope's record of the chat as the scan read it.
    #[must_use]
    pub fn current(&self, scope: usize, id: &str) -> Option<&Value> {
        self.scan.scopes[scope].chats.get(id).map(|c| &c.record)
    }

    /// The record the scope will hold.
    #[must_use]
    pub fn wanted(&self, scope: usize, id: &str) -> Option<&Value> {
        self.want[scope].get(id)
    }

    /// Whether the chat will be archived in the scope.
    #[must_use]
    pub fn archived(&self, scope: usize, id: &str) -> Option<bool> {
        self.wanted(scope, id).map(|r| archive_vote(r, 0).archived)
    }

    /// The parent the chat will have in the scope.
    #[must_use]
    pub fn parent(&self, scope: usize, id: &str) -> Option<String> {
        self.wanted(scope, id).and_then(parent_of)
    }

    /// Every scope keeps the chat as the scan read it: nothing is copied or changed.
    pub fn skip(&mut self, id: &str) {
        for i in 0..self.want.len() {
            match self.current(i, id).cloned() {
                Some(record) => self.want[i].insert(id.to_owned(), record),
                None => self.want[i].remove(id),
            };
        }
    }

    /// Drops the copies of the chat into the scopes `keep` refuses; the other changes stay.
    pub fn keep_copies(&mut self, id: &str, keep: &dyn Fn(usize) -> bool) {
        for i in 0..self.want.len() {
            if !keep(i) && self.current(i, id).is_none() {
                self.want[i].remove(id);
            }
        }
    }

    /// Every copy gets the history of `source`; each keeps its own other keys.
    pub fn take_history(&mut self, id: &str, source: &Value) {
        self.each(id, &mut |r| *r = advance_history(r, source));
    }

    /// Every copy is archived, or none.
    pub fn archive_all(&mut self, id: &str, archived: bool) {
        self.each(id, &mut |r| set_archived(r, archived));
    }

    /// The copies in the given scopes are archived, or not.
    pub fn archive_in(&mut self, scopes: &[usize], id: &str, archived: bool) {
        for &i in scopes {
            if let Some(r) = self.want[i].get_mut(id) {
                set_archived(r, archived);
            }
        }
    }

    /// Every copy becomes a sub-thread of the parent one copy names, with no detach left.
    pub fn link_all(&mut self, id: &str) {
        let source = (0..self.want.len())
            .filter_map(|i| self.want[i].get(id))
            .find(|r| parent_of(r).is_some())
            .or_else(|| {
                (0..self.want.len())
                    .filter_map(|i| self.want[i].get(id))
                    .find(|r| {
                        LINK_KEYS
                            .iter()
                            .any(|k| r.get(*k).is_some_and(|v| !v.is_null()))
                    })
            })
            .cloned();
        let Some(source) = source else {
            return;
        };
        self.each(id, &mut |r| {
            let Some(map) = r.as_object_mut() else {
                return;
            };
            for key in LINK_KEYS {
                match source.get(key).filter(|v| !v.is_null()) {
                    Some(v) => map.insert(key.to_owned(), v.clone()),
                    None => map.remove(key),
                };
            }
            map.remove("lineageDetached");
            map.remove("lineageDetachedBy");
        });
    }

    /// No copy is a sub-thread any more: every lineage key goes.
    pub fn unlink_all(&mut self, id: &str) {
        self.each(id, &mut |r| {
            if let Some(map) = r.as_object_mut() {
                for key in LINEAGE_KEYS {
                    map.remove(key);
                }
            }
        });
    }

    /// The chat is deleted from the scope: its record removed and a tombstone written when it
    /// is on disk, its planned copy dropped otherwise.
    pub fn delete(&mut self, scope: usize, id: &str) {
        self.want[scope].remove(id);
        if self.current(scope, id).is_some() {
            self.deleted[scope].insert(id.to_owned());
        }
    }

    fn each(&mut self, id: &str, f: &mut dyn FnMut(&mut Value)) {
        for scope in &mut self.want {
            if let Some(r) = scope.get_mut(id) {
                f(r);
            }
        }
    }

    /// The catalog changes, in the automatic plan's order: chats by id, each over the scopes,
    /// then the archive indexes. `at_ms` is the time written into tombstones.
    ///
    /// # Errors
    /// When a file or an index cannot be read.
    pub fn changes(&self, catalog: &dyn Catalog, at_ms: i64) -> Result<Vec<Change>, String> {
        let ids: BTreeSet<&String> = self
            .want
            .iter()
            .flat_map(BTreeMap::keys)
            .chain(self.deleted.iter().flatten())
            .collect();
        let mut changes = Vec::new();
        for id in ids {
            for i in 0..self.want.len() {
                if let Some(change) = self.change(catalog, i, id, at_ms)? {
                    changes.push(change);
                }
            }
        }
        for i in 0..self.want.len() {
            if let Some(change) = self.index_change(catalog, i)? {
                changes.push(change);
            }
        }
        Ok(changes)
    }

    fn change(
        &self,
        catalog: &dyn Catalog,
        i: usize,
        id: &str,
        at_ms: i64,
    ) -> Result<Option<Change>, String> {
        let scope = &self.scan.scopes[i].scope;
        let current = self.current(i, id);
        let (kind, data) = if self.deleted[i].contains(id) {
            (ChangeKind::DeleteSession, json!(at_ms))
        } else {
            match (current, self.want[i].get(id)) {
                (None, Some(want)) => (ChangeKind::CopySession, want.clone()),
                (Some(now), Some(want)) if now != want => (ChangeKind::UpdateSession, want.clone()),
                _ => return Ok(None),
            }
        };
        let file = format!("{id}.json");
        let title = self.want[i]
            .get(id)
            .or(current)
            .and_then(|r| r.get("title"))
            .and_then(Value::as_str)
            .unwrap_or_default()
            .to_owned();
        Ok(Some(Change {
            scope: scope.clone(),
            before_sha256: catalog.file_sha256(scope, &file)?,
            file,
            kind,
            session_id: id.to_owned(),
            title,
            data,
        }))
    }

    /// The scope's archive index follows its wanted records, as the automatic plan does; ids
    /// of chats deleted there leave it.
    fn index_change(&self, catalog: &dyn Catalog, i: usize) -> Result<Option<Change>, String> {
        let scope = &self.scan.scopes[i].scope;
        let index = catalog.archived(scope)?;
        let tombstones: BTreeSet<String> = catalog.tombstones(scope)?.into_iter().collect();
        let mut archived: BTreeSet<String> = index.iter().cloned().collect();
        for (id, record) in &self.want[i] {
            if archive_vote(record, 0).archived {
                archived.insert(id.clone());
            } else {
                archived.remove(id);
            }
        }
        archived.retain(|id| !tombstones.contains(id) && !self.deleted[i].contains(id));
        let chosen: Vec<String> = archived.into_iter().collect();
        if chosen == index {
            return Ok(None);
        }
        Ok(Some(Change {
            scope: scope.clone(),
            file: INDEX_FILE.to_owned(),
            kind: ChangeKind::ArchiveIndex,
            session_id: String::new(),
            title: String::new(),
            before_sha256: catalog.file_sha256(scope, INDEX_FILE)?,
            data: json!({ "v": 1, "archived": chosen }),
        }))
    }
}
