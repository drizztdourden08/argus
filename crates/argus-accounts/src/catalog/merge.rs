//! The merge rules of the chat sync, as pure functions on chat records. A record is a
//! `serde_json::Value` object exactly as the Desktop app wrote it.
//!
//! A record's merged keys fall in four groups: the conversation history (compared as a
//! fingerprint; the newest copy wins), the sub-thread lineage, the archived flag and the
//! published artifacts. The last three are merged on their own, so a difference in them never
//! makes two copies of a chat disagree about history. Every other key travels as it is: a new
//! copy keeps every field of the chat, and a copy that lacks a key gets it from another copy.
//! Without `scheduledTaskId` a routine run shows as a plain chat.

use std::cmp::Reverse;

use serde_json::{Map, Value};
use sha2::{Digest, Sha256};

use crate::config::ArchiveRule;

/// The sub-thread keys: a link present on any copy wins unless a copy detached it.
pub const LINEAGE_KEYS: [&str; 5] = [
    "spawnedFrom",
    "lineageDetached",
    "lineageDetachedBy",
    "dispatchParentId",
    "dispatchParentOrigin",
];

const ARCHIVED_KEY: &str = "isArchived";
const ARTIFACTS_KEY: &str = "publishedArtifacts";
const DETACHED_KEY: &str = "lineageDetached";
const MAX_ARTIFACTS: usize = 50;

/// The conversation history keys: the fingerprint of a chat is made of these.
const HISTORY_KEYS: [&str; 27] = [
    "sessionId",
    "cliSessionId",
    "priorCliSessionIds",
    "cwd",
    "originCwd",
    "createdAt",
    "lastActivityAt",
    "title",
    "titleSource",
    "titleTurn",
    "model",
    "effort",
    "isStarred",
    "completedTurns",
    "forkedFromSessionId",
    "forkedAtMessageUuid",
    "lastAssistantUuid",
    "gitAnchors",
    "gitAnchorsFolderRealpath",
    "gitAnchorsLookupOnly",
    "writtenBranches",
    "rewindEdges",
    "transcriptCuts",
    "transcriptModelStates",
    "latestUserFrameAt",
    "postTurnSummary",
    "postTurnSummaryFor",
];

/// A lineage key and the value the merged chat has for it; `None` means the key is absent.
pub type LineageValue = (&'static str, Option<Value>);

/// What one copy of a chat says about being archived.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ArchiveVote {
    /// Whether this copy is archived.
    pub archived: bool,
    /// `lastFocusedAt` of the copy, 0 when absent.
    pub focused_at: i64,
    /// `lastActivityAt` of the copy, 0 when absent.
    pub activity_at: i64,
    /// The record file's modification time.
    pub mtime_ms: i64,
}

fn pick(record: &Value, keep: impl Fn(&str) -> bool) -> Map<String, Value> {
    record
        .as_object()
        .map(|o| {
            o.iter()
                .filter(|(k, _)| keep(k))
                .map(|(k, v)| (k.clone(), v.clone()))
                .collect()
        })
        .unwrap_or_default()
}

/// The conversation history of a record: no archived flag, no lineage, no artifacts.
#[must_use]
pub fn history_record(record: &Value) -> Value {
    Value::Object(pick(record, |k| HISTORY_KEYS.contains(&k)))
}

/// Keys that say something about one account's copy only, never copied to another.
const LOCAL_KEYS: [&str; 1] = ["transcriptUnavailable"];

/// The keys the sync merges by its own rules; a missing one is never filled in from another copy.
fn is_merged_key(key: &str) -> bool {
    HISTORY_KEYS.contains(&key)
        || LINEAGE_KEYS.contains(&key)
        || key == ARCHIVED_KEY
        || key == ARTIFACTS_KEY
}

/// Gives `record` every key it lacks that another copy has, value for value, except the merged
/// and the local keys. The newest copy comes first in `copies`, so its value wins when the
/// others differ.
pub fn fill_missing(record: &mut Value, copies: &[Value]) {
    let Some(map) = record.as_object_mut() else {
        return;
    };
    for copy in copies {
        let Some(other) = copy.as_object() else {
            continue;
        };
        for (key, value) in other {
            let skip = is_merged_key(key) || LOCAL_KEYS.contains(&key.as_str());
            if !skip && !map.contains_key(key) {
                map.insert(key.clone(), value.clone());
            }
        }
    }
}

