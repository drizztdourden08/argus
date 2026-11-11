//! The sync plan: what to write so every scope holds every chat, with the newest history, the
//! merged lineage and archived state, and archive indexes that follow the records. Nothing is
//! written here. The plan is deterministic, and a second plan made right after applying the
//! first is empty.

use std::cmp::Reverse;
use std::collections::{BTreeMap, BTreeSet, HashMap};
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};
use serde_json::{Value, json};

use super::merge::{
    advance_history, archive_vote, fingerprint, merge_archive, merge_artifacts, merge_lineage,
    portable_record, set_archived, set_artifacts, set_lineage,
};
use super::read::is_uuid;
use crate::config::ArchiveRule;
use crate::ports::{Catalog, Transcripts};
use crate::types::Scope;

const INDEX_FILE: &str = "archived-sessions.idx";

/// What a change does.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ChangeKind {
    /// A chat the scope lacks is written.
    CopySession,
    /// A chat the scope has is brought up to date.
    UpdateSession,
    /// The scope's archive index follows its records.
    ArchiveIndex,
}

/// One file to write.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Change {
    /// The scope the file is in.
    pub scope: Scope,
    /// The bare file name: `local_<uuid>.json` or `archived-sessions.idx`.
    pub file: String,
    /// What the change does.
    pub kind: ChangeKind,
    /// The chat, empty for an archive index.
    pub session_id: String,
    /// The chat's title, empty for an archive index.
    pub title: String,
    /// SHA-256 of the file when it was read for the plan; `None` for a file that is not there.
    pub before_sha256: Option<String>,
    /// The new content: the record, or `{"v":1,"archived":[...]}` for an index.
    pub data: Value,
}

/// A chat the sync leaves alone, and why.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Conflict {
    /// The chat.
    pub session_id: String,
    /// The scopes involved, as `<account>/<org>`.
    pub scopes: Vec<String>,
    /// `history_differs` or `target_occupied`.
    pub reason: String,
}

/// A chat not copied into a scope because it was deleted there.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TombstoneSkip {
    /// The chat.
    pub session_id: String,
    /// The scope that holds the tombstone.
    pub scope: Scope,
}

/// The counts of a plan.
#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PlanSummary {
    /// Scopes looked at.
    pub scopes: usize,
    /// Chats copied into a scope.
    pub copies: usize,
    /// Chats updated.
    pub updates: usize,
    /// Archive indexes rewritten.
    pub archive_indexes: usize,
    /// Chats blocked by a missing, ambiguous or invalid transcript.
    pub missing_transcripts: usize,
    /// Chats with a conflict.
    pub conflicts: usize,
    /// Copies skipped for a tombstone.
    pub tombstone_skips: usize,
    /// Files to write.
    pub writes: usize,
}

/// Everything a sync run would do.
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Plan {
    /// The files to write, in the order to write them.
    pub changes: Vec<Change>,
    /// Chats left alone because the copies disagree.
    pub conflicts: Vec<Conflict>,
    /// Chats left alone because no single valid transcript was found.
    pub missing_transcripts: Vec<String>,
    /// Copies skipped because the scope deleted the chat.
    pub tombstone_skips: Vec<TombstoneSkip>,
    /// The counts.
    pub summary: PlanSummary,
}

struct Loaded {
    value: Value,
    sha: Option<String>,
    mtime: i64,
}

struct ScopeData {
    scope: Scope,
    records: BTreeMap<String, Loaded>,
    blocked: BTreeSet<String>,
    deleted: BTreeSet<String>,
    index: Vec<String>,
    index_sha: Option<String>,
}

/// Plans the sync of every scope the catalog has.
///
/// # Errors
/// When a record, an index or a file cannot be read.
pub fn build_plan(
    catalog: &dyn Catalog,
    transcripts: &dyn Transcripts,
    rule: ArchiveRule,
) -> Result<Plan, String> {
    let mut scopes = catalog.scopes()?;
    scopes.sort_by_key(Scope::name);
    let scopes = scopes
        .into_iter()
        .map(|scope| load_scope(catalog, scope))
        .collect::<Result<Vec<_>, _>>()?;
    let mut planner = Planner {
        transcripts,
        rule,
        scopes,
        plan: Plan::default(),
        valid: HashMap::new(),
    };
    planner.run();
    Ok(planner.finish())
}

