//! The items a closed agent held (I-206): the reaper and the reconciler end agents without the
//! `coord agent` verbs, so they hand the items back the way the verbs do, through the board's own
//! rule (`argus_board::holds`).

use crate::board::Board;
use crate::record::{Record, id_of, stamp};
use argus_board::holds::handbacks;
use serde_json::{Map, Value, json};

const ACTOR: &str = "dashboard";

fn keyed<'a>(docs: impl Iterator<Item = &'a Record>) -> Value {
    let map: Map<String, Value> = docs
        .map(|d| (id_of(d).to_owned(), Value::Object(d.clone())))
        .collect();
    Value::Object(map)
}

/// The board as the board crate reads it: the agent, the threads and the items.
fn view(board: &Board, agent_id: &str) -> Value {
    let own = board.agents.iter().filter(|a| id_of(a) == agent_id);
    json!({
        "agents": keyed(own),
        "threads": keyed(board.threads.iter()),
        "items": keyed(board.items.iter()),
    })
}

/// Hands each item the agent still holds back to the agent's thread.
pub fn release(board: &mut Board, agent_id: &str, at_ms: i64) {
    for (item, thread) in handbacks(&view(board, agent_id), agent_id) {
        if let Some(record) = board.item_mut(&item) {
            record.insert("assignee".into(), Value::from(thread));
            stamp(record, ACTOR, at_ms);
        }
    }
}
