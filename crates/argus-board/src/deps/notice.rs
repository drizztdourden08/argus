//! The words of a transition: one line for the log of both boards and one inbox line for the
//! side it concerns. The texts are the wake texts of the contract.

use super::ledger::Side;
use super::model::{Dep, DepState, Transition};
use super::roster::addressee;
use crate::Board;
use crate::inbox::queue;
use crate::store::{LogLine, append_log};

/// The line of one transition and the side whose inbox gets it.
pub(crate) struct Note {
    pub text: String,
    pub to: Side,
}

fn item_of(dep: &Dep) -> &str {
    dep.to.item.as_deref().unwrap_or("?")
}

/// The note of a transition, as the contract words it.
pub(crate) fn note(dep: &Dep, t: &Transition) -> Note {
    let id = &dep.id;
    let theirs = dep.assignee();
    let mine = dep.from.item.as_deref().unwrap_or("?");
    match (t.from, t.to) {
        (DepState::Opening, DepState::Open) => Note {
            text: format!(
                "{} asks you for {} \"{}\" ({id})",
                dep.requester(),
                item_of(dep),
                dep.title
            ),
            to: Side::Assignee,
        },
        (_, DepState::Open) => Note {
            text: format!("{theirs} is open again ({id})"),
            to: Side::Requester,
        },
        (_, DepState::Blocked) => Note {
            text: format!(
                "{theirs} is blocked on you ({id}): {}",
                dep.reason.as_deref().unwrap_or("no reason given")
            ),
            to: Side::Requester,
        },
        (_, DepState::Cancelled) => Note {
            text: format!(
                "{} cancelled its request {} ({id})",
                dep.requester(),
                item_of(dep)
            ),
            to: Side::Assignee,
        },
        (_, state) => Note {
            text: format!("{theirs} is {state} ({id}): resume {mine}"),
            to: Side::Requester,
        },
    }
}

/// Write the notes into one board: a log line for each, and an inbox line for each that is
/// addressed to this side.
pub(crate) fn write(board: &mut Board, dep: &Dep, side: Side, notes: &[Note]) {
    let (item, thread) = match side {
        Side::Requester => (&dep.from.item, &dep.from.thread),
        Side::Assignee => (&dep.to.item, &dep.to.thread),
    };
    for n in notes {
        let line = LogLine {
            thread: "argus",
            item: item.as_deref(),
            text: &n.text,
        };
        append_log(board, "deps", &line);
        if n.to != side {
            continue;
        }
        if let Some(to) = addressee(board, thread.as_deref()) {
            queue(
                board,
                &to,
                "argus",
                "change",
                &n.text,
                Some(dep.id.as_str()),
            );
        }
    }
}
