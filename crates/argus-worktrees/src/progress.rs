//! What a removal is doing right now, by worktree id: the step, the entries done so far, since when. A request that
//! waits on a removal shows it (`current.step`), so a long one is never a silent `running`.
//!
//! Steps: waiting for its turn (a scan or another removal runs), checking, `node_modules` clear (entries removed so
//! far), links, git worktree remove, folder delete (a deletion job: entries removed of the total).
//!
//! A wait for the turn past `NAME_HOLDER_AFTER_MS` names the holder instead: what took the project's turn, since
//! when, and the step that holder is on.

use crate::clock::iso_from_ms;
use crate::jobs::JobState;
use crate::paths::{clean_path, is_worktree_id, worktree_id};
use crate::service::Service;
use crate::turns::Holder;
use serde::{Deserialize, Serialize};

/// The step of a removal that waits for the project's turn.
pub(crate) const WAITING: &str = "waiting for its turn (a scan or another removal runs)";

/// A wait for the turn this long names the holder.
const NAME_HOLDER_AFTER_MS: i64 = 30_000;

/// One step of a removal.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct Step {
    pub name: String,
    /// Entries done so far, when the step counts them.
    pub done: Option<u64>,
    /// Entries in all, when known.
    pub total: Option<u64>,
    pub since: String,
    pub since_ms: i64,
    /// Seconds on this step, as of the reading.
    pub for_s: i64,
}

/// The worktree id a removal names (its id, or its path).
pub(crate) fn target_key(id: &str, case_insensitive: bool) -> String {
    let id = id.trim();
    if is_worktree_id(id) {
        id.to_string()
    } else {
        worktree_id(&clean_path(id), case_insensitive)
    }
}

impl Service {
    /// The removal of `id` (an id or a path) starts a step.
    pub(crate) fn step(&self, id: &str, name: &str) {
        let now = self.now();
        let step = Step {
            name: name.to_string(),
            since: iso_from_ms(now),
            since_ms: now,
            ..Step::default()
        };
        self.st().steps.insert(target_key(id, self.ci()), step);
    }

    /// The step counted one more batch of entries.
    pub(crate) fn step_done(&self, id: &str, done: u64) {
        if let Some(step) = self.st().steps.get_mut(&target_key(id, self.ci())) {
            step.done = Some(done);
        }
    }

    /// The removal of `id` ended.
    pub(crate) fn step_end(&self, id: &str) {
        self.st().steps.remove(&target_key(id, self.ci()));
    }

    /// What the removal of `id` does now: its deletion job when one runs, else its last step.
    pub(crate) fn step_of(&self, project: &str, id: &str) -> Option<Step> {
        let key = target_key(id, self.ci());
        let now = self.now();
        let mut step = self
            .job_step(project, &key)
            .or_else(|| self.st().steps.get(&key).cloned())?;
        step.for_s = (now - step.since_ms).max(0) / 1000;
        if step.name == WAITING
            && now - step.since_ms >= NAME_HOLDER_AFTER_MS
            && let Some(holder) = self.exclusive.holder(project)
        {
            step.name = self.holder_text(&holder, now);
        }
        Some(step)
    }

    /// The step of a long wait, as in this example:
    /// `waiting for its turn: the removal of 1a2b3c4d5e for lead has held it since <iso> (248 s), at links`.
    fn holder_text(&self, holder: &Holder, now: i64) -> String {
        let at = holder
            .key
            .as_ref()
            .and_then(|key| self.st().steps.get(key).cloned())
            .filter(|s| s.name != WAITING)
            .map(|s| {
                let done = s
                    .done
                    .map(|n| format!(", {n} entries removed"))
                    .unwrap_or_default();
                format!(", at {}{done}", s.name)
            })
            .unwrap_or_default();
        format!(
            "waiting for its turn: {} has held it since {} ({} s){at}",
            holder.what,
            iso_from_ms(holder.since_ms),
            (now - holder.since_ms).max(0) / 1000
        )
    }

    /// A deletion job of the folder (a leftover's, or a parked folder a restart took up): its count of the total. The
    /// time is the job's queueing; the seconds count from the step the removal last started, if any.
    fn job_step(&self, project: &str, key: &str) -> Option<Step> {
        let job = self.active_job_for(project, key)?;
        let since_ms = self
            .st()
            .steps
            .get(key)
            .map_or_else(|| self.now(), |s| s.since_ms);
        Some(Step {
            name: format!("folder delete (job {}, {})", job.id, job.state.as_str()),
            done: (job.state == JobState::Running).then_some(job.removed),
            total: job.total,
            since: job.queued_at.clone(),
            since_ms,
            for_s: 0,
        })
    }
}
