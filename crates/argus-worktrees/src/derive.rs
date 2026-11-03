//! The docs and the summary of one project: the last scan's facts judged now, with the board's keeps and requests.

use crate::board::{BoardFacts, request_for};
use crate::clock::iso_from_ms;
use crate::doc::{Derived, LeftoverOld, Meta, RuleSummary, Status, WorktreeDoc};
use crate::fact::Fact;
use crate::jobs::{Job, JobState};
use crate::rule::{Rule, RuleKind};
use crate::service::Service;
use crate::status::{Judging, doc_of};
use std::collections::BTreeMap;

/// The newest requests a summary carries.
const REQUESTS_SHOWN: usize = 20;

fn round1(value: f64) -> f64 {
    (value * 10.0).round() / 10.0
}

fn size_of(doc: &WorktreeDoc) -> f64 {
    doc.leftover.as_ref().and_then(|l| l.size_mb).unwrap_or(0.0)
}

fn rule_kind(rule: &Rule) -> &'static str {
    if rule.kind == RuleKind::Repo {
        "repo"
    } else {
        "folder"
    }
}

impl Service {
    fn judged_docs(
        &self,
        project: &str,
        board: &BoardFacts,
        jobs: &[Job],
    ) -> (Vec<WorktreeDoc>, usize) {
        let now_ms = self.now();
        let mut docs: Vec<WorktreeDoc> = Vec::new();
        let mut gone = 0;
        let facts = self.facts_of(project);
        for fact in &facts {
            let Some(rule) = board
                .rules
                .iter()
                .find(|r| r.id == fact.rule)
                .filter(|r| r.problem().is_none())
            else {
                continue;
            };
            if !fact.exists {
                gone += 1;
                continue;
            }
            let judging = Judging {
                rule: Some(rule),
                keep: board.keeps.get(&fact.id),
                now_ms,
                default_stale_days: Some(self.opts.stale_days),
            };
            let mut doc = doc_of(fact, &judging, request_for(board, &fact.id));
            doc.hygiene = self.hygiene_of(project, &fact.id);
            self.mark_item(project, board, fact, &mut doc);
            if let Some(leftover) = doc.leftover.as_mut() {
                leftover.made = self.is_made(project, &fact.path);
            }
            if let Some(job) = jobs
                .iter()
                .rev()
                .find(|j| j.row_id == fact.id && j.state.active())
            {
                doc.status = Status::Deleting;
                doc.job = Some(job.doc());
            }
            docs.push(doc);
        }
        (docs, gone)
    }

    /// The item of the row, how Argus knows it, and the `no item` flag. A main checkout has none.
    fn mark_item(&self, project: &str, board: &BoardFacts, fact: &Fact, doc: &mut WorktreeDoc) {
        if !Self::judged(fact) || fact.main {
            return;
        }
        let link = self.link_of(board, fact);
        doc.link.agent = self.owners(board, fact).last().map(|a| a.id.clone());
        doc.link.link_source = link.as_ref().map(|l| l.source);
        doc.link.item = link.map(|l| l.item);
        doc.link.unlinked = self.is_flagged_unlinked(project, &fact.id);
    }

    fn renamed_rows(board: &BoardFacts, jobs: &[Job], docs: &mut Vec<WorktreeDoc>) {
        let mut latest: Vec<&Job> = Vec::new();
        for job in jobs {
            latest.retain(|j| j.row_id != job.row_id);
            latest.push(job);
        }
        for job in latest {
            let shown = job.state.active() || job.state == JobState::Failed;
            let Some(row) = job.row.as_ref().filter(|_| shown) else {
                continue;
            };
            if docs.iter().any(|d| d.id == job.row_id)
                || !board.rules.iter().any(|r| r.id == row.rule)
            {
                continue;
            }
            docs.push(WorktreeDoc {
                status: Status::Deleting,
                job: Some(job.doc()),
                ..row.clone()
            });
        }
    }

    fn rule_summaries(&self, project: &str, board: &BoardFacts) -> Vec<RuleSummary> {
        let state = self.st();
        let infos = state
            .scans
            .get(project)
            .map(|s| s.rules.clone())
            .unwrap_or_default();
        board
            .rules
            .iter()
            .map(|r| {
                let info = infos.iter().find(|i| i.id == r.id);
                let problem = r.problem();
                RuleSummary {
                    id: r.id.clone(),
                    path: r.path.clone(),
                    kind: rule_kind(r).to_string(),
                    include: r.include.clone(),
                    exclude: r.exclude.clone(),
                    invalid: problem.map(String::from),
                    thread: r.thread.clone(),
                    note: r.note.clone(),
                    stale_days: r.stale_days,
                    added_by: r.added_by.clone(),
                    added_at: r.added_at.clone(),
                    exists: info.map(|i| i.exists),
                    matched: info.map(|i| i.matched),
                    candidates: info.map(|i| i.candidates),
                    error: problem
                        .map(|p| format!("invalid: {p}"))
                        .or_else(|| info.and_then(|i| i.error.clone())),
                }
            })
            .collect()
    }

