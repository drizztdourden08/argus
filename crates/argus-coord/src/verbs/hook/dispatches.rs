//! The short lists the sub-agent hooks keep on the board: `agentDispatches` (what a lead is
//! about to dispatch), `agentStops` (a stop that came before its start) and `agentLaunches`
//! (an agent id with no record yet). Each keeps its last 20 entries, and an entry ages out.

use argus_board::Board;
use argus_board::json::set;
use argus_core::clock::{now_ms, parse_ms};
use argus_core::js;
use serde_json::Value;

/// A dispatch or a stop note ages out after 10 minutes.
pub const DISPATCH_TTL_MS: i64 = 10 * 60_000;

/// A launch note ages out after an hour.
pub const LAUNCH_TTL_MS: i64 = 60 * 60_000;

const KEEP: usize = 20;

/// The entries of a list that are younger than `ttl`; an entry with no readable `at` is gone.
pub fn recent(list: &Value, ttl: i64) -> Vec<Value> {
    let now = now_ms();
    list.as_array()
        .map(|l| {
            l.iter()
                .filter(|d| {
                    d["at"]
                        .as_str()
                        .and_then(parse_ms)
                        .is_some_and(|at| now - at < ttl)
                })
                .cloned()
                .collect()
        })
        .unwrap_or_default()
}

/// Writes a list to the board, keeping its last 20 entries.
pub fn store(board: &mut Board, key: &str, mut list: Vec<Value>) {
    if list.len() > KEEP {
        list.drain(..list.len() - KEEP);
    }
    set(board, key, Value::Array(list));
}

/// The pending dispatch a starting sub-agent most likely is: the same thread, the same type
/// first, else the oldest of the thread. It leaves the list (which is written back, aged).
pub fn take_dispatch(board: &mut Board, thread: &str, kind: &str) -> Option<Value> {
    let mut list = recent(&board["agentDispatches"], DISPATCH_TTL_MS);
    let mine: Vec<usize> = (0..list.len())
        .filter(|&i| list[i]["thread"].as_str() == Some(thread))
        .collect();
    let same_type = |i: &&usize| {
        let t = &list[**i]["type"];
        if js::truthy(t) {
            t.as_str() == Some(kind)
        } else {
            kind == "general-purpose"
        }
    };
    let pick = mine.iter().find(same_type).or(mine.first()).copied();
    let taken = pick.map(|i| list.remove(i));
    set(board, "agentDispatches", Value::Array(list));
    taken
}
