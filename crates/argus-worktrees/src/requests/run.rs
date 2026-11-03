//! Doing a request: a fresh scan, the targets, each one through the very same removal, and the end.

use super::record::{Outcome, RequestRec, RequestStatus, Target};
use super::text::one_line;
use super::{CLOSED_SENTENCE_MAX, SENTENCE_MAX};
use crate::board::Caller;
use crate::clock::iso_from_ms;
use crate::error::WtError;
use crate::jobs::JobState;
use crate::progress::target_key;
use crate::request::RemoveInput;
use crate::service::Service;

/// The words of a refusal; a blank one still says something.
fn sentence_of(err: &WtError) -> String {
    let text = one_line(&err.message, SENTENCE_MAX);
    if text.is_empty() {
        "refused".to_string()
    } else {
        text
    }
}

/// A removal with its item gives the backup and how to restore it in full: the lead's note repeats it.
fn sentence_max(rec: &RequestRec) -> usize {
    if rec.mode == super::Mode::ItemClosed {
        CLOSED_SENTENCE_MAX
    } else {
        SENTENCE_MAX
    }
}

fn refusal(target: &Target, err: &WtError) -> Outcome {
    Outcome {
        id: target.id.clone(),
        label: target.label.clone(),
        ok: false,
        text: sentence_of(err),
        code: Some(err.code.as_str().to_string()),
    }
}

impl Service {
    /// The caller of a removal: the thread that asked, or Argus for the item that closed.
    fn caller_of(rec: &RequestRec, target: &Target) -> Caller {
        match rec.items.get(&target.id) {
            Some(item) if rec.mode == super::Mode::ItemClosed => {
                Caller::ItemClosed { item: item.clone() }
            }
            _ => Caller::Thread(rec.by.clone()),
        }
    }

    fn removal_of(rec: &RequestRec, target: &Target) -> RemoveInput {
        let mut input = RemoveInput::new(&rec.key, &target.id, Self::caller_of(rec, target));
        input.branch = rec.branch;
        input.discard = rec.discard || rec.mode == super::Mode::ItemClosed;
        input.owner_entry.clone_from(&rec.owner_entry);
        input.reason.clone_from(&rec.reason);
        if rec.mode == super::Mode::Leftovers {
            input.older_than_days = rec.older_than_days;
        }
        input.empty_goes = rec.mode == super::Mode::Released;
        input
    }

    /// A leftover starts a background job (and a folder parked aside before a restart has one already): the outcome
    /// waits for the job. `done` is the sentence of a job that finished.
    fn job_outcome(
        &self,
        rec: &RequestRec,
        target: &Target,
        (path, job_id): (&str, &str),
        done: &str,
    ) -> Outcome {
        self.jobs_queued_ran(&rec.key);
        let job = self.jobs(&rec.key).into_iter().find(|j| j.id == job_id);
        let unfinished = job.filter(|j| j.state != JobState::Done);
        let (ok, text, code) = match unfinished {
            Some(job) => {
                let why = job.error.map(|e| format!(": {e}")).unwrap_or_default();
                let text = format!(
                    "deleting {path} did not finish ({}{why})",
                    job.state.as_str()
                );
                (
                    false,
                    one_line(&text, SENTENCE_MAX),
                    Some("job_failed".to_string()),
                )
            }
            None => (true, one_line(done, SENTENCE_MAX), None),
        };
        Outcome {
            id: target.id.clone(),
            label: target.label.clone(),
            ok,
            text,
            code,
        }
    }

