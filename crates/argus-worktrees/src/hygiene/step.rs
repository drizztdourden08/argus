//! One step of the hygiene: judge every worktree, tell the threads what was released, and hand the ones whose grace
//! ended to the request worker.

use super::judge::{Look, Owner, Verdict};
use super::memory::{HygieneState, Mark, Memory};
use super::text::{REASON, review_text};
use crate::board::{BoardFacts, Caller};
use crate::fact::Fact;
use crate::ports::{ActivityLine, WakeEvent};
use crate::requests::RequestInput;
use crate::service::Service;
use std::collections::BTreeMap;

/// What one step did.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Stepped {
    /// The `worktree-review` wakes that went.
    pub told: usize,
    /// A row changed: the docs are written again.
    pub changed: bool,
}

/// The ids of one thread's released worktrees.
type ByThread = BTreeMap<String, Vec<String>>;

fn state_of(verdict: &Verdict) -> Option<(HygieneState, Option<String>)> {
    match verdict {
        Verdict::Quiet => None,
        Verdict::InUse(why) => Some((HygieneState::InUse, Some(why.clone()))),
        Verdict::Released => Some((HygieneState::Released, None)),
        Verdict::Kept(note) => Some((HygieneState::Kept, note.clone())),
        Verdict::Review(why) => Some((HygieneState::Review, Some(why.clone()))),
    }
}

/// The new mark of a worktree; a released one keeps when it was released, told and handed on.
fn mark_of(
    fact: &Fact,
    verdict: &Verdict,
    owner: Owner,
    prev: Option<&Mark>,
    now: i64,
) -> Option<Mark> {
    let (state, why) = state_of(verdict)?;
    let same = prev.filter(|p| p.state == state);
    let released = same.filter(|_| state == HygieneState::Released);
    Some(Mark {
        state,
        path: fact.path.clone(),
        branch: fact.branch.clone(),
        thread: owner.thread,
        agent: owner.agent,
        why,
        since: same.map_or(now, |p| p.since),
        told: released.and_then(|p| p.told),
        request: released.and_then(|p| p.request.clone()),
    })
}

/// The released marks of a thread that pass `due`, by thread.
fn released_by(memory: &Memory, due: impl Fn(&Mark) -> bool) -> ByThread {
    let mut out = ByThread::new();
    for (id, mark) in &memory.marks {
        if mark.state != HygieneState::Released || !due(mark) {
            continue;
        }
        if let Some(thread) = &mark.thread {
            out.entry(thread.clone()).or_default().push(id.clone());
        }
    }
    out
}

impl Service {
    /// The verdicts of every worktree of the last scan.
    fn verdicts(
        &self,
        project: &str,
        board: &BoardFacts,
        scanned_ms: i64,
    ) -> Vec<(Fact, Verdict, Owner)> {
        let runs = self.deps.store.live_runs();
        let refused = self.with_hygiene(project, |m| m.refused.clone());
        self.facts_of(project)
            .into_iter()
            .map(|fact| {
                let look = Look {
                    board,
                    runs: &runs,
                    scanned_ms,
                    fresh: false,
                    refused: refused.get(&fact.id),
                };
                let (verdict, owner) = self.judge(project, &look, &fact);
                (fact, verdict, owner)
            })
            .collect()
    }

    /// The last clean scan's time, when there is one.
    fn clean_scan_ms(&self, project: &str) -> Option<i64> {
        let state = self.st();
        let scan = state.scans.get(project)?;
        let failed = scan.error.is_some()
            || scan
                .rules
                .iter()
                .any(|r| r.error.is_some() && r.invalid.is_none());
        (!failed).then_some(scan.at_ms)
    }

    /// One step: judge, tell, hand on. Says how many wakes went and whether a row changed.
    pub fn hygiene_step(&self, project: &str) -> Stepped {
        if !self.opts.release || self.stopped() {
            return Stepped::default();
        }
        let (Some(board), Some(scanned_ms)) =
            (self.deps.store.board(project), self.clean_scan_ms(project))
        else {
            return Stepped::default();
        };
        let judged = self.verdicts(project, &board, scanned_ms);
        let alive: Vec<String> = judged.iter().map(|(f, _, _)| f.id.clone()).collect();
        let now = self.now();
        let changed = self.with_hygiene(project, |memory| {
            let before = std::mem::take(&mut memory.marks);
            let marks = judged
                .into_iter()
                .filter_map(|(fact, verdict, owner)| {
                    let id = fact.id.clone();
                    mark_of(&fact, &verdict, owner, before.get(&id), now).map(|m| (id, m))
                })
                .collect();
            memory.marks = marks;
            memory.refused.retain(|id, _| alive.contains(id));
            memory.marks != before
        });
        let told = self.tell_released(project);
        self.hand_on(project, &board);
        Stepped {
            told,
            changed: changed || told > 0,
        }
    }

    /// ONE `worktree-review` wake per thread for what it was not told yet, once nothing new came for the debounce.
    fn tell_released(&self, project: &str) -> usize {
        let now = self.now();
        let quiet = self.opts.release_debounce_ms;
        let (events, ids) = self.with_hygiene(project, |memory| {
            let untold = released_by(memory, |m| m.told.is_none());
            let mut events = Vec::new();
            let mut ids = Vec::new();
            for (thread, list) in untold {
                let newest = list
                    .iter()
                    .filter_map(|id| memory.marks.get(id))
                    .map(|m| m.since)
                    .max()
                    .unwrap_or(now);
                if now - newest < quiet {
                    continue;
                }
                let shown: Vec<(&String, &Mark)> = list
                    .iter()
                    .filter_map(|id| memory.marks.get(id).map(|m| (id, m)))
                    .collect();
                events.push(WakeEvent {
                    scope: project.to_string(),
                    thread: thread.clone(),
                    reason: REASON,
                    subject: thread.clone(),
                    text: review_text(&shown, self.opts.release_grace_ms),
                });
                ids.extend(list);
            }
            (events, ids)
        });
        if events.is_empty() || self.deps.notifier.wake(&events).is_err() {
            return 0;
        }
        self.with_hygiene(project, |memory| {
            for id in &ids {
                if let Some(mark) = memory.marks.get_mut(id) {
                    mark.told = Some(now);
                }
            }
        });
        for event in &events {
            self.deps.notifier.activity(ActivityLine {
                kind: REASON,
                level: "info",
                text: format!("{project}/{}: {}", event.thread, event.text),
                project: project.to_string(),
                thread: Some(event.thread.clone()),
            });
        }
        events.len()
    }

    /// The released worktrees whose grace ended go to the request worker: one request per thread.
    fn hand_on(&self, project: &str, board: &BoardFacts) {
        let now = self.now();
        let grace = self.opts.release_grace_ms;
        let due = self.with_hygiene(project, |memory| {
            released_by(memory, |m| {
                m.request.is_none() && m.told.is_some_and(|t| now - t >= grace)
            })
        });
        for (thread, ids) in due {
            if board.live_thread(&thread).is_none() {
                continue;
            }
            let mut input = RequestInput::new(project, Caller::Thread(thread.clone()));
            input.targets.clone_from(&ids);
            input.branch = true;
            input.reason =
                Some("released: no running agent owns it and nothing uses it".to_string());
            let Ok(queued) = self.request_released(&input) else {
                // The thread may not remove it (the request is refused and logged): it stays released, told.
                continue;
            };
            self.with_hygiene(project, |memory| {
                for id in &ids {
                    if let Some(mark) = memory.marks.get_mut(id) {
                        mark.request = Some(queued.id.clone());
                    }
                }
            });
        }
    }
}
