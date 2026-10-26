//! Sub-agents as board docs (collection `agents`, ids A-1, A-2, ...): what counts as running,
//! and stopping the agents of a thread that is paused, archived or done.

use argus_core::clock::now_iso;
use serde_json::Value;

use crate::Board;
use crate::error::BoardError;
use crate::json::{doc_mut, find_ids, set, truthy};
use crate::store::stamp;

const LIVE: [&str; 3] = ["working", "waiting", "stop-requested"];

/// Whether an agent is running: working, waiting or asked to stop, and neither archived nor in
/// the trash.
pub(crate) fn is_live(agent: &Value) -> bool {
    agent["status"].as_str().is_some_and(|s| LIVE.contains(&s))
        && !truthy(agent, "archivedAt")
        && !truthy(agent, "deletedAt")
}

/// The ids of the live agents of one thread, or of every thread.
pub(crate) fn live_agent_ids(board: &Board, thread: Option<&str>) -> Vec<String> {
    find_ids(board, "agents", |a| {
        is_live(a) && thread.is_none_or(|t| a["thread"].as_str() == Some(t))
    })
}

/// A thread that is done, archived or paused has no agents running: they become `stopped`.
pub(crate) fn stop_agents(board: &mut Board, thread: &str, actor: &str) -> Result<(), BoardError> {
    for id in live_agent_ids(board, Some(thread)) {
        if let Some(agent) = doc_mut(board, "agents", &id) {
            set(agent, "status", "stopped");
            set(agent, "finishedAt", now_iso());
        }
        stamp(board, "agents", &id, actor)?;
    }
    Ok(())
}
