//! What rule 4 asks of the lead and its work: which lead counts, when its idleness began, what is open.

use crate::blockers::is_blocked;
use crate::board::{BoardView, ItemView, LIVE_AGENT, ThreadView};
use crate::item_wakes::target_for;
use crate::time::parse_ms;

/// The lead when it has a live session and is not paused: a dormant project is not a stall.
pub(super) fn live_lead(board: &BoardView) -> Option<(String, &ThreadView)> {
    let id = board.lead()?;
    let thread = board.threads.get(&id)?;
    (thread.is_live() && !thread.status.is("paused")).then_some((id, thread))
}

/// Does a sub-agent of the lead still run?
pub(super) fn has_running_agent(board: &BoardView, lead_id: &str) -> bool {
    board
        .agents
        .values()
        .any(|a| a.rec.is_live() && a.thread.is(lead_id) && a.status.is_any(&LIVE_AGENT))
}

/// When the idleness began as far as the board knows: the lead's last sign of life when it said it was idle,
/// else now.
pub(super) fn idle_start(now: i64, lead: &ThreadView) -> i64 {
    parse_ms(lead.last_seen.as_str())
        .filter(|seen| lead.activity.is("idle") && (1..=now).contains(seen))
        .unwrap_or(now)
}

/// The shown ids (`B-12`) of the `todo` and `doing` items the lead answers for (its own, or a thread that is not live) with no blocker.
pub(super) fn open_items(board: &BoardView, lead_id: &str) -> Vec<String> {
    board
        .items
        .values()
        .filter(|item| item.rec.is_live() && item.status.is_any(&["todo", "doing"]))
        .filter(|item| target_for(board, item.thread.as_str()).as_deref() == Some(lead_id))
        .filter(|item| !is_blocked(board, item))
        .map(ItemView::shown_id)
        .collect()
}