/// What a new copy of a chat starts from: the newest copy (first in `copies`) with every field
/// but the local ones, and the keys only the other copies have.
#[must_use]
pub fn full_record(copies: &[Value]) -> Value {
    let mut record = copies
        .first()
        .map_or_else(|| Value::Object(Map::new()), Value::clone);
    if let Some(map) = record.as_object_mut() {
        map.retain(|k, _| !LOCAL_KEYS.contains(&k.as_str()));
    }
    fill_missing(&mut record, copies);
    record
}

/// A key-sorted copy of a value, so the same content always gives the same bytes.
#[must_use]
pub fn canonical(value: &Value) -> Value {
    match value {
        Value::Object(map) => {
            let mut keys: Vec<&String> = map.keys().collect();
            keys.sort();
            Value::Object(
                keys.into_iter()
                    .map(|k| (k.clone(), canonical(&map[k])))
                    .collect(),
            )
        }
        Value::Array(items) => Value::Array(items.iter().map(canonical).collect()),
        other => other.clone(),
    }
}

/// The bytes a record or index is written as: sorted keys, 2-space indent, final newline.
#[must_use]
pub fn json_bytes(value: &Value) -> Vec<u8> {
    let mut text = serde_json::to_string_pretty(&canonical(value)).unwrap_or_default();
    text.push('\n');
    text.into_bytes()
}

/// SHA-256 of some bytes, lower hex.
#[must_use]
pub fn sha256_hex(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}

/// The history fingerprint: two copies with the same one have the same conversation.
#[must_use]
pub fn fingerprint(record: &Value) -> String {
    sha256_hex(&json_bytes(&history_record(record)))
}

/// `current` with its history replaced by the history of `source`. The account's own keys,
/// lineage, archived flag and artifacts stay, as the caller merges those.
#[must_use]
pub fn advance_history(current: &Value, source: &Value) -> Value {
    let mut record = current.clone();
    if let Some(map) = record.as_object_mut() {
        map.retain(|k, _| !HISTORY_KEYS.contains(&k.as_str()));
        map.extend(pick(source, |k| HISTORY_KEYS.contains(&k)));
    }
    record
}

fn present(record: &Value, key: &str) -> Option<Value> {
    match record.get(key) {
        None | Some(Value::Null) => None,
        Some(Value::Bool(false)) if key == DETACHED_KEY => None,
        Some(v) => Some(v.clone()),
    }
}

/// The lineage the merged chat has. `copies` come newest first. A link on any copy wins; when a
/// copy detached the chat on purpose, that copy's lineage is kept as it is.
#[must_use]
pub fn merge_lineage(copies: &[&Value]) -> Vec<LineageValue> {
    let detached = copies
        .iter()
        .find(|r| r.get(DETACHED_KEY) == Some(&Value::Bool(true)));
    LINEAGE_KEYS
        .iter()
        .map(|&key| {
            let value = match detached {
                Some(copy) => present(copy, key),
                None => copies.iter().find_map(|r| present(r, key)),
            };
            (key, value)
        })
        .collect()
}

/// Puts the merged lineage on a record. A leftover `lineageDetached: false` is not worth a write.
pub fn set_lineage(record: &mut Value, merged: &[LineageValue]) {
    let Some(map) = record.as_object_mut() else {
        return;
    };
    for (key, value) in merged {
        match value {
            Some(v) => {
                map.insert((*key).to_owned(), v.clone());
            }
            None if map.get(*key) == Some(&Value::Bool(false)) => {}
            None => {
                map.remove(*key);
            }
        }
    }
}

/// The vote of one copy of a chat.
#[must_use]
pub fn archive_vote(record: &Value, mtime_ms: i64) -> ArchiveVote {
    ArchiveVote {
        archived: record.get(ARCHIVED_KEY) == Some(&Value::Bool(true)),
        focused_at: millis(record, "lastFocusedAt"),
        activity_at: millis(record, "lastActivityAt"),
        mtime_ms,
    }
}

