//! The item events: what changed on a board between two exports that someone has to hear about.
//! Pure: [`item_events`] compares the board before and after and returns the events of the table in
//! `docs/wake-policy.md` (assigned, taken away, a to-do assigned, a stage changed, an ask on your
//! item, a dependency that landed, an item unblocked, every child done, a bug to verify, a test
//! run ready to run, a run done). Nothing is read from disk and nothing is sent: the wake
//! crate turns each event into a wake or an owner notice, once.

mod asks;
mod assign;
mod flow;
mod people;
mod plans;
mod stage;
// test-tag: critical
#[cfg(test)]
mod tests;

use std::collections::HashSet;

use serde_json::Value;

use crate::Board;
use crate::handed::is_handed_run;
use crate::json::{section, text};
use people::gone;

/// What happened.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EventKind {
    /// An item was given to a thread, or to the owner.
    Assigned,
    /// An item was taken away from a thread.
    Unassigned,
    /// A to-do was given to the owner, a thread or a lead.
    TodoAssigned,
    /// The stage or the lane of an item changed under its assignee.
    StageChanged,
    /// Someone else asked a question or wants a decision on the item.
    Ask,
    /// The last thing that held the item back settled.
    Unblocked,
    /// Every child of an epic or a story is done or dropped.
    ChildrenDone,
    /// A request to another board that the item waited on landed.
    DepLanded,
    /// A bug the owner reported reached verifying.
    Verify,
    /// A test run is ready and its build is live.
    RunReady,
    /// A run of a test plan ended.
    PlanRunDone,
}

impl EventKind {
    /// The word that names the kind in a dedupe key.
    #[must_use]
    pub fn key(self) -> &'static str {
        match self {
            Self::Assigned => "assigned",
            Self::Unassigned => "unassigned",
            Self::TodoAssigned => "todo-assigned",
            Self::StageChanged => "status",
            Self::Ask => "ask",
            Self::Unblocked => "unblocked",
            Self::ChildrenDone => "children-done",
            Self::DepLanded => "dep-landed",
            Self::Verify => "verify",
            Self::RunReady => "run-ready",
            Self::PlanRunDone => "plan-run-done",
        }
    }
}

/// Who hears an event.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Hearer {
    /// A thread of this board.
    Thread(String),
    /// The owner.
    Owner,
    /// The lead of the board of this project key.
    Lead(String),
    /// A sub-agent, by its id; its wakes go to the thread that owns it.
    Agent(String),
}

/// One thing that happened to an item, a to-do or an ask, and who must hear of it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ItemEvent {
    /// What happened.
    pub kind: EventKind,
    /// The record it is about: an item, a to-do or an ask.
    pub id: String,
    /// The item it concerns (empty for a to-do that names none).
    pub item: String,
    /// The revision of the record after the change; with the id and the kind it names the event for good.
    pub rev: i64,
    /// Who made the change (`human` is the owner); empty when the board does not say.
    pub by: String,
    /// The title of the item, or the text of the to-do.
    pub title: String,
    /// Who hears it.
    pub to: Hearer,
    /// The facts the sentence needs: the stages, the counts, the ids that settled.
    pub detail: String,
}

impl ItemEvent {
    /// The dedupe key of the event: `<id>:<rev>:<kind>`.
    #[must_use]
    pub fn subject(&self) -> String {
        format!("{}:{}:{}", self.id, self.rev, self.kind.key())
    }

    /// The event with a revision at least this high: when the record it names does not change as the
    /// event happens, the highest revision of the records that make it keeps two occurrences apart.
    pub(crate) fn at_least(self, rev: i64) -> Self {
        Self {
            rev: self.rev.max(rev),
            ..self
        }
    }
}

/// The two exports of a board.
pub(crate) struct Seen<'a> {
    /// The board before.
    pub before: &'a Board,
    /// The board after.
    pub after: &'a Board,
    /// The items that had an open question or a pending decision before.
    pub asked: HashSet<&'a str>,
}

impl<'a> Seen<'a> {
    fn of(before: &'a Board, after: &'a Board) -> Self {
        let asked = section(before, "entries")
            .values()
            .filter(|e| asks::is_ask(e))
            .filter_map(|e| text(e, "item"))
            .collect();
        Self {
            before,
            after,
            asked,
        }
    }
}

/// One item that is there after the change, with what it was before (`null` when it is new).
pub(crate) struct Moved<'a> {
    pub seen: &'a Seen<'a>,
    pub id: &'a str,
    pub was: &'a Value,
    pub now: &'a Value,
}

impl Moved<'_> {
    /// An event about this item.
    pub(crate) fn event(&self, kind: EventKind, to: Hearer, detail: String) -> ItemEvent {
        ItemEvent {
            kind,
            id: self.id.to_owned(),
            item: self.id.to_owned(),
            rev: self.now["rev"].as_i64().unwrap_or(0),
            by: people::actor(self.now),
            title: text(self.now, "title").unwrap_or_default().to_owned(),
            to,
            detail,
        }
    }
}

/// The events between two exports of one board, in the order of the items.
#[must_use]
pub fn item_events(before: &Board, after: &Board) -> Vec<ItemEvent> {
    let seen = Seen::of(before, after);
    let mut out = Vec::new();
    for (id, now) in section(after, "items").iter().filter(|(_, i)| !gone(i)) {
        let was = &before["items"][id.as_str()];
        out.extend(of_item(&Moved {
            seen: &seen,
            id,
            was,
            now,
        }));
    }
    out.extend(assign::todos(&seen));
    out.extend(asks::new_asks(&seen));
    out.extend(flow::children_done(&seen));
    out
}

fn of_item(m: &Moved<'_>) -> Vec<ItemEvent> {
    let mut out = assign::item(m);
    out.extend(stage::changed(m));
    out.extend(stage::verify(m));
    out.extend(flow::unblocked(m));
    out.extend(flow::dep_landed(m));
    out.extend(plans::ready(m));
    out.extend(plans::run_done(m));
    if is_handed_run(m.seen.after, m.now) {
        out.retain(|e| !told_by_handed_ping(e));
    }
    out
}

/// An owner notice about a handed run that the notify handed ping already tells (D-64).
fn told_by_handed_ping(event: &ItemEvent) -> bool {
    event.to == Hearer::Owner && matches!(event.kind, EventKind::RunReady | EventKind::Assigned)
}