fn load_scope(catalog: &dyn Catalog, scope: Scope) -> Result<ScopeData, String> {
    let mut records = BTreeMap::new();
    let mut blocked = BTreeSet::new();
    for id in catalog.session_ids(&scope)? {
        let sha = catalog.file_sha256(&scope, &format!("{id}.json"))?;
        let value = catalog.read_session(&scope, &id)?;
        if is_syncable(&value, &id) {
            let mtime = catalog.session_mtime_ms(&scope, &id)?;
            records.insert(id, Loaded { value, sha, mtime });
        } else {
            blocked.insert(id);
        }
    }
    let index_sha = catalog.file_sha256(&scope, INDEX_FILE)?;
    let index = catalog.archived(&scope)?;
    let deleted = catalog.tombstones(&scope)?.into_iter().collect();
    Ok(ScopeData {
        scope,
        records,
        blocked,
        deleted,
        index,
        index_sha,
    })
}

/// A record that ran in Claude Code on this PC: not a remote one, with a valid CLI session id.
pub(crate) fn is_syncable(record: &Value, id: &str) -> bool {
    let remote = |key: &str| record.get(key).is_some_and(|v| !v.is_null());
    record.get("sessionId").and_then(Value::as_str) == Some(id)
        && !remote("sshConfig")
        && !remote("wslConfig")
        && record
            .get("cliSessionId")
            .and_then(Value::as_str)
            .is_some_and(is_uuid)
}

fn activity(record: &Value) -> i64 {
    archive_vote(record, 0).activity_at
}

fn title(record: &Value) -> String {
    record
        .get("title")
        .and_then(Value::as_str)
        .unwrap_or_default()
        .to_owned()
}

struct Planner<'a> {
    transcripts: &'a dyn Transcripts,
    rule: ArchiveRule,
    scopes: Vec<ScopeData>,
    plan: Plan,
    valid: HashMap<PathBuf, bool>,
}

/// What every copy of a chat ends up with besides its history.
struct Overlay {
    lineage: Vec<super::merge::LineageValue>,
    archived: bool,
    artifacts: Vec<Value>,
}