#[allow(clippy::cast_possible_truncation)]
fn millis(record: &Value, key: &str) -> i64 {
    match record.get(key) {
        Some(v) => v
            .as_i64()
            .or_else(|| v.as_f64().map(|f| f as i64))
            .unwrap_or(0),
        None => 0,
    }
}

/// Whether the merged chat is archived. With `Anywhere` one archived copy archives all; with
/// `Newest` the copy changed last decides (focus time, then activity time, then file time), and
/// a full tie archives.
#[must_use]
pub fn merge_archive(votes: &[ArchiveVote], rule: ArchiveRule) -> bool {
    match rule {
        ArchiveRule::Anywhere => votes.iter().any(|v| v.archived),
        ArchiveRule::Newest => votes
            .iter()
            .max_by_key(|v| (v.focused_at, v.activity_at, v.mtime_ms, v.archived))
            .is_some_and(|v| v.archived),
    }
}

/// Sets the archived flag, leaving a record that already says so as it is.
pub fn set_archived(record: &mut Value, archived: bool) {
    let now = record.get(ARCHIVED_KEY) == Some(&Value::Bool(true));
    if now == archived {
        return;
    }
    if let Some(map) = record.as_object_mut() {
        map.insert(ARCHIVED_KEY.to_owned(), Value::Bool(archived));
    }
}

fn artifact_time(artifact: &Value) -> i64 {
    millis(artifact, "updatedAt")
}

/// The published artifacts of all copies: one per URL (the most recently updated), newest first,
/// at most 50.
#[must_use]
pub fn merge_artifacts(copies: &[&Value]) -> Vec<Value> {
    let mut chosen: Vec<Value> = Vec::new();
    for artifact in copies
        .iter()
        .filter_map(|r| r.get(ARTIFACTS_KEY).and_then(Value::as_array))
        .flatten()
        .filter(|a| a.get("url").is_some_and(Value::is_string))
    {
        let url = artifact.get("url");
        match chosen.iter_mut().find(|c| c.get("url") == url) {
            Some(known) if artifact_time(artifact) > artifact_time(known) => {
                *known = artifact.clone();
            }
            Some(_) => {}
            None => chosen.push(artifact.clone()),
        }
    }
    chosen.sort_by_key(|a| {
        (
            Reverse(artifact_time(a)),
            a.get("url")
                .and_then(Value::as_str)
                .unwrap_or("")
                .to_owned(),
        )
    });
    chosen.truncate(MAX_ARTIFACTS);
    chosen
}

/// Puts the merged artifacts on a record; none removes the key.
pub fn set_artifacts(record: &mut Value, artifacts: &[Value]) {
    let Some(map) = record.as_object_mut() else {
        return;
    };
    if artifacts.is_empty() {
        map.remove(ARTIFACTS_KEY);
    } else {
        map.insert(ARTIFACTS_KEY.to_owned(), Value::Array(artifacts.to_vec()));
    }
}

