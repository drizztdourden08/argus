//! The leftover deletions a batch has queued and not yet carried out, kept in the key-value table
//! (`worktrees:leftovers:<project>`).
//!
//! The deletion queue lives in memory and a folder is renamed only when its job gets its turn, so a restart used to
//! lose a queued batch: the folders went back to plain leftovers and nothing ever deleted them. An intent is written
//! when the batch is queued and dropped when its job ends. After a restart the first scans find intents with no job:
//! they are queued again through the same batch, with the same guards, once the folder is measured.

use crate::batch::BatchInput;
use crate::board::Caller;
use crate::fact::Fact;
use crate::service::Service;
use crate::sync::locked;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Mutex;

/// The key-value key of a project's queued leftover deletions.
pub(crate) fn intents_key(project: &str) -> String {
    format!("worktrees:leftovers:{project}")
}

/// One folder a batch queued: its row, who asked and with which limits.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct Intent {
    pub row: String,
    /// The thread that asked; none when the owner's page did.
    #[serde(default)]
    pub thread: Option<String>,
    pub older_than_days: f64,
    #[serde(default)]
    pub confirm_young: bool,
    #[serde(default)]
    pub reason: Option<String>,
}

impl Intent {
    pub(crate) fn of(fact: &Fact, input: &BatchInput, min_days: f64) -> Self {
        Self {
            row: fact.id.clone(),
            thread: input.caller.thread().map(ToString::to_string),
            older_than_days: min_days,
            confirm_young: input.confirm_young,
            reason: input.reason.clone(),
        }
    }

    /// What makes two intents one batch again.
    fn batch_key(&self) -> (Option<String>, u64, bool, Option<String>) {
        (
            self.thread.clone(),
            self.older_than_days.to_bits(),
            self.confirm_young,
            self.reason.clone(),
        )
    }
}

#[derive(Default, Deserialize, Serialize)]
struct File {
    #[serde(default)]
    pending: Vec<Intent>,
}

/// The intents of every project, loaded on first use.
#[derive(Default)]
pub(crate) struct Intents(Mutex<HashMap<String, Vec<Intent>>>);

impl Service {
    /// Runs `f` on the project's intents, loading them first when this is the first look. Never call git, the removal
    /// or a derive from `f`.
    fn with_intents<T>(&self, project: &str, f: impl FnOnce(&mut Vec<Intent>) -> T) -> T {
        let mut all = locked(&self.intents.0);
        let list = all.entry(project.to_string()).or_insert_with(|| {
            let text = self.deps.store.kv_get(&intents_key(project));
            let file: File = text
                .and_then(|t| serde_json::from_str(&t).ok())
                .unwrap_or_default();
            file.pending
        });
        f(list)
    }

    /// Change the intents and write them down. A failed write is not fatal, only logged.
    fn change_intents(&self, project: &str, f: impl FnOnce(&mut Vec<Intent>)) {
        let written = self.with_intents(project, |list| {
            f(list);
            let file = File {
                pending: list.clone(),
            };
            let text = serde_json::to_string(&file).unwrap_or_default();
            self.deps.store.kv_set(&intents_key(project), &text)
        });
        if let Err(err) = written {
            self.deps.notifier.problem(&format!(
                "{project}: the queued leftover deletions were not saved: {err}"
            ));
        }
    }

    pub(crate) fn remember_intents(&self, project: &str, added: Vec<Intent>) {
        self.change_intents(project, |list| {
            list.retain(|i| !added.iter().any(|a| a.row == i.row));
            list.extend(added);
        });
    }

    pub(crate) fn forget_intents(&self, project: &str, rows: &[String]) {
        if self.with_intents(project, |list| list.iter().all(|i| !rows.contains(&i.row))) {
            return;
        }
        self.change_intents(project, |list| list.retain(|i| !rows.contains(&i.row)));
    }

    /// Queue again the deletions a restart cut off. A row that is not a leftover any more is forgotten; one not measured
    /// yet waits for a later scan. Runs at the end of every scan, and does nothing when no intent is left.
    pub(crate) fn resume_batches(&self, project: &str) {
        let intents = self.with_intents(project, |list| list.clone());
        if intents.is_empty() {
            return;
        }
        let facts = self.facts_of(project);
        let (mut gone, mut ready) = (Vec::new(), Vec::<Intent>::new());
        for intent in intents {
            if self.active_job_for(project, &intent.row).is_some() {
                continue;
            }
            match facts.iter().find(|f| f.id == intent.row && f.exists) {
                Some(f) if f.leftover.as_ref().is_some_and(|l| l.pending) => {}
                Some(f) if f.leftover.is_some() => ready.push(intent),
                _ => gone.push(intent.row),
            }
        }
        self.forget_intents(project, &gone);
        while let Some(first) = ready.first().cloned() {
            let (same, rest): (Vec<Intent>, Vec<Intent>) = ready
                .into_iter()
                .partition(|i| i.batch_key() == first.batch_key());
            ready = rest;
            self.requeue(project, &first, same.into_iter().map(|i| i.row).collect());
        }
    }

    /// One batch again: the rows that do not start are forgotten.
    fn requeue(&self, project: &str, like: &Intent, rows: Vec<String>) {
        let caller = like.thread.clone().map_or(Caller::Page, Caller::Thread);
        let input = BatchInput {
            ids: Some(rows.clone()),
            older_than_days: Some(like.older_than_days),
            confirm_young: like.confirm_young,
            reason: like.reason.clone(),
            ..BatchInput::new(project, caller)
        };
        let started: Vec<String> = self
            .remove_leftovers(&input)
            .map(|out| out.jobs.into_iter().map(|j| j.row_id).collect())
            .unwrap_or_default();
        let dropped: Vec<String> = rows.into_iter().filter(|r| !started.contains(r)).collect();
        self.forget_intents(project, &dropped);
    }
}
