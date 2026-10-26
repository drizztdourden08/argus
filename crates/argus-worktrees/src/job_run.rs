//! Running deletion jobs: one folder at a time, each top-level entry through the junction-safe walker.

use crate::clock::iso_from_ms;
use crate::deleting::{is_deleting, original_name};
use crate::doc::WorktreeDoc;
use crate::error::io_code;
use crate::fact::{Fact, blank_fact};
use crate::jobs::{Job, JobDoc, JobKind, JobState, trim_jobs};
use crate::measure::{Leftover, about_text};
use crate::paths::{base_name, clean_path, join_path, parent_of, same_path};
use crate::rule::{Rule, RuleKind};
use crate::service::Service;
use crate::status::{Judging, doc_of};
use crate::tree::{remove_entry, unlink_link};
use std::io;

#[derive(Default)]
struct Failures {
    count: u64,
    first: Option<String>,
}

impl Failures {
    fn add(&mut self, name: &str, err: &io::Error) {
        self.count += 1;
        if self.first.is_none() {
            self.first = Some(format!("{name}: {}", io_code(err)));
        }
    }

    fn text(&self) -> Option<String> {
        let first = self.first.as_ref()?;
        Some(if self.count > 1 {
            format!("{first} (and {} more)", self.count - 1)
        } else {
            first.clone()
        })
    }
}

/// git's sentence without the long path: `error: failed to delete 'X:/a/b': Directory not empty` -> `Directory not empty`.
fn git_said(text: &str) -> String {
    let rest = text.rfind("': ").map_or(text, |at| &text[at + 3..]);
    rest.chars().take(120).collect()
}

impl Service {
    /// The row a job keeps while it deletes the folder.
    pub(crate) fn doc_for(&self, project: &str, fact: &Fact) -> Option<WorktreeDoc> {
        let board = self.deps.store.board(project)?;
        let rule = board.rules.iter().find(|r| r.id == fact.rule);
        Some(doc_of(
            fact,
            &Judging {
                rule,
                keep: None,
                now_ms: self.now(),
                default_stale_days: Some(self.opts.stale_days),
            },
            None,
        ))
    }

    /// Register a job. Returns its id (`L-n`).
    pub(crate) fn new_job(&self, project: &str, fact: &Fact, by: &str, kind: JobKind) -> String {
        let row = self.doc_for(project, fact);
        let mut state = self.st();
        state.job_seq += 1;
        let id = format!("L-{}", state.job_seq);
        let doc = JobDoc::queued(&id, &fact.id, &fact.path, by, iso_from_ms(self.now()));
        state.jobs.push(Job {
            doc,
            project: project.to_string(),
            row,
            batch: false,
            kind,
            git_error: None,
        });
        id
    }

    pub(crate) fn update_job(&self, id: &str, change: impl FnOnce(&mut Job)) {
        if let Some(job) = self.st().jobs.iter_mut().find(|j| j.id == id) {
            change(job);
        }
    }

    pub(crate) fn job(&self, id: &str) -> Option<Job> {
        self.st().jobs.iter().find(|j| j.id == id).cloned()
    }

    /// The job that is deleting this row, if any.
    pub(crate) fn active_job_for(&self, project: &str, row_id: &str) -> Option<Job> {
        self.st()
            .jobs
            .iter()
            .rev()
            .find(|j| j.project == project && j.row_id == row_id && j.state.active())
            .cloned()
    }

    /// Run a job after the ones queued before it.
    pub(crate) fn enqueue_delete(&self, id: &str) {
        let (service, id) = (self.clone(), id.to_string());
        self.queue.push(Box::new(move || service.run_delete(&id)));
    }

    fn delete_folder(&self, id: &str, folder: &str) -> Failures {
        let fs = self.deps.fs.as_ref();
        let mut failures = Failures::default();
        match fs.lstat(folder) {
            Err(err) if err.kind() == io::ErrorKind::NotFound => return failures,
            Err(err) => {
                failures.add(&base_name(folder), &err);
                return failures;
            }
            Ok(stat) if stat.is_link => {
                match unlink_link(fs, folder) {
                    Ok(()) => self.update_job(id, |j| (j.removed, j.total) = (1, Some(1))),
                    Err(err) => failures.add(&base_name(folder), &err),
                }
                return failures;
            }
            Ok(_) => {}
        }
        let entries = match fs.read_dir(folder) {
            Ok(entries) => entries,
            Err(err) => {
                failures.add(&base_name(folder), &err);
                return failures;
            }
        };
        let total = u64::try_from(entries.len()).unwrap_or(u64::MAX);
        self.update_job(id, |j| j.total = Some(total));
        for entry in entries {
            match remove_entry(
                fs,
                &join_path(folder, &entry.name),
                self.opts.drop_links_first,
            ) {
                Ok(()) => self.update_job(id, |j| j.removed += 1),
                Err(err) => failures.add(&entry.name, &err),
            }
        }
        if failures.count == 0 {
            if let Err(err) = fs.remove_dir(folder).or_else(|e| {
                if e.kind() == io::ErrorKind::NotFound {
                    Ok(())
                } else {
                    Err(e)
                }
            }) {
                failures.add(&base_name(folder), &err);
            }
        }
        failures
    }

