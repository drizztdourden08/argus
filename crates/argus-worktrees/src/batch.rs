//! All the old leftovers of a project in one go. The listing's facts filter first (answered now); each remaining folder
//! then goes through the same guards as a single removal, one after another in the background (a job each).

use crate::board::{BoardFacts, Caller};
use crate::clock::DAY_MS;
use crate::error::Result;
use crate::fact::Fact;
use crate::guards::check_caller;
use crate::jobs::{JobDoc, JobKind, JobState, trim_jobs};
use crate::measure::about_text;
use crate::paths::{clean_path, is_worktree_id, worktree_id};
use crate::request::RemoveInput;
use crate::rule::Rule;
use crate::service::Service;
use std::sync::{Arc, Mutex};

/// A request to delete leftovers in bulk.
#[derive(Clone, Debug)]
pub struct BatchInput {
    pub project: String,
    pub caller: Caller,
    /// Only these (ids or paths); none means every leftover row.
    pub ids: Option<Vec<String>>,
    pub older_than_days: Option<f64>,
    pub confirm_young: bool,
    pub reason: Option<String>,
}

impl BatchInput {
    /// A request for every old leftover of the project, with nothing else to say.
    pub fn new(project: &str, caller: Caller) -> Self {
        Self {
            project: project.to_string(),
            caller,
            ids: None,
            older_than_days: None,
            confirm_young: false,
            reason: None,
        }
    }
}

/// A folder the batch did not take, and why.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Skipped {
    pub id: String,
    pub path: Option<String>,
    pub reason: String,
}

/// What the batch queued and skipped.
#[derive(Clone, Debug)]
pub struct BatchOutcome {
    pub started: usize,
    pub skipped: Vec<Skipped>,
    pub size_mb: f64,
    pub files: u64,
    pub jobs: Vec<JobDoc>,
    pub older_than_days: f64,
    pub text: String,
}

#[derive(Default)]
struct Tally {
    left: usize,
    started: usize,
    refused: usize,
}

fn round1(value: f64) -> f64 {
    (value * 10.0).round() / 10.0
}

fn changed_text(age_days: Option<f64>) -> String {
    match age_days {
        None => "at an unknown time".to_string(),
        Some(age) if age < 1.0 => {
            #[allow(clippy::cast_possible_truncation)]
            let hours = ((age * 24.0).floor() as i64).max(1);
            format!("{hours} {} ago", if hours <= 1 { "hour" } else { "hours" })
        }
        #[allow(clippy::cast_possible_truncation)]
        Some(age) => format!("{} days ago", age.floor() as i64),
    }
}

impl Service {
    fn refuse_batch(&self, input: &BatchInput, message: &str) {
        let (who, why) = Self::who_and_why(&as_remove(input, ""));
        self.note(
            &input.project,
            "warn",
            &format!("refused to delete leftover folders {who}: {message}{why}"),
            input.caller.thread(),
        );
    }

    fn skip_reason(
        &self,
        board: &BoardFacts,
        input: &BatchInput,
        fact: &Fact,
        rule: Option<&Rule>,
        min_days: f64,
    ) -> Option<String> {
        let Some(rule) = rule else {
            return Some(
                "not yours to delete (its rule was removed or is invalid (a wildcard path))"
                    .to_string(),
            );
        };
        if let Err(err) = check_caller(board, &input.caller, &[rule]) {
            return Some(format!(
                "not yours to delete ({})",
                err.message.chars().take(80).collect::<String>()
            ));
        }
        let leftover = fact.leftover.as_ref()?;
        #[allow(clippy::cast_precision_loss)]
        let age = leftover
            .newest_ms
            .map(|n| (self.now() - n) as f64 / DAY_MS as f64);
        if self.active_job_for(&input.project, &fact.id).is_some() {
            Some("already being deleted".to_string())
        } else if leftover.pending {
            Some("not measured yet (the next scan measures it)".to_string())
        } else if leftover.is_link {
            Some("a link or junction".to_string())
        } else if leftover.has_nested_git {
            Some(format!(
                "holds a nested git repository ({})",
                leftover.nested_git.as_deref().unwrap_or(".git")
            ))
        } else if !input.caller.is_page() && leftover.partial {
            Some("too big to look through".to_string())
        } else if !self.is_made(&input.project, &fact.path) && age.is_none_or(|a| a < min_days) {
            Some(format!(
                "changed {}, under {}",
                changed_text(age),
                Self::duration_text(min_days)
            ))
        } else {
            None
        }
    }

