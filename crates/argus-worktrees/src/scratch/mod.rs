//! Scratch roots: folders that are not git worktrees (`X:/brock-gate-*`, `X:/tmp`) and that the owner allowed Argus to
//! remove once nothing in them was touched for `minAgeDays` (`worktrees.scratch`, empty by default).
//!
//! Each root lists its direct children that match an include glob. A child whose newest file time anywhere inside is
//! older than the limit is untouched. Mode `report` only lists them; mode `remove` removes them, links first, never
//! following one, with one activity line each. The guard (`guard`) is separate from the leftover guard and refuses the
//! root itself, anything not a direct child, anything not matched by an include glob, any folder with a `.git` inside, a
//! main checkout or a board folder, and a folder a live process uses. The would-remove list is `Service::scratch_plan`.

mod guard;
mod walk;

use crate::fs::Entry;
use crate::paths::{join_path, name_allowed};
use crate::service::{Service, age_ms};
use crate::tree::remove_entry;
use guard::Verdict;
use serde::Serialize;

/// What a scratch root does with the untouched folders it finds.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum ScratchMode {
    #[default]
    Report,
    Remove,
}

/// One configured scratch root.
#[derive(Clone, Debug, PartialEq)]
pub struct ScratchRoot {
    pub root: String,
    pub include: Vec<String>,
    pub min_age_days: f64,
    pub mode: ScratchMode,
}

/// A folder the last step found untouched and removable. In mode `remove` a folder that went is not listed.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ScratchItem {
    pub root: String,
    pub path: String,
    pub mode: ScratchMode,
}

/// What the last step left behind.
#[derive(Default)]
pub struct ScratchState {
    pub at_ms: i64,
    pub plan: Vec<ScratchItem>,
}

impl Service {
    /// The folders the last scratch step would remove (or did not manage to).
    pub fn scratch_plan(&self) -> Vec<ScratchItem> {
        self.st().scratch.plan.clone()
    }

    /// One pass over every scratch root, at most once per `scratch_every_ms`. True when it ran: the plan is new.
    pub fn scratch_step(&self) -> bool {
        let now = self.now();
        let due = now - self.st().scratch.at_ms >= self.opts.scratch_every_ms;
        if !self.opts.hygiene || self.opts.scratch.is_empty() || self.stopped() || !due {
            return false;
        }
        let protected = self.protected_paths();
        let plan: Vec<ScratchItem> = self
            .opts
            .scratch
            .iter()
            .flat_map(|entry| self.scratch_root(entry, &protected, now))
            .collect();
        self.st().scratch = ScratchState { at_ms: now, plan };
        true
    }

    fn scratch_root(
        &self,
        entry: &ScratchRoot,
        protected: &[String],
        now: i64,
    ) -> Vec<ScratchItem> {
        let cutoff = now - age_ms(entry.min_age_days);
        let Ok(children) = self.deps.fs.read_dir(&entry.root) else {
            return Vec::new();
        };
        children
            .iter()
            .filter(|c| is_candidate(c, entry, self.ci()))
            .map(|c| join_path(&entry.root, &c.name))
            .filter(|path| self.scratch_verdict(entry, path, protected, cutoff) == Verdict::Remove)
            .filter_map(|path| self.scratch_one(entry, path))
            .collect()
    }

    /// The item when the folder stays (mode `report`, or the removal failed); none once it is gone.
    fn scratch_one(&self, entry: &ScratchRoot, path: String) -> Option<ScratchItem> {
        let item = ScratchItem {
            root: entry.root.clone(),
            path,
            mode: entry.mode,
        };
        if entry.mode == ScratchMode::Report {
            return Some(item);
        }
        let project = self.deps.store.overlord_board();
        // Every link goes as a link first, then the folder; the file system never follows one.
        match remove_entry(self.deps.fs.as_ref(), &item.path, true) {
            Ok(()) => {
                let days = Self::duration_text(entry.min_age_days);
                let text = format!(
                    "removed scratch folder {} (untouched for {days})",
                    item.path
                );
                self.note(&project, "info", &text, None);
                None
            }
            Err(err) => {
                let text = format!("could not remove scratch folder {}: {err}", item.path);
                self.note(&project, "warn", &text, None);
                Some(item)
            }
        }
    }
}

fn is_candidate(child: &Entry, entry: &ScratchRoot, ci: bool) -> bool {
    child.is_dir && !child.is_link && name_allowed(&entry.include, &[], &child.name, ci)
}
