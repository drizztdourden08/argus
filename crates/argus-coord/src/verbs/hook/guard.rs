//! `hook guard`, `PreToolUse` on the built-in browser's `tabs_close`: a registered thread keeps
//! its board tab open (the owner reads it there and the dashboard watches it), so closing a tab
//! is refused; it can browse in a new tab. Silent for anyone else, and the board is not touched.

use serde_json::json;

use super::Said;
use super::context::board_url;
use super::ctx::Hook;
use super::thread::{doc, is_hosted, registered};
use crate::cli::CoordError;

/// Denies the tab close for a registered thread.
pub fn run(hook: &Hook<'_>) -> Result<Said, CoordError> {
    let Some((project, board)) = hook.project_board()? else {
        return Ok(Said::Nothing);
    };
    let Some(id) = registered(&board, hook) else {
        return Ok(Said::Nothing);
    };
    if is_hosted(doc(&board, &id)) {
        return Ok(Said::Nothing);
    }
    let reason = format!(
        "Do not close browser tabs in a registered thread: your board tab ({}) must stay open, the dashboard reports a closed board to the Overlord. Open other pages in a new tab (tabs_create), and if the board tab is gone, reopen it with preview_start.",
        board_url(hook, &project, &id)
    );
    let out = json!({
        "hookSpecificOutput": {
            "hookEventName": "PreToolUse",
            "permissionDecision": "deny",
            "permissionDecisionReason": reason,
        }
    });
    Ok(Said::Out(out.to_string()))
}