impl Planner<'_> {
    fn run(&mut self) {
        let ids: BTreeSet<String> = self
            .scopes
            .iter()
            .flat_map(|s| s.records.keys().cloned())
            .collect();
        for id in &ids {
            self.plan_session(id);
        }
        self.plan_indexes();
    }

    fn finish(mut self) -> Plan {
        let count = |kind| self.plan.changes.iter().filter(|c| c.kind == kind).count();
        self.plan.summary = PlanSummary {
            scopes: self.scopes.len(),
            copies: count(ChangeKind::CopySession),
            updates: count(ChangeKind::UpdateSession),
            archive_indexes: count(ChangeKind::ArchiveIndex),
            missing_transcripts: self.plan.missing_transcripts.len(),
            conflicts: self.plan.conflicts.len(),
            tombstone_skips: self.plan.tombstone_skips.len(),
            writes: self.plan.changes.len(),
        };
        self.plan
    }

    /// The scopes that hold the chat, newest activity first, ties by scope order.
    fn holders(&self, id: &str) -> Vec<usize> {
        let mut holders: Vec<usize> = (0..self.scopes.len())
            .filter(|&i| self.scopes[i].records.contains_key(id))
            .collect();
        holders.sort_by_key(|&i| Reverse(activity(&self.scopes[i].records[id].value)));
        holders
    }

    fn plan_session(&mut self, id: &str) {
        let holders = self.holders(id);
        let values: Vec<Value> = holders
            .iter()
            .map(|&i| self.scopes[i].records[id].value.clone())
            .collect();
        let Some(source) = self.pick_source(id, &holders, &values) else {
            return;
        };
        if !self.has_transcript(&values[source]) {
            self.plan.missing_transcripts.push(id.to_owned());
            return;
        }
        let overlay = self.overlay(&holders, &values, id);
        for i in 0..self.scopes.len() {
            if holders.contains(&i) {
                self.update(i, id, &values[source], &overlay);
            } else {
                self.copy(i, id, &values[source], &overlay);
            }
        }
    }

    /// The copy whose history wins, as an index into `values` (newest first); `None` when the
    /// newest histories differ, which is reported as a conflict.
    fn pick_source(&mut self, id: &str, holders: &[usize], values: &[Value]) -> Option<usize> {
        let prints: Vec<String> = values.iter().map(fingerprint).collect();
        let newest = activity(&values[0]);
        let latest: BTreeSet<&String> = values
            .iter()
            .zip(&prints)
            .filter(|(v, _)| activity(v) == newest)
            .map(|(_, p)| p)
            .collect();
        if latest.len() == 1 {
            return Some(0);
        }
        let scopes = holders
            .iter()
            .map(|&i| self.scopes[i].scope.name())
            .collect();
        self.plan.conflicts.push(Conflict {
            session_id: id.to_owned(),
            scopes,
            reason: "history_differs".to_owned(),
        });
        None
    }

    /// Whether the chat's current transcript is one valid file.
    fn has_transcript(&mut self, record: &Value) -> bool {
        let current = record.get("cliSessionId").and_then(Value::as_str);
        let prior = record
            .get("priorCliSessionIds")
            .and_then(Value::as_array)
            .into_iter()
            .flatten()
            .rev()
            .filter_map(Value::as_str);
        for sid in current.into_iter().chain(prior) {
            let found = self.transcripts.find(sid);
            match found.as_slice() {
                [] => {}
                [path] => return self.is_valid(path),
                _ => return false,
            }
        }
        false
    }

    fn is_valid(&mut self, path: &Path) -> bool {
        *self
            .valid
            .entry(path.to_path_buf())
            .or_insert_with(|| self.transcripts.is_valid(path))
    }

    fn overlay(&self, holders: &[usize], values: &[Value], id: &str) -> Overlay {
        let refs: Vec<&Value> = values.iter().collect();
        let votes: Vec<_> = holders
            .iter()
            .zip(values)
            .map(|(&i, v)| archive_vote(v, self.scopes[i].records[id].mtime))
            .collect();
        Overlay {
            lineage: merge_lineage(&refs),
            archived: merge_archive(&votes, self.rule),
            artifacts: merge_artifacts(&refs),
        }
    }

    fn update(&mut self, i: usize, id: &str, source: &Value, overlay: &Overlay) {
        let current = self.scopes[i].records[id].value.clone();
        let mut record = if fingerprint(&current) == fingerprint(source) {
            current.clone()
        } else {
            advance_history(&current, source)
        };
        apply_overlay(&mut record, overlay);
        if let Some(map) = record.as_object_mut() {
            map.remove("transcriptUnavailable");
        }
        if record == current {
            return;
        }
        let loaded = self.scopes[i]
            .records
            .get_mut(id)
            .expect("holder has the record");
        let before = loaded.sha.clone();
        loaded.value = record.clone();
        self.push(i, ChangeKind::UpdateSession, id, before, record);
    }

    fn copy(&mut self, i: usize, id: &str, source: &Value, overlay: &Overlay) {
        let scope = &self.scopes[i];
        if scope.deleted.contains(id) {
            let skip = TombstoneSkip {
                session_id: id.to_owned(),
                scope: scope.scope.clone(),
            };
            self.plan.tombstone_skips.push(skip);
            return;
        }
        if scope.blocked.contains(id) {
            let scopes = vec![scope.scope.name()];
            let reason = "target_occupied".to_owned();
            self.plan.conflicts.push(Conflict {
                session_id: id.to_owned(),
                scopes,
                reason,
            });
            return;
        }
        let mut record = portable_record(source);
        apply_overlay(&mut record, overlay);
        let loaded = Loaded {
            value: record.clone(),
            sha: None,
            mtime: 0,
        };
        self.scopes[i].records.insert(id.to_owned(), loaded);
        self.push(i, ChangeKind::CopySession, id, None, record);
    }

    fn push(&mut self, i: usize, kind: ChangeKind, id: &str, before: Option<String>, data: Value) {
        let session = kind != ChangeKind::ArchiveIndex;
        self.plan.changes.push(Change {
            scope: self.scopes[i].scope.clone(),
            file: if session {
                format!("{id}.json")
            } else {
                INDEX_FILE.to_owned()
            },
            kind,
            session_id: if session {
                id.to_owned()
            } else {
                String::new()
            },
            title: if session { title(&data) } else { String::new() },
            before_sha256: before,
            data,
        });
    }

    /// Each scope's archive index follows its records. Ids of archived chats not loaded as
    /// records stay, except the ones deleted there.
    fn plan_indexes(&mut self) {
        for i in 0..self.scopes.len() {
            let scope = &self.scopes[i];
            let mut archived: BTreeSet<String> = scope.index.iter().cloned().collect();
            for (id, loaded) in &scope.records {
                if archive_vote(&loaded.value, 0).archived {
                    archived.insert(id.clone());
                } else {
                    archived.remove(id);
                }
            }
            archived.retain(|id| !scope.deleted.contains(id));
            let chosen: Vec<String> = archived.into_iter().collect();
            if chosen != scope.index {
                let before = scope.index_sha.clone();
                let data = json!({ "v": 1, "archived": chosen });
                self.push(i, ChangeKind::ArchiveIndex, "", before, data);
            }
        }
    }
}