    fn report_deleted(&self, job: &Job, failures: &Failures, secs: i64) {
        let what = if job.kind == JobKind::Worktree {
            "worktree folder"
        } else {
            "leftover folder"
        };
        let total = job.total.map_or_else(|| "?".to_string(), |t| t.to_string());
        if failures.count > 0 {
            let error = failures.text().unwrap_or_default();
            let text = format!(
                "deleting {what} {} failed after {} of {total} entries: {error}. It is renamed {} and is finished at the next start ({})",
                job.path,
                job.removed,
                base_name(&job.folder),
                job.by
            );
            self.note(&job.project, "error", &text, None);
        } else if job.kind == JobKind::Worktree {
            let said = job
                .git_error
                .as_deref()
                .map(|g| format!(", it said: {}", git_said(g)))
                .unwrap_or_default();
            let text = format!(
                "deleted worktree folder {} (finished by Argus: git could not delete it{said}; {} top-level entries, {secs} s) for {}",
                job.path,
                job.total.unwrap_or(0),
                job.by
            );
            self.note(&job.project, "info", &text, None);
        } else {
            let size = job
                .size_mb
                .map(|mb| format!(", {}", about_text(Some(mb))))
                .unwrap_or_default();
            let files = job.files.map_or_else(|| "?".to_string(), |f| f.to_string());
            let text = format!(
                "deleted leftover folder {} ({} top-level entries, {files} files{size}; {secs} s) for {}",
                job.path,
                job.total.unwrap_or(0),
                job.by
            );
            self.note(&job.project, "info", &text, None);
        }
    }

    /// The deletion itself, in the background. Each top-level entry goes through the walker (links as links, never
    /// followed); the renamed folder last.
    pub(crate) fn run_delete(&self, id: &str) {
        let started = self.now();
        self.update_job(id, |j| j.state = JobState::Running);
        let Some(job) = self.job(id) else {
            return;
        };
        let failures = self.delete_folder(id, &job.folder);
        let finished = iso_from_ms(self.now());
        self.update_job(id, |j| {
            j.finished_at = Some(finished);
            j.state = if failures.count > 0 {
                JobState::Failed
            } else {
                JobState::Done
            };
            j.error = failures.text();
        });
        trim_jobs(&mut self.st().jobs);
        if let Some(done) = self.job(id) {
            self.report_deleted(
                &done,
                &failures,
                ((self.now() - started).max(0) + 500) / 1000,
            );
            if !done.batch {
                self.request_rescan(&done.project);
            }
        }
    }

    /// A deletion a crash cut short: the folder is already renamed `<name>.argus-deleting-<ms>` under a current rule.
    /// Finish it with a job. None when it is not such a folder.
    pub(crate) fn finish_deleting(&self, project: &str, dir: &str, rule: &Rule) -> Option<String> {
        let path = clean_path(dir);
        let name = base_name(&path);
        let ci = self.ci();
        if !is_deleting(&name)
            || rule.problem().is_some()
            || rule.kind != RuleKind::Folder
            || !same_path(&rule.path, &parent_of(&path), ci)
        {
            return None;
        }
        let original = join_path(&parent_of(&path), &original_name(&name));
        if !rule.allows(&base_name(&original), ci)
            || self.st().jobs.iter().any(|j| {
                j.project == project && j.state.active() && same_path(&j.folder, &path, ci)
            })
        {
            return None;
        }
        let mut fact = blank_fact(&original, rule, ci);
        fact.leftover = Some(Leftover::default());
        let id = self.new_job(project, &fact, "startup", JobKind::Leftover);
        self.update_job(&id, |j| j.folder.clone_from(&path));
        self.note(
            project,
            "info",
            &format!("finishing a deletion a crash cut short: {path}"),
            None,
        );
        self.enqueue_delete(&id);
        Some(id)
    }
}