    fn batch_text(queued: &[Fact], skipped: &[Skipped]) -> (String, f64, u64) {
        let mb: f64 = queued
            .iter()
            .map(|f| f.leftover.as_ref().and_then(|l| l.size_mb).unwrap_or(0.0))
            .sum();
        let files: u64 = queued
            .iter()
            .map(|f| f.leftover.as_ref().and_then(|l| l.files).unwrap_or(0))
            .sum();
        let text = if queued.is_empty() {
            format!(
                "no leftover folder to delete{}",
                if skipped.is_empty() {
                    String::new()
                } else {
                    format!(" ({} skipped)", skipped.len())
                }
            )
        } else {
            let s = if queued.len() == 1 { "" } else { "s" };
            let more = if skipped.is_empty() {
                String::new()
            } else {
                format!(", skipped {}", skipped.len())
            };
            format!(
                "queued {} leftover folder{s} for deletion ({}, {files} files; no backup: git does not know them){more}",
                queued.len(),
                about_text(Some(mb))
            )
        };
        (text, round1(mb), files)
    }

    fn wanted_ids(&self, ids: Option<&Vec<String>>) -> Option<Vec<String>> {
        ids.map(|ids| {
            let one = |x: &String| {
                if is_worktree_id(x) {
                    x.clone()
                } else {
                    worktree_id(&clean_path(x), self.ci())
                }
            };
            ids.iter().map(one).collect()
        })
    }

    /// The leftover rows asked for, split into the ones to queue and the ones skipped with their reason.
    fn split_rows(
        &self,
        board: &BoardFacts,
        input: &BatchInput,
        min_days: f64,
    ) -> (Vec<Fact>, Vec<Skipped>) {
        let want = self.wanted_ids(input.ids.as_ref());
        let keep = |f: &Fact| {
            f.leftover.is_some() && f.exists && want.as_ref().is_none_or(|w| w.contains(&f.id))
        };
        let rows: Vec<Fact> = self
            .facts_of(&input.project)
            .into_iter()
            .filter(keep)
            .collect();
        let missing = want
            .iter()
            .flatten()
            .filter(|id| !rows.iter().any(|f| &f.id == *id));
        let mut skipped: Vec<Skipped> = missing
            .map(|id| Skipped {
                id: id.clone(),
                path: None,
                reason: "not a leftover row in the listing".to_string(),
            })
            .collect();
        let mut queued = Vec::new();
        for fact in rows {
            let rule = board
                .rules
                .iter()
                .find(|r| r.id == fact.rule && r.problem().is_none());
            match self.skip_reason(board, input, &fact, rule, min_days) {
                Some(reason) => skipped.push(Skipped {
                    id: fact.id.clone(),
                    path: Some(fact.path.clone()),
                    reason,
                }),
                None => queued.push(fact),
            }
        }
        (queued, skipped)
    }

