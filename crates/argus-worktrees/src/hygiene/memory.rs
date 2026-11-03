//! What the hygiene remembers per project, kept in the key-value table, and how the page reads it.

use crate::clock::iso_from_ms;
use crate::service::Service;
use crate::sync::locked;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, HashMap};
use std::sync::Mutex;

/// The last removals a project keeps for the page.
const REMOVED_KEPT: usize = 20;

/// The key-value key of a project's hygiene.
pub fn hygiene_key(project: &str) -> String {
    format!("worktrees:hygiene:{project}")
}

/// Where a worktree stands.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum HygieneState {
    /// A running agent owns it, or a process, a run, a launcher, a pull request or work in progress holds it.
    #[serde(rename = "in-use")]
    InUse,
    /// Every condition holds: Argus removes it after the grace.
    Released,
    /// Its item closed: Argus removes it after the grace (a reopen inside it cancels), its full backup first.
    Closing,
    /// Something keeps Argus from removing it (uncommitted work, commits no ref keeps, a lock): the thread decides.
    Review,
    /// The thread kept it.
    Kept,
}

/// One worktree's mark.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct Mark {
    pub state: HygieneState,
    pub path: String,
    #[serde(default)]
    pub branch: Option<String>,
    #[serde(default)]
    pub thread: Option<String>,
    #[serde(default)]
    pub agent: Option<String>,
    #[serde(default)]
    pub why: Option<String>,
    /// The item of the worktree when that item is closed.
    #[serde(default)]
    pub item: Option<String>,
    /// When it first was in this state.
    pub since: i64,
    /// When its thread was told it is released.
    #[serde(default)]
    pub told: Option<i64>,
    /// The removal request that carries it.
    #[serde(default)]
    pub request: Option<String>,
}

/// A removal the removal itself refused: the worktree is not released again while it stays as it was.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct Refused {
    pub head: Option<String>,
    pub dirty: u32,
    pub why: String,
}

/// A worktree the hygiene removed, as the page lists it.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RemovedDoc {
    pub id: String,
    pub path: String,
    pub branch: Option<String>,
    pub thread: Option<String>,
    pub at: String,
    pub text: String,
}

/// A project's hygiene.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct Memory {
    #[serde(default)]
    pub marks: BTreeMap<String, Mark>,
    #[serde(default)]
    pub refused: BTreeMap<String, Refused>,
    #[serde(default)]
    pub removed: Vec<RemovedDoc>,
    /// When each worktree was first seen, by id.
    #[serde(default)]
    pub seen: BTreeMap<String, i64>,
    /// The worktrees flagged for having no item, by id: whether their thread was told.
    #[serde(default)]
    pub unlinked: BTreeMap<String, bool>,
    /// The worktrees of closed items that stay, by id, with the reason their thread was told.
    #[serde(default)]
    pub stuck: BTreeMap<String, String>,
}

impl Memory {
    /// One more removal for the page; the oldest go past the limit.
    pub fn removed_one(&mut self, done: RemovedDoc) {
        self.marks.remove(&done.id);
        self.refused.remove(&done.id);
        self.stuck.remove(&done.id);
        self.removed.retain(|r| r.id != done.id);
        self.removed.push(done);
        let extra = self.removed.len().saturating_sub(REMOVED_KEPT);
        self.removed.drain(..extra);
    }
}

/// The hygiene of every project, loaded on first use.
#[derive(Default)]
pub(crate) struct Memories(Mutex<HashMap<String, Memory>>);

/// The hygiene part of a row.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct HygieneDoc {
    pub state: HygieneState,
    pub why: Option<String>,
    pub agent: Option<String>,
    pub thread: Option<String>,
    /// The closed item the worktree belongs to.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub item: Option<String>,
    pub since: String,
    /// When Argus removes it (a released one whose thread was told, or one whose item closed).
    pub goes_at: Option<String>,
}

/// The hygiene part of the summary.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct HygieneMeta {
    pub on: bool,
    pub grace_ms: i64,
    /// A worktree is removed with its item (`worktrees.closeWithItem`).
    pub close_with_item: bool,
    /// How long after its item closed (`worktrees.itemCloseGraceMs`).
    pub item_close_grace_ms: i64,
    /// The last worktrees the hygiene removed, newest first.
    pub removed: Vec<RemovedDoc>,
}

/// When Argus removes the worktree: a released one `grace` after its thread was told, one whose item closed
/// `close_grace` after it was first seen so.
fn goes_at(mark: &Mark, grace: i64, close_grace: i64) -> Option<i64> {
    match mark.state {
        HygieneState::Released => mark.told.map(|t| t + grace),
        HygieneState::Closing => Some(mark.since + close_grace),
        _ => None,
    }
}

impl Service {
    /// Runs `f` on the project's hygiene and saves it. Locked for short stretches only: never git, a removal or a
    /// process listing inside `f`.
    pub(crate) fn with_hygiene<T>(&self, project: &str, f: impl FnOnce(&mut Memory) -> T) -> T {
        let mut all = locked(&self.hygiene.0);
        let memory = all.entry(project.to_string()).or_insert_with(|| {
            self.deps
                .store
                .kv_get(&hygiene_key(project))
                .and_then(|text| serde_json::from_str(&text).ok())
                .unwrap_or_default()
        });
        let before = memory.clone();
        let out = f(memory);
        if *memory != before {
            let text = serde_json::to_string(&*memory).unwrap_or_default();
            let _ = self.deps.store.kv_set(&hygiene_key(project), &text);
        }
        out
    }

    /// The row's hygiene, from the last step.
    pub(crate) fn hygiene_of(&self, project: &str, id: &str) -> Option<HygieneDoc> {
        let (grace, close_grace) = (self.opts.release_grace_ms, self.opts.item_close_grace_ms);
        self.with_hygiene(project, |m| {
            m.marks.get(id).map(|mark| HygieneDoc {
                state: mark.state,
                why: mark.why.clone(),
                agent: mark.agent.clone(),
                thread: mark.thread.clone(),
                item: mark.item.clone(),
                since: iso_from_ms(mark.since),
                goes_at: goes_at(mark, grace, close_grace).map(iso_from_ms),
            })
        })
    }

    /// Is the worktree flagged for having had no item for an hour (the `no item` chip)?
    pub(crate) fn is_flagged_unlinked(&self, project: &str, id: &str) -> bool {
        self.with_hygiene(project, |m| m.unlinked.contains_key(id))
    }

    /// The summary's hygiene.
    pub(crate) fn hygiene_meta(&self, project: &str) -> HygieneMeta {
        let mut removed = self.with_hygiene(project, |m| m.removed.clone());
        removed.reverse();
        HygieneMeta {
            on: self.opts.release,
            grace_ms: self.opts.release_grace_ms,
            close_with_item: self.opts.close_with_item,
            item_close_grace_ms: self.opts.item_close_grace_ms,
            removed,
        }
    }
}
