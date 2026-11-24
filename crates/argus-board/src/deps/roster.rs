//! Who is live on a board. The server knows its running threads and passes its own roster;
//! [`BoardRoster`] answers from the thread statuses of the board files.

use std::path::Path;

use serde_json::Value;

use crate::Board;
use crate::json::section;
use crate::store::read_board;

/// The live threads and the live lead of a project.
pub trait Roster {
    /// The live lead thread of a project, when it has one.
    fn lead(&self, project: &str) -> Option<String>;
    /// The live threads of a project.
    fn live_threads(&self, project: &str) -> Vec<String>;
}

/// A roster read from the boards: a thread is live while it is neither done nor archived.
#[derive(Debug, Clone, Copy)]
pub struct BoardRoster<'a> {
    home: &'a Path,
}

impl<'a> BoardRoster<'a> {
    /// A roster over the boards under `home`.
    #[must_use]
    pub fn new(home: &'a Path) -> Self {
        BoardRoster { home }
    }

    fn board(&self, project: &str) -> Board {
        read_board(self.home, project)
            .ok()
            .flatten()
            .unwrap_or(Value::Null)
    }
}

impl Roster for BoardRoster<'_> {
    fn lead(&self, project: &str) -> Option<String> {
        lead_in(&self.board(project))
    }

    fn live_threads(&self, project: &str) -> Vec<String> {
        live_in(&self.board(project))
    }
}

fn is_live(thread: &Value) -> bool {
    !matches!(thread["status"].as_str(), Some("done" | "archived"))
}

/// The first live lead thread of a board.
pub(crate) fn lead_in(board: &Board) -> Option<String> {
    section(board, "threads")
        .iter()
        .find(|(_, t)| t["role"] == "lead" && is_live(t))
        .map(|(id, _)| id.clone())
}

/// The live threads of a board.
pub(crate) fn live_in(board: &Board) -> Vec<String> {
    section(board, "threads")
        .iter()
        .filter(|(_, t)| is_live(t))
        .map(|(id, _)| id.clone())
        .collect()
}

/// The thread a notice goes to: the preferred one when it is live on the board, else the lead.
pub(crate) fn addressee(board: &Board, preferred: Option<&str>) -> Option<String> {
    preferred
        .filter(|t| section(board, "threads").get(*t).is_some_and(is_live))
        .map(str::to_owned)
        .or_else(|| lead_in(board))
}