fn apply_overlay(record: &mut Value, overlay: &Overlay) {
    set_lineage(record, &overlay.lineage);
    set_archived(record, overlay.archived);
    set_artifacts(record, &overlay.artifacts);
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::{ChangeKind, Plan, build_plan};
    use crate::catalog::fixtures::{Fixture, cli_id, record, session_id};
    use crate::config::ArchiveRule;
    use crate::ports::Catalog;

    fn plan(fx: &Fixture, rule: ArchiveRule) -> Plan {
        build_plan(&fx.catalog(), &fx.transcripts(), rule).unwrap()
    }

    fn kinds(plan: &Plan) -> Vec<ChangeKind> {
        plan.changes.iter().map(|c| c.kind).collect()
    }

    #[test]
    fn copies_a_missing_chat_with_portable_fields_only() {
        let fx = Fixture::new();
        let (a, b) = (fx.scope("a1", "o1"), fx.scope("a2", "o2"));
        fx.ensure(&b);
        let mut rec = record(1, 10);
        rec["permissionMode"] = json!("bypassPermissions");
        rec["enabledMcpTools"] = json!(["x"]);
        fx.put(&a, 1, &rec);
        fx.transcript(1);
        let plan = plan(&fx, ArchiveRule::Newest);
        assert_eq!(kinds(&plan), vec![ChangeKind::CopySession]);
        let change = &plan.changes[0];
        assert_eq!(change.scope, b);
        assert_eq!(change.file, format!("{}.json", session_id(1)));
        assert!(change.before_sha256.is_none());
        assert_eq!(change.data["permissionMode"], "default");
        assert!(change.data.get("enabledMcpTools").is_none());
        assert_eq!(change.data["title"], "chat 1");
    }

    #[test]
    fn update_takes_history_from_the_newest_copy_and_keeps_account_settings() {
        let fx = Fixture::new();
        let (a, b) = (fx.scope("a1", "o1"), fx.scope("a2", "o2"));
        let mut old = record(1, 10);
        old["permissionMode"] = json!("plan");
        let mut new = record(1, 20);
        new["title"] = json!("newer title");
        fx.put(&a, 1, &old);
        fx.put(&b, 1, &new);
        fx.transcript(1);
        let plan = plan(&fx, ArchiveRule::Newest);
        assert_eq!(kinds(&plan), vec![ChangeKind::UpdateSession]);
        let change = &plan.changes[0];
        assert_eq!(change.scope, a);
        assert_eq!(change.data["title"], "newer title");
        assert_eq!(change.data["permissionMode"], "plan");
        assert!(change.before_sha256.is_some());
    }

    #[test]
    fn equal_activity_with_different_history_is_a_conflict() {
        let fx = Fixture::new();
        let (a, b) = (fx.scope("a1", "o1"), fx.scope("a2", "o2"));
        let mut other = record(1, 10);
        other["title"] = json!("different");
        fx.put(&a, 1, &record(1, 10));
        fx.put(&b, 1, &other);
        fx.transcript(1);
        let plan = plan(&fx, ArchiveRule::Newest);
        assert!(plan.changes.is_empty());
        assert_eq!(plan.conflicts.len(), 1);
        assert_eq!(plan.conflicts[0].session_id, session_id(1));
    }

    #[test]
    fn a_tombstone_is_never_resurrected() {
        let fx = Fixture::new();
        let (a, b) = (fx.scope("a1", "o1"), fx.scope("a2", "o2"));
        fx.ensure(&b);
        fx.put(&a, 1, &record(1, 10));
        fx.transcript(1);
        fx.tombstone(&b, 1);
        let plan = plan(&fx, ArchiveRule::Newest);
        assert!(plan.changes.is_empty());
        assert_eq!(plan.tombstone_skips.len(), 1);
        assert_eq!(plan.tombstone_skips[0].scope, b);
    }

    #[test]
    fn records_that_never_ran_or_run_remotely_are_skipped() {
        let fx = Fixture::new();
        let (a, b) = (fx.scope("a1", "o1"), fx.scope("a2", "o2"));
        fx.ensure(&b);
        let mut never = record(1, 10);
        never.as_object_mut().unwrap().remove("cliSessionId");
        let mut ssh = record(2, 10);
        ssh["sshConfig"] = json!({ "host": "h" });
        fx.put(&a, 1, &never);
        fx.put(&a, 2, &ssh);
        fx.transcript(1);
        fx.transcript(2);
        let plan = plan(&fx, ArchiveRule::Newest);
        assert!(plan.changes.is_empty() && plan.conflicts.is_empty());
    }

    #[test]
    fn a_missing_or_invalid_transcript_blocks_the_chat() {
        let fx = Fixture::new();
        let (a, b) = (fx.scope("a1", "o1"), fx.scope("a2", "o2"));
        fx.ensure(&b);
        fx.put(&a, 1, &record(1, 10));
        fx.put(&a, 2, &record(2, 10));
        fx.bad_transcript(2);
        let plan = plan(&fx, ArchiveRule::Newest);
        assert!(plan.changes.is_empty());
        assert_eq!(plan.missing_transcripts, vec![session_id(1), session_id(2)]);
    }

    #[test]
    fn the_same_transcript_in_two_projects_is_ambiguous() {
        let fx = Fixture::new();
        let (a, b) = (fx.scope("a1", "o1"), fx.scope("a2", "o2"));
        fx.ensure(&b);
        fx.put(&a, 1, &record(1, 10));
        fx.transcript(1);
        fx.transcript_in(1, "other-project");
        let plan = plan(&fx, ArchiveRule::Newest);
        assert!(plan.changes.is_empty());
        assert_eq!(plan.missing_transcripts.len(), 1);
    }

    #[test]
    fn the_prior_transcript_is_used_when_the_current_one_is_absent() {
        let fx = Fixture::new();
        let (a, b) = (fx.scope("a1", "o1"), fx.scope("a2", "o2"));
        fx.ensure(&b);
        let mut rec = record(1, 10);
        rec["priorCliSessionIds"] = json!([cli_id(9)]);
        fx.put(&a, 1, &rec);
        fx.transcript(9);
        let plan = plan(&fx, ArchiveRule::Newest);
        assert_eq!(kinds(&plan), vec![ChangeKind::CopySession]);
    }

    #[test]
    fn a_link_travels_unless_a_copy_detached_it() {
        let fx = Fixture::new();
        let (a, b, c) = (
            fx.scope("a1", "o1"),
            fx.scope("a2", "o2"),
            fx.scope("a3", "o3"),
        );
        let mut linked = record(1, 10);
        linked["spawnedFrom"] = json!({ "sessionId": "local_parent", "name": "p" });
        linked["dispatchParentId"] = json!("d1");
        fx.put(&a, 1, &linked);
        fx.put(&b, 1, &record(1, 10));
        fx.transcript(1);
        fx.ensure(&c);
        let plan = plan(&fx, ArchiveRule::Newest);
        assert_eq!(plan.changes.len(), 2);
        for change in &plan.changes {
            assert_eq!(change.data["spawnedFrom"]["name"], "p");
            assert_eq!(change.data["dispatchParentId"], "d1");
        }

        let mut detached = record(1, 10);
        detached["lineageDetached"] = json!(true);
        detached["lineageDetachedBy"] = json!("owner");
        fx.put(&b, 1, &detached);
        let plan = self::plan(&fx, ArchiveRule::Newest);
        let on_a = plan.changes.iter().find(|ch| ch.scope == a).unwrap();
        assert!(on_a.data.get("spawnedFrom").is_none());
        assert_eq!(on_a.data["lineageDetached"], true);
    }

    #[test]
    fn archived_follows_the_newest_change_and_the_index_follows_the_records() {
        let fx = Fixture::new();
        let (a, b) = (fx.scope("a1", "o1"), fx.scope("a2", "o2"));
        let mut archived = record(1, 10);
        archived["isArchived"] = json!(true);
        archived["lastFocusedAt"] = json!(5);
        let mut live = record(1, 10);
        live["lastFocusedAt"] = json!(9);
        fx.put(&a, 1, &archived);
        fx.put(&b, 1, &live);
        fx.write_index(&a, &[session_id(1)]);
        fx.transcript(1);
        let plan = plan(&fx, ArchiveRule::Newest);
        let update = plan
            .changes
            .iter()
            .find(|c| c.kind == ChangeKind::UpdateSession)
            .unwrap();
        assert_eq!(update.scope, a);
        assert_eq!(update.data["isArchived"], false);
        let index = plan
            .changes
            .iter()
            .find(|c| c.kind == ChangeKind::ArchiveIndex)
            .unwrap();
        assert_eq!(index.scope, a);
        assert_eq!(index.data["archived"], json!([]));
    }

    #[test]
    fn anywhere_archives_everywhere_and_indexes_follow() {
        let fx = Fixture::new();
        let (a, b) = (fx.scope("a1", "o1"), fx.scope("a2", "o2"));
        let mut archived = record(1, 10);
        archived["isArchived"] = json!(true);
        fx.put(&a, 1, &archived);
        fx.put(&b, 1, &record(1, 10));
        fx.write_index(&a, &[session_id(1)]);
        fx.transcript(1);
        let plan = plan(&fx, ArchiveRule::Anywhere);
        assert_eq!(plan.summary.updates, 1);
        let on_b = plan
            .changes
            .iter()
            .find(|c| c.scope == b && c.kind == ChangeKind::UpdateSession)
            .unwrap();
        assert_eq!(on_b.data["isArchived"], true);
        let index = plan
            .changes
            .iter()
            .find(|c| c.scope == b && c.kind == ChangeKind::ArchiveIndex)
            .unwrap();
        assert_eq!(index.data["archived"], json!([session_id(1)]));
        assert!(fx.catalog().archived(&b).unwrap().is_empty());
    }

    #[test]
    fn the_order_is_deterministic() {
        let fx = Fixture::new();
        let (a, b, c) = (
            fx.scope("a1", "o1"),
            fx.scope("a2", "o2"),
            fx.scope("a3", "o3"),
        );
        fx.ensure(&b);
        fx.ensure(&c);
        for n in [3, 1, 2] {
            fx.put(&a, n, &record(n, 10));
            fx.transcript(n);
        }
        let first = plan(&fx, ArchiveRule::Newest);
        let second = plan(&fx, ArchiveRule::Newest);
        assert_eq!(first, second);
        let ids: Vec<&str> = first
            .changes
            .iter()
            .map(|c| c.session_id.as_str())
            .filter(|s| !s.is_empty())
            .collect();
        assert!(ids.windows(2).all(|w| w[0] <= w[1]));
    }
}
