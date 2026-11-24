//! What the crate reads from a project's board, as plain records the store fills in.

use crate::clock::iso_from_ms;
use crate::rule::Rule;
use serde::Serialize;
use std::collections::{BTreeMap, HashMap};

/// The longest owner request kept on a row.
pub const REQUEST_MAX: usize = 600;

/// A snooze of the stale signal for one worktree.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Keep {
    pub until_ms: i64,
    pub note: Option<String>,
    pub by: Option<String>,
}

/// The thread's answer to an owner request.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Answer {
    pub note: String,
    pub at_ms: i64,
    pub by: Option<String>,
}

/// A thread of the board, as far as the guards care.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ThreadInfo {
    pub id: String,
    pub lead: bool,
    /// Registered and not ended, not a placeholder, not deleted.
    pub live: bool,
}

/// The `worktree` field of an owner entry.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct EntryWorktree {
    pub id: String,
    pub path: String,
    pub kind: String,
    pub message: Option<String>,
}

/// An entry of the board, as far as the owner's say and the requests care.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct OwnerEntry {
    pub id: String,
    pub author: String,
    pub via: Option<String>,
    pub kind: String,
    pub deleted: bool,
    pub created_ms: Option<i64>,
    pub text: String,
    pub worktree: Option<EntryWorktree>,
    pub reply_to: Option<String>,
    pub context: Option<String>,
    pub options: Vec<String>,
}

/// A sub-agent record of the board, as far as the worktree hygiene cares.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct AgentFact {
    pub id: String,
    /// The thread that started it.
    pub thread: Option<String>,
    pub status: String,
    /// The worktree it names: an absolute path or a folder name.
    pub worktree: Option<String>,
    /// Its Claude agent id: its own worktree is `.claude/worktrees/agent-<id>`.
    pub claude_id: Option<String>,
    /// When it ended (`finishedAt`).
    pub ended_ms: Option<i64>,
    /// The item it works for (`item`): the link of its worktree to that item.
    pub item: Option<String>,
}

impl AgentFact {
    /// Done or stopped: the agent no longer needs its worktree.
    pub fn ended(&self) -> bool {
        matches!(self.status.as_str(), "done" | "stopped")
    }

    /// Still at work (working, waiting, stop-requested).
    pub fn live(&self) -> bool {
        matches!(
            self.status.as_str(),
            "working" | "waiting" | "stop-requested"
        )
    }
}

/// Something open that works in a worktree: a board item in progress, a launcher, a bundle, a run.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Link {
    /// How a sentence names it: `item I-3`, `launcher web`, `bundle b1`, `run R-4`.
    pub what: String,
    /// The folder it works in.
    pub path: Option<String>,
    /// The branch it is about.
    pub branch: Option<String>,
    /// Free text that may name the worktree (an item's title and description).
    pub text: Option<String>,
}

/// A board item, as far as the worktrees tied to it care.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct ItemFact {
    pub id: String,
    /// Its lane (`todo`, `doing`, `review`, `done`, `dropped`).
    pub lane: String,
    /// In the trash.
    pub trashed: bool,
    /// The thread it is assigned to: the lead the hygiene asks about a worktree linked to it.
    pub thread: Option<String>,
    /// Its title and description: free text that may name a worktree.
    pub text: String,
}

impl ItemFact {
    /// Done, dropped or in the trash: the item no longer holds a worktree.
    pub fn closed(&self) -> bool {
        self.trashed || matches!(self.lane.as_str(), "done" | "dropped")
    }
}

/// An explicit link of a worktree to an item (`coord worktree link`).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct WorktreeLink {
    pub item: String,
    pub by: Option<String>,
    pub at_ms: Option<i64>,
}

/// A project's board, read as one snapshot.
#[derive(Clone, Debug, Default)]
pub struct BoardFacts {
    pub name: Option<String>,
    pub rules: Vec<Rule>,
    pub keeps: HashMap<String, Keep>,
    pub answers: HashMap<String, Answer>,
    pub threads: BTreeMap<String, ThreadInfo>,
    pub entries: HashMap<String, OwnerEntry>,
    /// The sub-agent records.
    pub agents: Vec<AgentFact>,
    /// The items in progress of a live thread, the launchers not retired and the bundles not closed.
    pub links: Vec<Link>,
    /// The items, by id.
    pub items: BTreeMap<String, ItemFact>,
    /// The explicit links of worktrees to items, by worktree id.
    pub worktree_links: HashMap<String, WorktreeLink>,
}

impl BoardFacts {
    /// The rules that can be used (a rule with a wildcard path never counts).
    pub fn valid_rules(&self) -> Vec<&Rule> {
        self.rules
            .iter()
            .filter(|r| r.problem().is_none())
            .collect()
    }

    /// The id of a live thread, or none.
    pub fn live_thread(&self, id: &str) -> Option<String> {
        self.threads
            .get(id)
            .filter(|t| t.live)
            .map(|t| t.id.clone())
    }

    /// The project's lead: the first live thread whose role is lead.
    pub fn lead(&self) -> Option<String> {
        self.threads
            .values()
            .find(|t| t.live && t.lead)
            .map(|t| t.id.clone())
    }
}

