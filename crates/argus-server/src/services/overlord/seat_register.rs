//! What `coord thread register coordinator --role coordinator` did for the hosted Overlord, done by
//! Argus itself when the live child reports its session id: the seat thread of the Overlord's
//! board is created or refreshed with the role coordinator, host argus and that session id. The
//! child never runs the verb (its settings do not allow coord), so nothing waits on an approval.

use std::path::Path;

use argus_board::Board;
use argus_board::error::BoardError;
use argus_board::json::{map_mut, set, unset};
use argus_board::store::{rank_end, stamp, touch, with_board};
use argus_core::clock::now_iso;
use argus_core::js;
use argus_core::log::Logger;
use argus_overlord::seat::SEAT;
use serde_json::json;

/// The Overlord's board.
const BOARD: &str = "portfolio";
const ACTOR: &str = "argus";
const HOST: &str = "argus";
/// The marks a live seat never carries.
const STALE: [&str; 4] = ["placeholder", "mergedInto", "archivedAt", "archivedBy"];

fn new_seat(board: &mut Board) {
    let at = now_iso();
    let rank = js::number(rank_end(board, "threads"));
    let thread = json!({
        "id": SEAT, "name": SEAT, "role": "coordinator",
        "flags": { "needsHuman": false, "blocked": false },
        "note": null, "subject": null, "brief": null,
        "registeredAt": at, "createdBy": ACTOR, "updatedAt": at, "updatedBy": ACTOR,
        "rev": 1, "rank": rank,
    });
    map_mut(board, "threads").insert(SEAT.to_owned(), thread);
    touch(board, "threads", SEAT);
}

/// Creates or refreshes the seat thread for the hosted session `session_id`.
fn register(board: &mut Board, session_id: &str) -> Result<(), BoardError> {
    if board["threads"].get(SEAT).is_none() {
        new_seat(board);
    }
    let thread = &mut board["threads"][SEAT];
    set(thread, "role", "coordinator");
    set(thread, "host", HOST);
    set(thread, "sessionId", session_id);
    let status = if thread["status"] == "paused" {
        "paused"
    } else {
        "active"
    };
    set(thread, "status", status);
    for key in STALE {
        unset(thread, key);
    }
    set(thread, "lastSeen", now_iso());
    stamp(board, "threads", SEAT, ACTOR)
}

/// Registers the hosted Overlord's thread; a failure logs one warning and changes nothing else.
pub(super) fn register_seat(coord_home: &Path, log: &Logger, session_id: &str) {
    let done = with_board(coord_home, BOARD, false, |board| {
        register(board, session_id)
    });
    if let Err(error) = done {
        log.warn(
            "the hosted overlord thread was not registered",
            json!({ "error": error.to_string() }),
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // test-tag: critical
    #[test]
    fn a_missing_seat_is_created_as_the_hosted_coordinator() {
        let mut board = json!({ "threads": {} });
        register(&mut board, "sid-1").unwrap();
        let seat = &board["threads"][SEAT];
        assert_eq!(seat["role"], "coordinator");
        assert_eq!(seat["host"], "argus");
        assert_eq!(seat["sessionId"], "sid-1");
        assert_eq!(seat["status"], "active");
        assert_eq!(seat["createdBy"], "argus");
    }

    // test-tag: critical
    #[test]
    fn an_ended_seat_is_taken_back_with_the_new_session() {
        let mut board = json!({ "threads": { "coordinator": {
            "id": "coordinator", "status": "done", "rev": 4, "sessionId": "old",
            "archivedAt": "2026-11-01", "note": "kept",
        }}});
        register(&mut board, "sid-2").unwrap();
        let seat = &board["threads"][SEAT];
        assert_eq!(seat["status"], "active");
        assert_eq!(seat["sessionId"], "sid-2");
        assert_eq!(seat["note"], "kept");
        assert_eq!(seat["rev"], 5);
        assert!(seat.get("archivedAt").is_none());
    }

    #[test]
    fn a_paused_seat_stays_paused() {
        let mut board = json!({ "threads": { "coordinator": {
            "id": "coordinator", "status": "paused", "rev": 1,
        }}});
        register(&mut board, "sid-3").unwrap();
        assert_eq!(board["threads"][SEAT]["status"], "paused");
    }
}
