//! The stale signal (`worktree-stale`) and the state kept about live worktrees only.
//!
//! ONE wake per thread per scan, listing the worktrees that need a decision and are due, worst first. Due: new in a
//! needing status, its status changed, or `repeat_ms` since it was reported. What was told is kept in the key-value
//! store (`worktrees:sent:<project>`), so a restart does not tell the thread again.

use crate::board::BoardFacts;
use crate::doc::{Derived, WorktreeDoc};
use crate::ports::{ActivityLine, WakeEvent};
use crate::service::Service;
use crate::status::{by_worst, many_text, wake_text};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, HashMap};

/// The wake reason for a worktree that needs a decision.
pub const REASON: &str = "worktree-stale";
/// The wake reason for a worktree that has had no item for an hour (a passive notice, once).
pub const UNLINKED_REASON: &str = "worktree-unlinked";
/// The key of the "too many worktrees" sentence in the sent map.
pub const MANY_KEY: &str = "__many";

/// The key-value key of what was told about a project.
pub fn sent_key(project: &str) -> String {
    format!("worktrees:sent:{project}")
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
struct Sent {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    s: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    n: Option<usize>,
    #[serde(default)]
    a: i64,
}

type SentMap = HashMap<String, Sent>;

impl Service {
    /// One activity line per wake just sent, so the log says whom it woke and why.
    pub(crate) fn log_wakes(&self, project: &str, events: &[WakeEvent]) {
        for event in events {
            self.deps.notifier.activity(ActivityLine {
                kind: event.reason,
                level: "info",
                text: format!("{project}/{}: {}", event.thread, event.text),
                project: project.to_string(),
                thread: Some(event.thread.clone()),
            });
        }
    }

    fn load_sent(&self, project: &str) -> SentMap {
        let text = self
            .deps
            .store
            .kv_get(&sent_key(project))
            .unwrap_or_default();
        serde_json::from_str(&text).unwrap_or_default()
    }

    /// The thread told about a worktree: the rule's, the project's lead when that thread has ended.
    fn target_of(board: &BoardFacts, doc: &WorktreeDoc) -> Option<String> {
        doc.thread
            .as_deref()
            .and_then(|t| board.live_thread(t))
            .or_else(|| board.coordinator())
    }

    fn collect_due<'a>(
        &self,
        board: &BoardFacts,
        docs: &'a [WorktreeDoc],
        sent: &SentMap,
        next: &mut SentMap,
    ) -> BTreeMap<String, Vec<&'a WorktreeDoc>> {
        let now = self.now();
        let mut due: BTreeMap<String, Vec<&WorktreeDoc>> = BTreeMap::new();
        for doc in docs.iter().filter(|d| d.status.needs_decision()) {
            let prev = sent.get(&doc.id);
            if let Some(prev) = prev.filter(|p| {
                p.s.as_deref() == Some(doc.status.as_str()) && now - p.a < self.opts.repeat_ms
            }) {
                next.insert(doc.id.clone(), prev.clone());
                continue;
            }
            let Some(thread) = Self::target_of(board, doc) else {
                if let Some(prev) = prev {
                    next.insert(doc.id.clone(), prev.clone());
                }
                continue;
            };
            next.insert(
                doc.id.clone(),
                Sent {
                    s: Some(doc.status.as_str().to_string()),
                    n: None,
                    a: now,
                },
            );
            due.entry(thread).or_default().push(doc);
        }
        due
    }

    /// Who hears "this project has a lot of worktrees", if it is due: the lead, else the first live rule thread.
    fn many_target(
        &self,
        board: &BoardFacts,
        derived: &Derived,
        sent: &SentMap,
        next: &mut SentMap,
    ) -> Option<String> {
        if !derived.meta.too_many {
            return None;
        }
        let now = self.now();
        let prev = sent.get(MANY_KEY);
        let target = if prev.is_some_and(|p| now - p.a < self.opts.repeat_ms) {
            None
        } else {
            board.coordinator().or_else(|| {
                board
                    .rules
                    .iter()
                    .find_map(|r| r.thread.as_deref().and_then(|t| board.live_thread(t)))
            })
        };
        if target.is_some() {
            next.insert(
                MANY_KEY.to_string(),
                Sent {
                    s: None,
                    n: Some(derived.meta.count),
                    a: now,
                },
            );
        } else if let Some(prev) = prev {
            next.insert(MANY_KEY.to_string(), prev.clone());
        }
        target
    }

    /// Tell the threads what needs a decision. Returns how many wakes were queued.
    pub fn wake_step(&self, project: &str) -> usize {
        let Some(board) = self.deps.store.board(project) else {
            return 0;
        };
        let derived = self.derive_with(project, &board);
        let sent = self.load_sent(project);
        let mut next = SentMap::new();
        let mut due = self.collect_due(&board, &derived.docs, &sent, &mut next);
        let many_to = self.many_target(&board, &derived, &sent, &mut next);
        if let Some(thread) = &many_to {
            due.entry(thread.clone()).or_default();
        }
        let name = derived.meta.name.as_str();
        let events: Vec<WakeEvent> = due
            .into_iter()
            .map(|(thread, mut list)| {
                list.sort_by(|a, b| by_worst(a, b));
                let prefix = if many_to.as_deref() == Some(thread.as_str()) {
                    let meta = &derived.meta;
                    many_text(name, meta.count, meta.git, meta.leftovers, meta.leftover_mb)
                } else {
                    String::new()
                };
                WakeEvent {
                    scope: project.to_string(),
                    subject: thread.clone(),
                    text: wake_text(&list, &prefix),
                    thread,
                    reason: REASON,
                }
            })
            .collect();
        if !events.is_empty() {
            if self.deps.notifier.wake(&events).is_err() {
                return 0;
            }
            self.log_wakes(project, &events);
        }
        let _ = self.deps.store.kv_set(
            &sent_key(project),
            &serde_json::to_string(&next).unwrap_or_default(),
        );
        events.len()
    }

    /// After a clean scan: the keeps and the answers of worktrees that are not on disk any more leave the board.
    /// Returns the ids dropped, keeps first.
    pub fn prune_state(&self, project: &str) -> (Vec<String>, Vec<String>) {
        let Some(board) = self.deps.store.board(project) else {
            return (Vec::new(), Vec::new());
        };
        let alive: Vec<String> = {
            let state = self.st();
            match state.scans.get(project) {
                Some(scan)
                    if scan.error.is_none()
                        && !scan
                            .rules
                            .iter()
                            .any(|r| r.error.is_some() && r.invalid.is_none()) =>
                {
                    scan.facts
                        .iter()
                        .filter(|f| f.exists)
                        .map(|f| f.id.clone())
                        .collect()
                }
                _ => return (Vec::new(), Vec::new()),
            }
        };
        let mut keeps: Vec<String> = board
            .keeps
            .keys()
            .filter(|id| !alive.contains(id))
            .cloned()
            .collect();
        let mut answers: Vec<String> = board
            .answers
            .keys()
            .filter(|entry| {
                !board
                    .entries
                    .get(*entry)
                    .and_then(|e| e.worktree.as_ref())
                    .is_some_and(|w| alive.contains(&w.id))
            })
            .cloned()
            .collect();
        keeps.sort();
        answers.sort();
        if keeps.is_empty() && answers.is_empty() {
            return (keeps, answers);
        }
        match self.deps.store.drop_kept(project, &keeps, &answers) {
            Ok(()) => (keeps, answers),
            Err(_) => (Vec::new(), Vec::new()),
        }
    }
}