// test-tag: critical
#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::*;

    fn vote(archived: bool, focused_at: i64, activity_at: i64, mtime_ms: i64) -> ArchiveVote {
        ArchiveVote {
            archived,
            focused_at,
            activity_at,
            mtime_ms,
        }
    }

    #[test]
    fn fingerprint_ignores_archived_lineage_and_account_keys() {
        let a = json!({ "sessionId": "local_a", "title": "t", "isArchived": true,
            "spawnedFrom": { "sessionId": "x" }, "permissionMode": "acceptEdits" });
        let b = json!({ "sessionId": "local_a", "title": "t", "isArchived": false });
        assert_eq!(fingerprint(&a), fingerprint(&b));
        let c = json!({ "sessionId": "local_a", "title": "other" });
        assert_ne!(fingerprint(&a), fingerprint(&c));
    }

    #[test]
    fn a_new_copy_keeps_every_field_of_the_chat() {
        let rec = json!({ "sessionId": "local_a", "title": "t", "spawnedFrom": { "name": "n" },
            "permissionMode": "bypass", "enabledMcpTools": ["x"], "scheduledTaskId": "st_1",
            "folderPlace": { "at": 2 }, "spawnSeed": "s", "transcriptUnavailable": true });
        let out = full_record(std::slice::from_ref(&rec));
        let mut expected = rec.clone();
        expected
            .as_object_mut()
            .expect("an object")
            .remove("transcriptUnavailable");
        assert_eq!(out, expected);
    }

    #[test]
    fn a_stripped_copy_gets_the_keys_it_lacks_and_keeps_its_own() {
        let mut stripped = json!({ "sessionId": "local_a", "title": "mine" });
        let full = json!({ "sessionId": "local_a", "title": "theirs", "scheduledTaskId": "st_1" });
        fill_missing(&mut stripped, &[full]);
        assert_eq!(stripped["scheduledTaskId"], "st_1");
        assert_eq!(stripped["title"], "mine");
    }

    #[test]
    fn advance_history_keeps_account_keys() {
        let current = json!({ "sessionId": "local_a", "title": "old", "permissionMode": "plan",
            "isArchived": true });
        let source = json!({ "sessionId": "local_a", "title": "new", "model": "opus" });
        let out = advance_history(&current, &source);
        assert_eq!(out["title"], "new");
        assert_eq!(out["model"], "opus");
        assert_eq!(out["permissionMode"], "plan");
        assert_eq!(out["isArchived"], true);
    }

    #[test]
    fn a_link_on_any_copy_wins() {
        let linked = json!({ "spawnedFrom": { "sessionId": "p" }, "dispatchParentId": "d" });
        let bare = json!({ "lineageDetached": false });
        let merged = merge_lineage(&[&bare, &linked]);
        let mut rec = json!({ "lineageDetached": false });
        set_lineage(&mut rec, &merged);
        assert_eq!(rec["spawnedFrom"]["sessionId"], "p");
        assert_eq!(rec["dispatchParentId"], "d");
        assert_eq!(rec["lineageDetached"], false);
    }

    #[test]
    fn a_detached_copy_keeps_the_chat_detached() {
        let linked = json!({ "spawnedFrom": { "sessionId": "p" } });
        let detached = json!({ "lineageDetached": true, "lineageDetachedBy": "owner" });
        let merged = merge_lineage(&[&linked, &detached]);
        let mut rec = linked.clone();
        set_lineage(&mut rec, &merged);
        assert!(rec.get("spawnedFrom").is_none());
        assert_eq!(rec["lineageDetached"], true);
        assert_eq!(rec["lineageDetachedBy"], "owner");
    }

    #[test]
    fn newest_archive_change_wins() {
        let votes = [vote(true, 5, 1, 1), vote(false, 9, 1, 1)];
        assert!(!merge_archive(&votes, ArchiveRule::Newest));
        let by_activity = [vote(true, 5, 8, 1), vote(false, 5, 2, 9)];
        assert!(merge_archive(&by_activity, ArchiveRule::Newest));
        let by_mtime = [vote(true, 0, 0, 4), vote(false, 0, 0, 7)];
        assert!(!merge_archive(&by_mtime, ArchiveRule::Newest));
    }

    #[test]
    fn anywhere_archives_everywhere() {
        let votes = [vote(true, 1, 1, 1), vote(false, 9, 9, 9)];
        assert!(merge_archive(&votes, ArchiveRule::Anywhere));
        assert!(!merge_archive(
            &[vote(false, 1, 1, 1)],
            ArchiveRule::Anywhere
        ));
    }

    #[test]
    fn artifacts_union_by_url_newest_first() {
        let a = json!({ "publishedArtifacts": [{ "url": "u1", "updatedAt": 1 },
            { "url": "u2", "updatedAt": 5 }] });
        let b = json!({ "publishedArtifacts": [{ "url": "u1", "updatedAt": 9 }, { "title": "no url" }] });
        let merged = merge_artifacts(&[&a, &b]);
        assert_eq!(merged.len(), 2);
        assert_eq!(merged[0]["url"], "u1");
        assert_eq!(merged[0]["updatedAt"], 9);
    }

    #[test]
    fn json_bytes_is_sorted_and_ends_in_a_newline() {
        let bytes = json_bytes(&json!({ "b": 1, "a": { "d": 1, "c": 2 } }));
        let text = String::from_utf8(bytes).unwrap();
        assert!(text.ends_with("}\n"));
        assert!(text.find("\"a\"").unwrap() < text.find("\"b\"").unwrap());
        assert!(text.find("\"c\"").unwrap() < text.find("\"d\"").unwrap());
    }
}
