//! What reconcile sees: the item of each end as its board holds it right now.

use std::path::Path;

use argus_core::clock::now_ms;
use serde_json::Value;

use super::cycle;
use super::mirror::{entries, entry_of};
use super::model::{Dep, node};
use crate::Board;
use crate::error::BoardError;
use crate::json::{doc, nullish, section, truthy};
use crate::store::read_board;

/// An item as a dependency needs to know it.
#[derive(Debug, Clone)]
pub(crate) struct Seen {
    pub id: String,
    pub status: String,
    pub trashed: bool,
    pub archived: bool,
    pub thread: Option<String>,
    pub title: String,
    pub created_by: String,
    pub waits_on: Vec<Value>,
    pub requested_by: Vec<Value>,
}

impl Seen {
    pub(crate) fn of(id: &str, item: &Value) -> Seen {
        Seen {
            id: id.to_owned(),
            status: nullish(item, "status").unwrap_or_default(),
            trashed: truthy(item, "deletedAt"),
            archived: truthy(item, "archivedAt"),
            thread: nullish(item, "thread"),
            title: nullish(item, "title").unwrap_or_default(),
            created_by: nullish(item, "createdBy").unwrap_or_default(),
            waits_on: entries(&item["waitsOn"]),
            requested_by: entries(&item["requestedBy"]),
        }
    }

    /// In the trash or archived.
    pub(crate) fn removed(&self) -> bool {
        self.trashed || self.archived
    }

    /// Not done, not dropped, not removed: work that can still happen.
    pub(crate) fn open(&self) -> bool {
        !self.removed() && !matches!(self.status.as_str(), "done" | "dropped")
    }

    /// The status as a refusal prints it.
    pub(crate) fn shown_status(&self) -> &str {
        if self.trashed {
            "in the trash"
        } else if self.archived {
            "archived"
        } else {
            &self.status
        }
    }
}

/// One end of a dependency as it is on disk.
#[derive(Debug, Clone)]
pub(crate) enum Look {
    NoBoard,
    NoItem,
    Item(Box<Seen>),
}

impl Look {
    pub(crate) fn item(&self) -> Option<&Seen> {
        match self {
            Look::Item(seen) => Some(seen.as_ref()),
            _ => None,
        }
    }
}

/// The id of the item of a board that carries `requestedBy` for a dependency.
pub(crate) fn find_requested(board: &Board, dep: &str) -> Option<String> {
    section(board, "items")
        .iter()
        .find(|(_, item)| entry_of(&entries(&item["requestedBy"]), dep).is_some())
        .map(|(id, _)| id.clone())
}

/// An item of a board, by its id or else by the dependency that requested it.
pub(crate) fn look_in(board: &Board, item: Option<&str>, dep: &str) -> Look {
    let found = item
        .map(str::to_owned)
        .filter(|id| doc(board, "items", id).is_some())
        .or_else(|| find_requested(board, dep));
    match found.and_then(|id| doc(board, "items", &id).map(|i| Seen::of(&id, i))) {
        Some(seen) => Look::Item(Box::new(seen)),
        None => Look::NoItem,
    }
}

fn look(home: &Path, project: &str, item: Option<&str>, dep: &str) -> Result<Look, BoardError> {
    Ok(match read_board(home, project)? {
        Some(board) => look_in(&board, item, dep),
        None => Look::NoBoard,
    })
}

/// Both ends of a dependency, the retired projects, the loop test and the time.
pub(crate) struct Sight {
    pub requester: Look,
    pub assignee: Look,
    pub retired: (bool, bool),
    pub loops: bool,
    pub now: i64,
}

fn retired(ledger: &Board, project: &str) -> bool {
    truthy(&ledger["projects"][project], "retired")
}

impl Sight {
    /// Read both boards. `ledger` is the portfolio board, `deps` its dependencies.
    pub(crate) fn read(
        home: &Path,
        ledger: &Board,
        deps: &[Dep],
        dep: &Dep,
    ) -> Result<Sight, BoardError> {
        let requester = look(home, &dep.from.project, dep.from.item.as_deref(), &dep.id)?;
        let assignee = look(home, &dep.to.project, dep.to.item.as_deref(), &dep.id)?;
        let loops = match (&requester, &assignee) {
            (Look::Item(r), Look::Item(a)) => {
                let (r, a) = (node(&dep.from.project, &r.id), node(&dep.to.project, &a.id));
                cycle::path(deps, &a, &r).is_some()
            }
            _ => false,
        };
        Ok(Sight {
            requester,
            assignee,
            retired: (
                retired(ledger, &dep.from.project),
                retired(ledger, &dep.to.project),
            ),
            loops,
            now: now_ms(),
        })
    }
}
