//! What an item event comes to: a wake for a thread, a notice for the owner, or a line in the activity
//! log for an item the owner holds.

use argus_board::item_events::{EventKind, Hearer, ItemEvent};

use super::text;
use crate::activity::{ActivityLine, Level};
use crate::board::BoardView;
use crate::item_wakes::target_for;
use crate::policy::coordinator_in;
use crate::reminders::{OwnerNotice, TOPIC_ASSIGNED, TOPIC_RUN, TOPIC_VERIFY};
use crate::source::BoardSource;
use crate::wake::WakeInput;

/// The thread that did not make the change when a notice names no one.
const NO_ONE: &str = "argus";

/// What an event comes to.
pub(super) enum Told {
    /// A wake for a thread, passive or not as the wake policy says.
    Wake(WakeInput),
    /// A toast and a notification for the owner.
    Owner(OwnerNotice),
    /// A line of the activity log.
    Line(ActivityLine),
}

/// The board an event happened on, and the boards around it.
pub(super) struct Where<'a> {
    /// The project key.
    pub scope: &'a str,
    /// The board after the change.
    pub board: &'a BoardView,
    /// Every project board, for the lead of another project.
    pub boards: &'a dyn BoardSource,
}

/// The reason of the wake an event makes; none for an event that is not a wake.
fn reason_of(kind: EventKind) -> Option<&'static str> {
    match kind {
        EventKind::Assigned => Some("item-assigned"),
        EventKind::Unassigned => Some("item-unassigned"),
        EventKind::TodoAssigned => Some("todo-assigned"),
        EventKind::StageChanged => Some("item-status"),
        EventKind::Ask => Some("item-ask"),
        EventKind::Unblocked => Some("item-unblocked"),
        EventKind::ChildrenDone => Some("children-done"),
        EventKind::PlanRunDone => Some("plan-run-done"),
        EventKind::DepLanded | EventKind::Verify | EventKind::RunReady => None,
    }
}

/// The topic of the owner notice an event makes, when it makes one.
fn topic_of(event: &ItemEvent) -> Option<&'static str> {
    match (event.kind, &event.to) {
        (EventKind::Verify, _) => Some(TOPIC_VERIFY),
        (EventKind::RunReady, _) => Some(TOPIC_RUN),
        (EventKind::Assigned | EventKind::TodoAssigned, Hearer::Owner) => Some(TOPIC_ASSIGNED),
        _ => None,
    }
}

/// What an event comes to; `None` for the ones another producer owns (a dependency that landed is told by
/// the ledger) and for a hearer that has nobody to wake.
pub(super) fn route(at: &Where<'_>, event: &ItemEvent) -> Option<Told> {
    if let Some(topic) = topic_of(event) {
        return Some(Told::Owner(notice(at, event, topic)));
    }
    match (&event.to, reason_of(event.kind)) {
        (Hearer::Owner, Some(_)) => Some(Told::Line(line(at, event))),
        (_, Some(reason)) => wake(at, event, reason).map(Told::Wake),
        (_, None) => None,
    }
}

fn notice(at: &Where<'_>, event: &ItemEvent, topic: &str) -> OwnerNotice {
    let (title, words) = text::notice_words(event, at.scope);
    OwnerNotice {
        topic: topic.to_owned(),
        title,
        text: words,
        project: at.scope.to_owned(),
        thread: if event.by.is_empty() {
            NO_ONE.to_owned()
        } else {
            event.by.clone()
        },
        since: event.subject(),
    }
}

fn line(at: &Where<'_>, event: &ItemEvent) -> ActivityLine {
    ActivityLine {
        kind: event.kind.key().to_owned(),
        level: Level::Info,
        text: format!("{}: {}", at.scope, text::wake_line(event)),
        project: Some(at.scope.to_owned()),
        thread: None,
    }
}

/// The project and the thread a wake goes to: the thread while it is live, else its lead; or the lead of
/// the project a to-do names.
fn address(at: &Where<'_>, to: &Hearer) -> Option<(String, String)> {
    match to {
        Hearer::Thread(thread) => {
            target_for(at.board, thread).map(|found| (at.scope.to_owned(), found))
        }
        Hearer::Lead(key) => {
            let board = at.boards.board(key)?;
            coordinator_in(Some(&*board)).map(|lead| (key.clone(), lead))
        }
        Hearer::Owner => None,
    }
}

fn wake(at: &Where<'_>, event: &ItemEvent, reason: &str) -> Option<WakeInput> {
    let (scope, thread) = address(at, &event.to)?;
    Some(WakeInput::about(
        &scope,
        &thread,
        reason,
        &event.subject(),
        text::wake_line(event),
    ))
}
