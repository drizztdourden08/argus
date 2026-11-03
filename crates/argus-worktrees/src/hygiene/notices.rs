//! Two passive notices to a thread: a worktree with no item an hour after Argus first saw it (`worktree-unlinked`,
//! once; it is never removed for that alone), and a worktree of a closed item that stays because something Argus
//! cannot undo keeps it (`worktree-stale`, once per reason).

use super::memory::{HygieneState, Memory};
use crate::board::BoardFacts;
use crate::fact::Fact;
use crate::linking::UNLINKED_AFTER_MS;
use crate::ports::WakeEvent;
use crate::service::Service;
use crate::status::MAX_TEXT;
use crate::waking::{REASON as STALE_REASON, UNLINKED_REASON};
use std::collections::{BTreeMap, BTreeSet};

/// Worktrees named in one wake.
const LISTED: usize = 2;

/// A worktree to tell a thread about.
struct Notice {
    id: String,
    path: String,
    thread: String,
    why: String,
}

/// A worktree with no item.
struct Loose {
    id: String,
    path: String,
    thread: Option<String>,
}

fn clip(text: &str) -> String {
    text.chars().take(MAX_TEXT).collect()
}

fn listed(list: &[&Notice], cell: impl Fn(&Notice) -> String) -> String {
    let shown: Vec<String> = list.iter().take(LISTED).copied().map(cell).collect();
    let more = list.len().saturating_sub(LISTED);
    let tail = if more > 0 {
        format!(" (+{more} more)")
    } else {
        String::new()
    };
    format!("{}{tail}", shown.join(", "))
}

fn unlinked_text(list: &[&Notice]) -> String {
    let (count, s) = (list.len(), if list.len() == 1 { "" } else { "s" });
    let names = listed(list, |n| format!("{} {}", n.id, n.path));
    clip(&format!(
        "{count} worktree{s} with no item an hour after Argus first saw {}: {names}. Argus never removes one for that alone. Link each to an item (coord worktree link <id> I-n), or start its agent with --item.",
        if count == 1 { "it" } else { "them" }
    ))
}

fn stuck_text(list: &[&Notice]) -> String {
    let (count, s) = (list.len(), if list.len() == 1 { "" } else { "s" });
    let names = listed(list, |n| format!("{} ({})", n.path, n.why));
    clip(&format!(
        "{count} worktree{s} of closed items stay: {names}. Argus removes {} by itself once that is solved.",
        if count == 1 { "it" } else { "them" }
    ))
}

fn events_of(
    project: &str,
    reason: &'static str,
    list: &[Notice],
    text: fn(&[&Notice]) -> String,
) -> Vec<WakeEvent> {
    let mut by_thread: BTreeMap<&str, Vec<&Notice>> = BTreeMap::new();
    for notice in list {
        by_thread
            .entry(notice.thread.as_str())
            .or_default()
            .push(notice);
    }
    by_thread
        .into_iter()
        .map(|(thread, items)| WakeEvent {
            scope: project.to_string(),
            thread: thread.to_string(),
            reason,
            subject: thread.to_string(),
            text: text(&items),
        })
        .collect()
}

/// The first-seen times and the flags after this look: a worktree with no item is flagged once it has been seen for an
/// hour, and the flag goes the moment an item links to it.
fn track(memory: &mut Memory, seen: &[&Fact], loose: &[Loose], now: i64) {
    let alive: BTreeSet<&String> = seen.iter().map(|f| &f.id).collect();
    memory.seen.retain(|id, _| alive.contains(id));
    for fact in seen {
        memory.seen.entry(fact.id.clone()).or_insert(now);
    }
    let flagged: BTreeSet<&String> = loose
        .iter()
        .map(|l| &l.id)
        .filter(|id| {
            memory
                .seen
                .get(*id)
                .is_some_and(|at| now - at >= UNLINKED_AFTER_MS)
        })
        .collect();
    memory.unlinked.retain(|id, _| flagged.contains(id));
    for id in flagged {
        memory.unlinked.entry(id.clone()).or_insert(false);
    }
    let staying = stuck_now(memory);
    memory.stuck.retain(|id, _| staying.contains_key(id));
}

/// The marks in review whose item is closed, with the reason.
fn stuck_now(memory: &Memory) -> BTreeMap<String, (String, String)> {
    memory
        .marks
        .iter()
        .filter(|(_, m)| m.state == HygieneState::Review && m.item.is_some())
        .filter_map(|(id, m)| {
            let thread = m.thread.clone()?;
            Some((id.clone(), (thread, m.why.clone()?)))
        })
        .collect()
}

/// The flagged worktrees whose thread was not told, with their thread.
fn untold_unlinked(memory: &Memory, loose: &[Loose]) -> Vec<Notice> {
    loose
        .iter()
        .filter(|l| memory.unlinked.get(&l.id) == Some(&false))
        .filter_map(|l| {
            Some(Notice {
                id: l.id.clone(),
                path: l.path.clone(),
                thread: l.thread.clone()?,
                why: String::new(),
            })
        })
        .collect()
}

/// The staying worktrees of closed items whose reason their thread was not told.
fn untold_stuck(memory: &Memory) -> Vec<Notice> {
    stuck_now(memory)
        .into_iter()
        .filter(|(id, (_, why))| memory.stuck.get(id) != Some(why))
        .map(|(id, (thread, why))| Notice {
            path: memory
                .marks
                .get(&id)
                .map(|m| m.path.clone())
                .unwrap_or_default(),
            id,
            thread,
            why,
        })
        .collect()
}

impl Service {
    /// The worktrees with no item, as the board and the last scan show them.
    fn loose_of(&self, board: &BoardFacts, seen: &[&Fact]) -> Vec<Loose> {
        seen.iter()
            .filter(|f| self.link_of(board, f).is_none())
            .map(|f| Loose {
                id: f.id.clone(),
                path: f.path.clone(),
                thread: Self::told_thread(board, f, None),
            })
            .collect()
    }

    /// Flag what is due and tell each thread once. Says how many wakes went and whether the memory changed.
    pub(super) fn notice_items(&self, project: &str, board: &BoardFacts) -> (usize, bool) {
        let facts = self.facts_of(project);
        let seen: Vec<&Fact> = facts
            .iter()
            .filter(|f| Self::judged(f) && !f.main)
            .collect();
        let loose = self.loose_of(board, &seen);
        let now = self.now();
        let (unlinked, stuck, changed) = self.with_hygiene(project, |memory| {
            let before = (
                memory.seen.clone(),
                memory.unlinked.clone(),
                memory.stuck.clone(),
            );
            track(memory, &seen, &loose, now);
            let after = (
                memory.seen.clone(),
                memory.unlinked.clone(),
                memory.stuck.clone(),
            );
            (
                untold_unlinked(memory, &loose),
                untold_stuck(memory),
                before != after,
            )
        });
        let mut events = events_of(project, UNLINKED_REASON, &unlinked, unlinked_text);
        events.extend(events_of(project, STALE_REASON, &stuck, stuck_text));
        if events.is_empty() || self.deps.notifier.wake(&events).is_err() {
            return (0, changed);
        }
        self.with_hygiene(project, |memory| {
            for notice in &unlinked {
                memory.unlinked.insert(notice.id.clone(), true);
            }
            for notice in &stuck {
                memory.stuck.insert(notice.id.clone(), notice.why.clone());
            }
        });
        self.log_wakes(project, &events);
        (events.len(), true)
    }
}
