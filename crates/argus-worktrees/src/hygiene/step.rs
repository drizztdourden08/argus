//! One step of the hygiene: judge every worktree, mark it, remove the clean ones nothing is attached to, ask the lead
//! about the ones that hold work, and tell the Overlord of the asks left unanswered.

use super::judge::{Look, Owner, Verdict};
use super::memory::{HygieneState, Mark};
use crate::board::{BoardFacts, Caller};
use crate::fact::Fact;
use crate::requests::{Mode, RequestInput};
use crate::service::Service;
use std::collections::{BTreeMap, HashSet};

/// What one step did.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Stepped {
    /// The approval asks to leads and the reports to the Overlord that went.
    pub told: usize,
    /// The `worktree-unlinked` wakes of worktrees with no item.
    pub noticed: usize,
    /// A row changed: the docs are written again.
    pub changed: bool,
}

/// The state and the sentence a verdict gives.
fn state_of(verdict: &Verdict) -> Option<(HygieneState, Option<String>)> {
    match verdict {
        Verdict::Quiet => None,
        Verdict::Attached(why) => Some((HygieneState::Attached, Some(why.clone()))),
        Verdict::Exception(reason) => Some((HygieneState::Exception, Some(reason.clone()))),
        Verdict::Clean => Some((HygieneState::Removing, None)),
        Verdict::Left(what) => Some((HygieneState::AwaitingLead, Some(what.clone()))),
    }
}

/// The new mark of a worktree. While nothing is attached it keeps when that began and its request; while it still
/// holds work it keeps when its lead was asked and when the Overlord was told. A removal in flight shows `removing`.
fn mark_of(
    fact: &Fact,
    verdict: &Verdict,
    owner: Owner,
    prev: Option<&Mark>,
    now: i64,
) -> Option<Mark> {
    let (state, why) = state_of(verdict)?;
    let free = prev.filter(|p| p.state.free() && state.free());
    let left = free.filter(|_| state == HygieneState::AwaitingLead);
    let request = free.and_then(|p| p.request.clone());
    let reported = left.and_then(|p| p.reported);
    let shown = if request.is_some() {
        HygieneState::Removing
    } else if reported.is_some() {
        HygieneState::Reported
    } else {
        state
    };
    let since = free
        .or_else(|| prev.filter(|p| p.state == state))
        .map_or(now, |p| p.since);
    Some(Mark {
        state: shown,
        path: fact.path.clone(),
        branch: fact.branch.clone(),
        thread: owner.thread,
        agent: owner.agent,
        why,
        since,
        asked: left.and_then(|p| p.asked),
        reported,
        request,
    })
}

impl Service {
    /// The verdicts of every worktree of the last scan.
    pub(super) fn verdicts(
        &self,
        project: &str,
        board: &BoardFacts,
        scanned_ms: i64,
    ) -> Vec<(Fact, Verdict, Owner)> {
        let runs = self.deps.store.live_runs();
        let (refused, exceptions) =
            self.with_hygiene(project, |m| (m.refused.clone(), m.exceptions.clone()));
        let failed = self.failed_rules(project);
        self.facts_of(project)
            .into_iter()
            .filter(|fact| !failed.contains(&fact.rule))
            .map(|fact| {
                let look = Look {
                    board,
                    runs: &runs,
                    scanned_ms,
                    fresh: false,
                    refused: refused.get(&fact.id),
                    exception: exceptions.get(&fact.id),
                };
                let (verdict, owner) = self.judge(project, &look, &fact);
                (fact, verdict, owner)
            })
            .collect()
    }

    /// The rules the last scan could not read (a rule that is invalid is not an error of the scan). Their worktrees
    /// are skipped for this step; every other rule goes on.
    fn failed_rules(&self, project: &str) -> HashSet<String> {
        let state = self.st();
        state.scans.get(project).map_or_else(HashSet::new, |scan| {
            scan.rules
                .iter()
                .filter(|r| r.error.is_some() && r.invalid.is_none())
                .map(|r| r.id.clone())
                .collect()
        })
    }

    /// The last scan's time, when the scan as a whole worked (a rule that failed only skips its own worktrees).
    pub(super) fn clean_scan_ms(&self, project: &str) -> Option<i64> {
        let state = self.st();
        let scan = state.scans.get(project)?;
        scan.error.is_none().then_some(scan.at_ms)
    }

    /// Mark every worktree of the last scan whose rule was read. True when a mark changed.
    fn mark_all(&self, project: &str, board: &BoardFacts, scanned_ms: i64) -> bool {
        let alive: Vec<String> = self.facts_of(project).into_iter().map(|f| f.id).collect();
        let now = self.now();
        self.with_hygiene(project, |memory| memory.prune_exceptions(&alive, now));
        let judged = self.verdicts(project, board, scanned_ms);
        self.with_hygiene(project, |memory| {
            let before = std::mem::take(&mut memory.marks);
            memory.marks = judged
                .into_iter()
                .filter(|(fact, ..)| !self.reports_only(board, &fact.rule))
                .filter_map(|(fact, verdict, owner)| {
                    let id = fact.id.clone();
                    mark_of(&fact, &verdict, owner, before.get(&id), now).map(|m| (id, m))
                })
                .collect();
            memory.refused.retain(|id, _| alive.contains(id));
            memory.marks != before
        })
    }

    /// One step: mark, remove, ask, report. Says how many wakes went and whether a row changed.
    pub fn hygiene_step(&self, project: &str) -> Stepped {
        if !self.opts.hygiene || self.stopped() {
            return Stepped::default();
        }
        let (Some(board), Some(scanned_ms)) =
            (self.deps.store.board(project), self.clean_scan_ms(project))
        else {
            return Stepped::default();
        };
        let lifted = self.unlock_stale_claude(project, &board);
        let pruned = self.prune_gone_records(project, &board);
        let changed = self.mark_all(project, &board, scanned_ms) || lifted || pruned;
        let filed = self.remove_detached(project);
        let told = self.ask_leads(project) + self.report_overlord(project);
        let (noticed, noted) = self.notice_items(project, &board);
        Stepped {
            told,
            noticed,
            changed: changed || filed || noted || told > 0,
        }
    }

    /// The clean worktrees nothing has been attached to for the debounce go to the request worker: one request per
    /// lead, told of the result. True when a request was filed.
    fn remove_detached(&self, project: &str) -> bool {
        let now = self.now();
        let debounce = self.opts.detach_debounce_ms;
        let due = self.with_hygiene(project, |memory| {
            let mut out: BTreeMap<String, Vec<String>> = BTreeMap::new();
            for (id, mark) in &memory.marks {
                let ready = mark.state == HygieneState::Removing
                    && mark.request.is_none()
                    && now - mark.since >= debounce;
                if ready {
                    let thread = mark.thread.clone().unwrap_or_default();
                    out.entry(thread).or_default().push(id.clone());
                }
            }
            out
        });
        let mut filed = false;
        for (thread, ids) in due {
            let mut input = RequestInput::new(project, Caller::Thread(thread));
            input.targets.clone_from(&ids);
            input.branch = true;
            input.reason = Some("nothing is attached to it and nothing would be lost".to_string());
            let Ok(queued) = self.request_hygiene(&input, Mode::Detached) else {
                continue;
            };
            filed = true;
            self.with_hygiene(project, |memory| {
                for id in &ids {
                    if let Some(mark) = memory.marks.get_mut(id) {
                        mark.request = Some(queued.id.clone());
                    }
                }
            });
        }
        filed
    }
}