    /// One target through the very same removal the CLI route calls.
    pub(super) fn remove_one(&self, rec: &RequestRec, target: &Target) -> Outcome {
        match self.remove(&Self::removal_of(rec, target)) {
            Ok(out) if out.leftover && out.job_id.is_some() => {
                let path = out.path.as_deref().unwrap_or_default();
                let job_id = out.job_id.as_deref().unwrap_or_default();
                let done = format!("deleted leftover folder {path}");
                self.job_outcome(rec, target, (path, job_id), &done)
            }
            Ok(out) => {
                let said = one_line(&out.text, sentence_max(rec));
                Outcome {
                    id: target.id.clone(),
                    label: target.label.clone(),
                    ok: true,
                    text: if said.is_empty() {
                        format!("removed {}", target.label)
                    } else {
                        said
                    },
                    code: None,
                }
            }
            Err(err) => self
                .parked_outcome(rec, target)
                .unwrap_or_else(|| refusal(target, &err)),
        }
    }

    /// After a restart: the target's folder was already parked aside and a job finishes its deletion (it goes on from
    /// what is left, never from zero). The request follows that job instead of reporting the removal's refusal.
    fn parked_outcome(&self, rec: &RequestRec, target: &Target) -> Option<Outcome> {
        let key = target_key(&target.id, self.ci());
        let job = self.active_job_for(&rec.key, &key)?;
        let done = format!(
            "deleted {}: it was parked aside for deletion before, and Argus finished it",
            job.path
        );
        Some(self.job_outcome(rec, target, (job.path.as_str(), job.id.as_str()), &done))
    }

    /// Settle what is to be removed: the named targets, or the ones a listing right now gives.
    fn settle_targets(
        &self,
        project: &str,
        id: &str,
        rec: &RequestRec,
    ) -> Result<RequestRec, WtError> {
        if rec.resolved {
            return Ok(rec.clone());
        }
        let found = self.resolve_request(rec)?;
        self.update_request(project, id, |r| {
            r.targets = found.targets;
            r.resolved = true;
            r.skipped = found.skipped;
            r.note = found.note;
        });
        self.save_requests(project);
        Ok(self.request_of(project, id).unwrap_or_else(|| rec.clone()))
    }

    /// The targets with no outcome yet, one at a time. False when the worker was stopped on the way.
    fn run_targets(&self, project: &str, id: &str) -> Result<bool, WtError> {
        let _ = self.scan_now(project);
        if self.requests_halted() {
            return Ok(false);
        }
        let Some(first) = self.request_of(project, id) else {
            return Ok(false);
        };
        let rec = self.settle_targets(project, id, &first)?;
        for target in rec.targets.iter().skip(first.results.len()) {
            if self.requests_halted() {
                return Ok(false);
            }
            let outcome = match self.still_released(&rec, target) {
                Ok(()) => self.remove_watched(project, &rec, target),
                Err(outcome) => outcome,
            };
            self.update_request(project, id, |r| r.results.push(outcome));
            self.save_requests(project);
        }
        Ok(true)
    }

    fn finish_request(&self, project: &str, id: &str) {
        let finished = iso_from_ms(self.now());
        self.update_request(project, id, |rec| {
            rec.status = match (rec.removed(), rec.refused()) {
                (_, 0) => RequestStatus::Done,
                (0, _) => RequestStatus::Refused,
                _ => RequestStatus::Partial,
            };
            rec.finished_at = Some(finished);
        });
        self.save_requests(project);
    }

    /// Carry out one request, then tell the thread. A stop on the way leaves it running: the next start goes on.
    pub(super) fn process_request(&self, project: &str, id: &str) {
        let started = iso_from_ms(self.now());
        self.update_request(project, id, |rec| {
            rec.status = RequestStatus::Running;
            rec.started_at.get_or_insert(started);
        });
        self.save_requests(project);
        match self.run_targets(project, id) {
            Ok(false) => return,
            Ok(true) => {}
            Err(err) => {
                let text = format!("the request could not run: {}", sentence_of(&err));
                self.deps
                    .notifier
                    .problem(&format!("{project}: worktree request {id}: {text}"));
                let failed = Outcome {
                    id: String::new(),
                    label: String::new(),
                    ok: false,
                    text,
                    code: None,
                };
                self.update_request(project, id, |rec| rec.results.push(failed));
            }
        }
        self.finish_request(project, id);
        self.after_released(project, id);
        self.deliver_request(project, id);
    }
}
