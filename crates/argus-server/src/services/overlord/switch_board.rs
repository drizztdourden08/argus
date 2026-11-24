//! What the switch-over and the rollback write outside the child (I-173): the wake that asks the
//! desktop Overlord for its handover, the Overlord's threads marked ended on its board, and the
//! handover file the hosted side leaves at a rollback. The desktop session itself is never touched.

use std::fs;
use std::path::{Path, PathBuf};

use argus_board::Board;
use argus_board::error::BoardError;
use argus_board::json::set;
use argus_board::store::{stamp, with_board};
use argus_core::clock::now_iso;
use argus_core::js::truthy;
use argus_overlord::seat::SEAT;
use argus_overlord::switch::{Handback, SWITCH_KIND, SWITCH_REASON, handback_text, switch_text};
use serde_json::{Value, json};

use super::board_id::overlord_board_id;
use crate::{Services, WakeRequest};

const ACTOR: &str = "argus";
const FILE: &str = "handover.md";
const PREVIOUS: &str = "handover.prev.md";
const THREAD_ENDED: [&str; 2] = ["done", "archived"];
const ITEM_CLOSED: [&str; 2] = ["done", "dropped"];

/// `~/.coord/portfolio/handover.md`, the file both sides of the switch read.
pub(crate) fn handover_file(svc: &Services) -> PathBuf {
    svc.paths
        .coord_home
        .join(overlord_board_id(&svc.paths.coord_home))
        .join(FILE)
}

/// Moves an older handover file aside, so the live child never reads a stale one as the desktop
/// Overlord's new handover.
pub(super) fn set_aside(svc: &Services) {
    let file = handover_file(svc);
    if file.exists() {
        let _ = fs::rename(&file, file.with_file_name(PREVIOUS));
    }
}

/// Wakes the desktop Overlord: Argus is switching, write the handover, end the turn.
pub(super) fn ask_handover(svc: &Services) {
    let file = handover_file(svc).display().to_string();
    let wake = WakeRequest::about(
        &overlord_board_id(&svc.paths.coord_home),
        SEAT,
        SWITCH_REASON,
        SWITCH_KIND,
        switch_text(&file),
    );
    if let Err(error) = svc.wake_hook().submit(&[wake]) {
        svc.log.warn(
            "the overlord switch wake was not raised",
            json!({ "error": error }),
        );
    }
}

fn end_thread(board: &mut Board, id: &str) -> Result<(), BoardError> {
    let Some(thread) = board["threads"].get_mut(id) else {
        return Ok(());
    };
    let status = thread["status"].as_str().unwrap_or_default();
    if THREAD_ENDED.contains(&status) {
        return Ok(());
    }
    set(thread, "status", "done");
    stamp(board, "threads", id, ACTOR)
}

/// Marks the named threads of the Overlord's board ended (`done`); a missing one is skipped.
pub(super) fn end_threads(svc: &Services, ids: &[&str]) {
    let ended = with_board(
        &svc.paths.coord_home,
        &overlord_board_id(&svc.paths.coord_home),
        false,
        |board| ids.iter().try_for_each(|id| end_thread(board, id)),
    );
    if let Err(error) = ended {
        svc.log.warn(
            "overlord threads not ended",
            json!({ "error": error.to_string() }),
        );
    }
}

fn is_open(item: &Value) -> bool {
    let status = item["status"].as_str().unwrap_or_default();
    !ITEM_CLOSED.contains(&status) && !truthy(&item["deletedAt"]) && !truthy(&item["archivedAt"])
}

/// The open items of a board, as id and title.
fn open_items(board: &Board) -> Vec<(String, String)> {
    let Some(items) = board["items"].as_object() else {
        return Vec::new();
    };
    items
        .iter()
        .filter(|(_, item)| is_open(item))
        .map(|(id, item)| {
            let title = item["title"].as_str().unwrap_or_default();
            (id.clone(), title.to_owned())
        })
        .collect()
}

fn seat_note(board: &Board) -> Option<String> {
    board["threads"][SEAT]["note"].as_str().map(str::to_owned)
}

fn write_file(file: &Path, text: &str) -> std::io::Result<()> {
    if let Some(dir) = file.parent() {
        fs::create_dir_all(dir)?;
    }
    fs::write(file, text)
}

/// The rollback's board side: reads the seat's note and the open items, marks the hosted seat
/// ended and writes the handover file for the desktop Overlord.
pub(super) fn hand_back(svc: &Services, session_id: Option<String>) {
    let board = svc
        .boards()
        .fresh(&overlord_board_id(&svc.paths.coord_home));
    let back = Handback {
        at: now_iso(),
        session_id,
        note: board.as_ref().and_then(seat_note),
        items: board.as_ref().map(open_items).unwrap_or_default(),
    };
    end_threads(svc, &[SEAT]);
    if let Err(error) = write_file(&handover_file(svc), &handback_text(&back)) {
        svc.log.warn(
            "overlord handover not written",
            json!({ "error": error.to_string() }),
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // test-tag: critical
    #[test]
    fn open_items_leave_out_the_closed_and_the_removed() {
        let board = json!({ "items": {
            "I-1": { "title": "Open", "status": "doing" },
            "I-2": { "title": "Done", "status": "done" },
            "I-3": { "title": "Dropped", "status": "dropped" },
            "I-4": { "title": "Trashed", "status": "todo", "deletedAt": "2026-11-01" },
            "I-5": { "title": "Review", "status": "review" },
        }});
        let open = open_items(&board);
        let ids: Vec<&str> = open.iter().map(|(id, _)| id.as_str()).collect();
        assert_eq!(ids, ["I-1", "I-5"]);
        assert_eq!(open[1].1, "Review");
    }

    #[test]
    fn a_seat_is_marked_done_once_and_a_missing_one_is_skipped() {
        let mut board = json!({ "threads": {
            "coordinator": { "id": "coordinator", "status": "active" },
            "old": { "id": "old", "status": "archived" },
        }});
        end_thread(&mut board, "coordinator").unwrap();
        end_thread(&mut board, "old").unwrap();
        end_thread(&mut board, "missing").unwrap();
        assert_eq!(board["threads"]["coordinator"]["status"], "done");
        assert_eq!(board["threads"]["coordinator"]["updatedBy"], "argus");
        assert_eq!(board["threads"]["old"]["status"], "archived");
    }
}