    fn leftover_old(&self, project: &str, docs: &[WorktreeDoc]) -> LeftoverOld {
        let limit = self.opts.leftover_age_days;
        let old: Vec<&WorktreeDoc> = docs
            .iter()
            .filter(|d| d.status == Status::Leftover)
            .filter(|d| {
                d.leftover.as_ref().is_some_and(|l| {
                    #[allow(clippy::cast_precision_loss)]
                    let aged = l.age_days.is_some_and(|a| a as f64 >= limit);
                    !l.pending
                        && !l.has_nested_git
                        && !l.is_link
                        && (self.is_made(project, &d.path) || aged)
                })
            })
            .collect();
        LeftoverOld {
            days: limit,
            count: old.len(),
            size_mb: round1(old.iter().map(|d| size_of(d)).sum()),
        }
    }

    /// The summary of a project: the counts of its rows, the last scan, the rules and the latest jobs.
    fn meta_of(
        &self,
        project: &str,
        board: &BoardFacts,
        docs: &[WorktreeDoc],
        gone: usize,
        jobs: &[Job],
    ) -> Meta {
        let counts = Counts::of(docs);
        let state = self.st();
        let scan = state.scans.get(project);
        let (scanned_at, took_ms, error) = (
            scan.map(|s| iso_from_ms(s.at_ms)),
            scan.map(|s| s.took_ms),
            scan.and_then(|s| s.error.clone()),
        );
        drop(state);
        Meta {
            id: "worktrees".to_string(),
            name: board.name.clone().unwrap_or_else(|| project.to_string()),
            scanned_at,
            scan_ms: self.opts.scan_ms,
            live_ms: self.opts.live_ms,
            took_ms,
            count: counts.counted,
            listed: counts.listed,
            running_agents: board.agents.iter().filter(|a| a.live()).count(),
            git: counts.counted - counts.leftovers,
            leftovers: counts.leftovers,
            leftover_mb: round1(counts.measured_mb),
            leftover_pending: counts.pending,
            leftover_old: self.leftover_old(project, docs),
            deleting: counts.by_status.get("deleting").copied().unwrap_or(0),
            jobs: jobs.iter().rev().take(20).rev().map(Job::doc).collect(),
            requests: self.requests_recent(project, REQUESTS_SHOWN),
            gone,
            many_at: self.opts.many_at,
            too_many: counts.counted > self.opts.many_at,
            by_status: counts.by_status,
            stale_days: self.opts.stale_days,
            rules: self.rule_summaries(project, board),
            hygiene: self.hygiene_meta(project),
            error,
        }
    }

    /// The docs and the summary of a project, from the last scan, the board and the jobs, judged now.
    pub(crate) fn derive_with(&self, project: &str, board: &BoardFacts) -> Derived {
        let jobs: Vec<Job> = self
            .st()
            .jobs
            .iter()
            .filter(|j| j.project == project)
            .cloned()
            .collect();
        let (mut docs, gone) = self.judged_docs(project, board, &jobs);
        Self::renamed_rows(board, &jobs, &mut docs);
        let meta = self.meta_of(project, board, &docs, gone, &jobs);
        Derived { docs, meta }
    }
}

/// How many rows there are, of what status, and how big the leftovers are.
struct Counts {
    by_status: BTreeMap<String, usize>,
    counted: usize,
    listed: usize,
    leftovers: usize,
    measured_mb: f64,
    pending: usize,
}

impl Counts {
    fn of(docs: &[WorktreeDoc]) -> Self {
        let mut by_status: BTreeMap<String, usize> = BTreeMap::new();
        for d in docs {
            *by_status.entry(d.status.as_str().to_string()).or_default() += 1;
        }
        let leftover_rows = || docs.iter().filter(|d| d.status == Status::Leftover);
        Self {
            by_status,
            counted: docs.iter().filter(|d| d.status != Status::Deleting).count(),
            listed: docs
                .iter()
                .filter(|d| !d.main && d.status != Status::Deleting)
                .count(),
            leftovers: leftover_rows().count(),
            measured_mb: leftover_rows()
                .filter(|d| d.leftover.as_ref().is_some_and(|l| !l.pending))
                .map(size_of)
                .sum(),
            pending: docs
                .iter()
                .filter(|d| d.leftover.as_ref().is_some_and(|l| l.pending))
                .count(),
        }
    }
}