/// Who asks for a removal.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Caller {
    /// The owner's page.
    Page,
    /// A registered thread, by id.
    Thread(String),
    /// Argus itself, for a worktree nothing is attached to (the owner's rule of 2026-11-06). With `approved_by`, the
    /// lead or the Overlord approved discarding what is left on it (full backup first); without it, nothing may be lost.
    Hygiene { approved_by: Option<String> },
}

impl Caller {
    pub fn is_page(&self) -> bool {
        matches!(self, Self::Page)
    }

    /// The thread id, when a thread asks.
    pub fn thread(&self) -> Option<&str> {
        match self {
            Self::Thread(id) => Some(id),
            Self::Page | Self::Hygiene { .. } => None,
        }
    }

    /// `the owner (page)` or the thread's id, as a job records who asked.
    pub fn by_text(&self) -> String {
        match self {
            Self::Page => "the owner (page)".to_string(),
            Self::Thread(id) if id.is_empty() => "a thread".to_string(),
            Self::Thread(id) => id.clone(),
            Self::Hygiene { approved_by: None } => "the hygiene".to_string(),
            Self::Hygiene {
                approved_by: Some(by),
            } => format!("the hygiene, approved by {by}"),
        }
    }

    /// `for the owner (page)` or `for <thread>`, as an activity line ends.
    pub fn for_text(&self) -> String {
        match self {
            Self::Page => "for the owner (page)".to_string(),
            Self::Thread(id) if id.is_empty() => "for a thread".to_string(),
            Self::Thread(id) => format!("for {id}"),
            Self::Hygiene { .. } => format!("for {}", self.by_text()),
        }
    }
}

/// The thread's answer on a row.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct AnswerDoc {
    pub note: String,
    pub at: String,
    pub by: Option<String>,
}

/// The owner's newest request about one worktree.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct RequestDoc {
    pub id: String,
    pub kind: String,
    pub text: String,
    pub at: Option<String>,
    pub by: String,
    pub answer: Option<AnswerDoc>,
}

fn request_text(entry: &OwnerEntry, wt: &EntryWorktree) -> String {
    if let Some(message) = &wt.message {
        return message.clone();
    }
    let prefix = format!("[worktree {}] {}:", wt.kind, wt.path);
    entry
        .text
        .strip_prefix(&prefix)
        .map_or_else(|| entry.text.clone(), |rest| rest.trim().to_string())
}

/// The newest owner request about one worktree (by its id), with the thread's answer.
pub fn request_for(board: &BoardFacts, id: &str) -> Option<RequestDoc> {
    let best = board
        .entries
        .values()
        .filter(|e| !e.deleted && e.worktree.as_ref().is_some_and(|w| w.id == id))
        .max_by(|a, b| {
            a.created_ms
                .cmp(&b.created_ms)
                .then_with(|| b.id.cmp(&a.id))
        })?;
    let wt = best.worktree.as_ref()?;
    let kind = if wt.kind == "delete" {
        "delete"
    } else {
        "check"
    };
    let answer = board.answers.get(&best.id).map(|a| AnswerDoc {
        note: a.note.clone(),
        at: iso_from_ms(a.at_ms),
        by: a.by.clone(),
    });
    Some(RequestDoc {
        id: best.id.clone(),
        kind: kind.to_string(),
        text: request_text(best, wt).chars().take(REQUEST_MAX).collect(),
        at: best.created_ms.map(iso_from_ms),
        by: "owner".to_string(),
        answer,
    })
}

// test-tag: critical
#[cfg(test)]
mod tests {
    use super::*;

    fn entry(id: &str, kind: &str, created: i64, text: &str, message: Option<&str>) -> OwnerEntry {
        OwnerEntry {
            id: id.into(),
            author: "human".into(),
            kind: "followup".into(),
            created_ms: Some(created),
            text: text.into(),
            worktree: Some(EntryWorktree {
                id: "abc".into(),
                path: "p".into(),
                kind: kind.into(),
                message: message.map(Into::into),
            }),
            ..OwnerEntry::default()
        }
    }

    #[test]
    fn the_request_is_the_newest_entry_with_the_answer() {
        let mut board = BoardFacts::default();
        for e in [
            entry(
                "O-1",
                "delete",
                1000,
                "[worktree delete] p: old",
                Some("old one"),
            ),
            entry(
                "O-2",
                "check",
                2000,
                "[worktree check] p: still needed?",
                None,
            ),
        ] {
            board.entries.insert(e.id.clone(), e);
        }
        board.answers.insert(
            "O-2".into(),
            Answer {
                note: "kept".into(),
                at_ms: 3000,
                by: Some("worker".into()),
            },
        );
        let request = request_for(&board, "abc").unwrap();
        assert_eq!(
            (
                request.id.as_str(),
                request.kind.as_str(),
                request.text.as_str()
            ),
            ("O-2", "check", "still needed?")
        );
        assert_eq!(request.answer.unwrap().note, "kept");
        assert!(request_for(&board, "other").is_none());
    }

    #[test]
    fn the_lead_is_the_first_live_one() {
        let mut board = BoardFacts::default();
        for (id, lead, live) in [("a", true, false), ("b", true, true), ("c", false, true)] {
            board.threads.insert(
                id.into(),
                ThreadInfo {
                    id: id.into(),
                    lead,
                    live,
                },
            );
        }
        assert_eq!(board.lead().as_deref(), Some("b"));
        assert_eq!(board.live_thread("a"), None);
        assert_eq!(board.live_thread("c").as_deref(), Some("c"));
    }
}
