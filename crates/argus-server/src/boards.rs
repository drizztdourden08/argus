//! Reading boards for the services: the sync service's last good copy, else the file on disk.

use argus_core::lead::LEAD_ROLE;
use std::path::PathBuf;
use std::sync::Arc;

use argus_board::Board;
use argus_board::roles;
use argus_board::store::read_board;
use argus_core::boards::Role;
use argus_sync::{ProjectRow, SyncService};

const ENDED: [&str; 2] = ["done", "archived"];

/// What the services read of the boards.
#[derive(Clone)]
pub struct BoardReader {
    sync: Option<Arc<SyncService>>,
    coord_home: PathBuf,
}

impl BoardReader {
    /// A reader over the sync service, falling back to the boards under `coord_home`.
    #[must_use]
    pub fn new(sync: Option<Arc<SyncService>>, coord_home: PathBuf) -> Self {
        BoardReader { sync, coord_home }
    }

    /// The board of a key, when there is one that reads.
    #[must_use]
    pub fn board(&self, key: &str) -> Option<Board> {
        let kept = self.sync.as_ref().and_then(|s| s.board(key));
        kept.or_else(|| read_board(&self.coord_home, key).ok().flatten())
    }

    /// The board file of a key as it is on disk now, else the last good copy. For a board that
    /// another writer changed a moment ago and the sync service has not exported yet.
    #[must_use]
    pub fn fresh(&self, key: &str) -> Option<Board> {
        read_board(&self.coord_home, key)
            .ok()
            .flatten()
            .or_else(|| self.sync.as_ref().and_then(|s| s.board(key)))
    }

    /// The projects the sync service knows; none without it.
    #[must_use]
    pub fn projects(&self) -> Vec<ProjectRow> {
        self.sync.as_ref().map(|s| s.projects()).unwrap_or_default()
    }

    /// The id of the Overlord's board, found by its role.
    #[must_use]
    pub fn overlord(&self) -> Option<String> {
        roles::id_of(&self.coord_home, Role::Overlord)
    }

    /// The Overlord's board.
    #[must_use]
    pub fn overlord_board(&self) -> Option<Board> {
        self.board(&self.overlord()?)
    }

    /// Is this board (an id or an alias) the Overlord's?
    #[must_use]
    pub fn is_overlord(&self, id: &str) -> bool {
        roles::is_overlord(&self.coord_home, id)
    }

    /// The live lead thread of the Overlord's board.
    #[must_use]
    pub fn overlord_thread(&self) -> Option<String> {
        self.lead(&self.overlord()?)
    }

    /// The thread of a board that coordinates it, unless it is done or archived.
    #[must_use]
    pub fn lead(&self, key: &str) -> Option<String> {
        let board = self.board(key)?;
        lead_in(&board)
    }
}

/// The id of the first thread of a board whose role is `lead` and whose status is not
/// `done` or `archived`.
#[must_use]
pub fn lead_in(board: &Board) -> Option<String> {
    let threads = board["threads"].as_object()?;
    threads
        .values()
        .find(|t| t["role"] == LEAD_ROLE && !ENDED.contains(&t["status"].as_str().unwrap_or("")))
        .and_then(|t| t["id"].as_str().map(str::to_owned))
}
