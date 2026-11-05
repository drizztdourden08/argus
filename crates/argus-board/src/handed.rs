//! A test run handed to the owner: ready, assigned to him, on a live build. The notify handed
//! ping tells him of it, once; the item watch sends no run-ready or assigned notice for it (D-64).

use serde_json::Value;

use crate::Board;
use crate::json::{text, truthy};
use crate::launchers::build_ready;

/// Is the item a live test run in stage `ready`, assigned to the owner, whose build is live?
#[must_use]
pub fn is_handed_run(board: &Board, run: &Value) -> bool {
    text(run, "type") == Some("test-run")
        && text(run, "stage") == Some("ready")
        && text(run, "assignee") == Some("owner")
        && !truthy(run, "deletedAt")
        && !truthy(run, "archivedAt")
        && text(&run["fields"], "build").is_some_and(|name| build_ready(board, name))
}
