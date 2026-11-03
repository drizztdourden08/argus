//! `hook subagent-stop`, `SubagentStop`: closes the record of the sub-agent's own id with the
//! answer and the model its transcript shows. The session need not be the one that started it
//! (a lead resumed since has a new session id). A stop for an id the board has no record of
//! touches no record at all, however alike a record of the lead may read: it is noted for the
//! start that may still come. A stop that only waits on the agent's own background command
//! leaves its record working: the final stop, after the handback, closes it.

use argus_agents::hook_logic::failed;
use argus_board::json::set;
use argus_board::store::stamp;
use argus_board::{Board, BoardError};
use argus_core::clock::now_iso;
use argus_core::js;
use serde_json::json;

use super::binding::{close, payload_agent_id, record_of};
use super::ctx::Hook;
use super::dispatches::{DISPATCH_TTL_MS, recent, store};
use super::thread::registered;
use super::transcript::model_of;
use super::waiting::on_background;
use crate::cli::CoordError;

/// The stop as the hook saw it.
struct Stop {
    agent_id: String,
    last: String,
    used: Option<String>,
    waiting: bool,
}

fn close_known(board: &mut Board, stop: &Stop, id: &str) -> Result<(), BoardError> {
    let record = &mut board["agents"][id];
    if record["claudeId"].is_null() {
        set(record, "claudeId", stop.agent_id.as_str());
    }
    if stop.waiting {
        return Ok(());
    }
    let actor = js::string(&record["thread"]);
    close(board, id, &stop.last, stop.used.as_deref());
    stamp(board, "agents", id, &actor)
}

fn on_board(board: &mut Board, hook: &Hook<'_>, stop: &Stop) -> Result<(), BoardError> {
    if let Some(id) = record_of(board, &stop.agent_id) {
        return close_known(board, stop, &id);
    }
    let Some(thread) = registered(board, hook) else {
        return Ok(());
    };
    let mut stops = recent(&board["agentStops"], DISPATCH_TTL_MS);
    stops.push(json!({
        "hookId": stop.agent_id, "thread": thread, "last": stop.last,
        "used": stop.used, "at": now_iso(),
    }));
    store(board, "agentStops", stops);
    Ok(())
}

/// Closes the record of a finished sub-agent, or notes the stop.
pub fn run(hook: &Hook<'_>) -> Result<(), CoordError> {
    let Some(agent_id) = payload_agent_id(hook) else {
        return Ok(());
    };
    let last = hook.input["last_assistant_message"]
        .as_str()
        .map_or_else(String::new, |s| js::trim(s).to_owned());
    let path = &hook.input["agent_transcript_path"];
    let stop = Stop {
        used: model_of(path),
        waiting: !failed(&last) && on_background(path),
        agent_id,
        last,
    };
    let admit = |peek: &Board| {
        record_of(peek, &stop.agent_id).is_some() || registered(peek, hook).is_some()
    };
    hook.mutate_if(admit, |board| on_board(board, hook, &stop))
}
