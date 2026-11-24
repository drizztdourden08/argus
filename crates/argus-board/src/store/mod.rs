//! The per-project board file, `<home>/<project>/board.json`: one JSON document holding every
//! collection the dashboard shows. Every mutation goes through [`with_board`] (blocking, for the
//! CLI) or [`with_board_async`] (polling, for the server): take the lock file, load, mutate,
//! write atomically (temp file, flush to the disk, rename; see `argus_core::fs`) and release.

mod damage;
mod docs;
mod io;
mod lane;
mod lock;
mod migrate;
mod migrate_lead;
mod normalize;
mod rank;

use std::path::{Path, PathBuf};

use argus_core::clock::now_iso;
use serde_json::json;

pub use damage::{BoardDamage, DamagedBoard, check_board_bytes, damaged_boards, read_board_text};
pub use docs::{LogLine, append_log, create_doc, next_id, remove_doc, stamp, touch};
pub use io::{read_board, save_board, save_board_with};
pub use lane::{set_lane, stamp_lane};
pub use lock::{BoardLock, LockWait, try_lock, with_board, with_board_async};
pub use normalize::normalize;
pub use rank::{rank_between, rank_end};

pub(crate) use docs::push_log_line;

use crate::Board;

/// The board layout version. Boards written before schema 2 are migrated on load.
pub const SCHEMA: i64 = 2;

/// The folder of a project under the coord home.
#[must_use]
pub fn board_dir(home: &Path, key: &str) -> PathBuf {
    home.join(key)
}

/// The board file of a project.
#[must_use]
pub fn board_path(home: &Path, key: &str) -> PathBuf {
    board_dir(home, key).join("board.json")
}

/// A new board for a project, with every collection empty.
#[must_use]
pub fn empty_board(key: &str) -> Board {
    let at = now_iso();
    json!({
        "schema": SCHEMA,
        "project": key,
        "artifact": {"url": null, "publishedAt": null},
        "createdAt": at,
        "updatedAt": at,
        "counters": {},
        "threads": {},
        "items": {},
        "tests": {},
        "entries": {},
        "locks": {},
        "actions": {},
        "agents": {},
        "log": {},
        "inbox": {},
        "runDefs": {},
        "builds": {},
        "launchers": {},
        "dirty": {},
        "versions": {},
    })
}
