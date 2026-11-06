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

/// Where a worktree stands. The aliases read the states of the rule before 2026-11-06.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum HygieneState {
    /// An agent, work in progress, a launcher, a bundle, a run or a process is attached to it.
    #[serde(alias = "in-use")]
    Attached,
    /// Nothing is attached and nothing would be lost: Argus removes it, or a removal it approved runs.
    #[serde(alias = "released", alias = "closing")]
    Removing,
    /// Nothing is attached but something would be lost: its lead is asked to approve the removal.
    #[serde(alias = "review")]
    AwaitingLead,
    /// The lead did not answer in time: the Overlord was told to delete it.
    Reported,
    /// On the Overlord's exception list: never removed, never reported.
    #[serde(alias = "kept")]
    Exception,
}

impl HygieneState {
    /// Nothing is attached and it is not an exception.
    pub(crate) fn free(self) -> bool {
        matches!(self, Self::Removing | Self::AwaitingLead | Self::Reported)
    }
}

/// One worktree's mark.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct Mark {
    pub state: HygieneState,
    pub path: String,
    #[serde(default)]
    pub branch: Option<String>,
    /// The owning lead: the thread asked, and the one told of the removal.
    #[serde(default)]
    pub thread: Option<String>,
    #[serde(default)]
    pub agent: Option<String>,
    /// Why it is attached, what would be lost, or the reason of its exception.
    #[serde(default)]
    pub why: Option<String>,
    /// When it entered this state; for a worktree nothing is attached to, when it was first seen so.
    pub since: i64,
    /// When its lead was asked to approve the removal.
    #[serde(default)]
    pub asked: Option<i64>,
    /// When the Overlord was told.
    #[serde(default)]
    pub reported: Option<i64>,
    /// The removal request that carries it.
    #[serde(default)]
    pub request: Option<String>,
}

/// A removal the removal itself refused: the worktree is clean no more while it stays as it was.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct Refused {
    pub head: Option<String>,
    pub dirty: u32,
    pub why: String,
}

/// A worktree on the Overlord's exception list.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Exception {
    pub path: String,
    pub reason: String,
    /// The lead that asked the Overlord for it.
    pub asked_by: Option<String>,
    /// The Overlord's thread that wrote it.
    pub by: String,
    pub at: i64,
    /// When it ends; none holds until the Overlord clears it.
    pub until: Option<i64>,
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
    /// The Overlord's exception list, by worktree id.
    #[serde(default)]
    pub exceptions: BTreeMap<String, Exception>,
}

impl Memory {
    /// One more removal for the page; the oldest go past the limit.
    pub fn removed_one(&mut self, done: RemovedDoc) {
        self.marks.remove(&done.id);
        self.refused.remove(&done.id);
        self.removed.retain(|r| r.id != done.id);
        self.removed.push(done);
        let extra = self.removed.len().saturating_sub(REMOVED_KEPT);
        self.removed.drain(..extra);
    }

    /// The exceptions that still hold: not expired, and their worktree still listed.
    pub fn prune_exceptions(&mut self, alive: &[String], now: i64) {
        self.exceptions
            .retain(|id, e| alive.contains(id) && e.until.is_none_or(|u| u > now));
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
    pub since: String,
    /// When Argus removes it (a clean one nothing is attached to, once the debounce ends).
    pub goes_at: Option<String>,
    /// When its lead was asked to approve the removal.
    pub asked_at: Option<String>,
    /// When the Overlord was told.
    pub reported_at: Option<String>,
    /// When its exception ends.
    pub until: Option<String>,
}

/// The hygiene part of the summary.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct HygieneMeta {
    pub on: bool,
    /// How long a worktree stays unattached before the hygiene acts.
    pub debounce_ms: i64,
    /// How long the lead's ask stays unanswered before the Overlord is told.
    pub report_after_ms: i64,
    /// The last worktrees the hygiene removed, newest first.
    pub removed: Vec<RemovedDoc>,
}

/// When Argus removes the worktree: a clean one nothing is attached to, the debounce after it was first seen so.
fn goes_at(mark: &Mark, debounce: i64) -> Option<i64> {
    (mark.state == HygieneState::Removing && mark.request.is_none())
        .then_some(mark.since + debounce)
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
        let debounce = self.opts.detach_debounce_ms;
        self.with_hygiene(project, |m| {
            let until = m.exceptions.get(id).and_then(|e| e.until);
            m.marks.get(id).map(|mark| HygieneDoc {
                state: mark.state,
                why: mark.why.clone(),
                agent: mark.agent.clone(),
                thread: mark.thread.clone(),
                since: iso_from_ms(mark.since),
                goes_at: goes_at(mark, debounce).map(iso_from_ms),
                asked_at: mark.asked.map(iso_from_ms),
                reported_at: mark.reported.map(iso_from_ms),
                until: until
                    .filter(|_| mark.state == HygieneState::Exception)
                    .map(iso_from_ms),
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
            on: self.opts.hygiene,
            debounce_ms: self.opts.detach_debounce_ms,
            report_after_ms: self.opts.report_after_ms,
            removed,
        }
    }
}