    /// Delete the old leftovers of a project: `ids` (else every leftover row) untouched for the age limit. A thread never
    /// goes below the server's limit without `older_than_days` and a reason.
    pub fn remove_leftovers(&self, input: &BatchInput) -> Result<BatchOutcome> {
        let board = self.board(&input.project)?;
        let single = as_remove(input, "");
        if let Err(err) = check_caller(&board, &input.caller, &board.valid_rules()) {
            self.refuse_batch(input, &err.message);
            return Err(err);
        }
        let min_days = self
            .age_limit(&single)
            .inspect_err(|err| self.refuse_batch(input, &err.message))?;
        let (queued, skipped) = self.split_rows(&board, input, min_days);
        let (text, size_mb, files) = Self::batch_text(&queued, &skipped);
        let jobs = self.queue_batch(input, &queued, min_days);
        let (who, why) = Self::who_and_why(&single);
        let level = if queued.is_empty() { "warn" } else { "info" };
        self.note(
            &input.project,
            level,
            &format!("{text} {who}{why}{}", skipped_tail(&skipped)),
            input.caller.thread(),
        );
        Ok(BatchOutcome {
            started: queued.len(),
            skipped,
            size_mb,
            files,
            jobs,
            older_than_days: min_days,
            text,
        })
    }

    fn queue_batch(&self, input: &BatchInput, queued: &[Fact], min_days: f64) -> Vec<JobDoc> {
        let tally = Arc::new(Mutex::new(Tally {
            left: queued.len(),
            ..Tally::default()
        }));
        let by = input.caller.by_text();
        let mut docs = Vec::new();
        for fact in queued {
            let id = self.new_job(&input.project, fact, &by, JobKind::Leftover);
            let leftover = fact.leftover.clone().unwrap_or_default();
            self.update_job(&id, |j| {
                (j.size_mb, j.files, j.batch) = (leftover.size_mb, leftover.files, true);
            });
            docs.extend(self.job(&id).map(|j| j.doc()));
            let (service, tally, one) = (
                self.clone(),
                tally.clone(),
                RemoveInput {
                    older_than_days: Some(min_days),
                    confirm_young: input.confirm_young,
                    reason: input.reason.clone(),
                    job: Some(id),
                    ..RemoveInput::new(&input.project, &fact.id, input.caller.clone())
                },
            );
            self.queue
                .push(Box::new(move || service.run_batch_job(&one, &tally)));
        }
        docs
    }

    fn run_batch_job(&self, one: &RemoveInput, tally: &Arc<Mutex<Tally>>) {
        let id = one.job.clone().unwrap_or_default();
        self.update_job(&id, |j| j.state = JobState::Checking);
        if let Err(err) = self.guarded(one, false, |trace| self.run(one, trace)) {
            let finished = crate::clock::iso_from_ms(self.now());
            self.update_job(&id, |j| {
                (j.state, j.error, j.finished_at) = (
                    JobState::Refused,
                    Some(err.message.chars().take(300).collect()),
                    Some(finished),
                );
            });
            trim_jobs(&mut self.st().jobs);
            crate::sync::locked(tally).refused += 1;
        } else {
            crate::sync::locked(tally).started += 1;
            self.run_delete(&id);
        }
        let done = {
            let mut t = crate::sync::locked(tally);
            t.left -= 1;
            (t.left == 0).then(|| (t.started, t.refused))
        };
        if let Some((started, refused)) = done {
            let (who, _) = Self::who_and_why(one);
            self.note(
                &one.project,
                "info",
                &format!(
                    "leftover batch finished: {started} deleted or started, {refused} refused {who}"
                ),
                one.caller.thread(),
            );
            self.request_rescan(&one.project);
        }
    }
}

/// `. Skipped: <the first four distinct reasons>`, or nothing.
fn skipped_tail(skipped: &[Skipped]) -> String {
    let mut reasons: Vec<&str> = Vec::new();
    for reason in skipped.iter().map(|s| s.reason.as_str()) {
        if !reasons.contains(&reason) {
            reasons.push(reason);
        }
    }
    if reasons.is_empty() {
        return String::new();
    }
    format!(
        ". Skipped: {}",
        reasons
            .iter()
            .take(4)
            .copied()
            .collect::<Vec<_>>()
            .join("; ")
    )
}

fn as_remove(input: &BatchInput, id: &str) -> RemoveInput {
    RemoveInput {
        older_than_days: input.older_than_days,
        confirm_young: input.confirm_young,
        reason: input.reason.clone(),
        ..RemoveInput::new(&input.project, id, input.caller.clone())
    }
}
