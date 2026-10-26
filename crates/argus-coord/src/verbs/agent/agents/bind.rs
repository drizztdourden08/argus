//! Binding the agent a lead registers to the record the hooks made for it: the hook records the
//! sub-agent the moment the Agent call starts, `agent start` comes a little later, and each side
//! looks for the other by how well their descriptions fit (`bestFit`) and by the Claude id.

use argus_agents::record::claude_id_of;
use argus_agents::text::best_fit;
use argus_board::Board;
use argus_board::agents::is_live;
use argus_board::json::{display, section, strict_eq, truthy};
use argus_core::clock::parse_ms;
use serde_json::Value;

/// A hook record and a registration within this window are the same sub-agent.
const BIND_MS: i64 = 3 * 60_000;
/// How long a launch note of the `PostToolUse` hook stays usable.
const LAUNCH_MS: i64 = 60 * 60_000;

/// The Claude id of an agent record: `claudeId`, else its older name `hookId`.
#[must_use]
pub fn claude_id(agent: &Value) -> Option<String> {
    agent.as_object().and_then(claude_id_of)
}

fn within(at: &Value, now_ms: i64, window: i64) -> bool {
    at.as_str()
        .and_then(parse_ms)
        .is_some_and(|t| now_ms - t < window)
}

/// The record the hooks made for the sub-agent being registered, if any: the same thread, still
/// working, not claimed by an earlier `agent start`, started within the window, and described like
/// this agent. A record the hook could not describe fits only when it is the one candidate.
#[must_use]
pub fn hook_record(board: &Board, owner: &str, text: &str, now_ms: i64) -> Option<String> {
    let mut open: Vec<&Value> = section(board, "agents")
        .values()
        .filter(|a| {
            a["thread"].as_str() == Some(owner)
                && a["source"] == "hook"
                && !truthy(a, "leadBound")
                && a["status"] == "working"
                && is_live(a)
                && within(&a["startedAt"], now_ms, BIND_MS)
        })
        .collect();
    open.sort_by_key(|a| display(&a["startedAt"]));
    let described: Vec<&Value> = open
        .iter()
        .copied()
        .filter(|a| truthy(a, "task") && !strict_eq(&a["task"], &a["agentType"]))
        .collect();
    let task_of = |a: &&Value| display(&a["task"]);
    let fit = best_fit(text, &described, task_of, 1).copied();
    let only = (open.len() == 1 && described.is_empty()).then(|| open[0]);
    fit.or(only).map(|a| display(&a["id"]))
}

/// The launch note (`board.agentLaunches`, left by the `PostToolUse` hook of the Agent call) that
/// reads like this agent: the same thread, recent, not already the lead's own record, and
/// described alike (two shared words at least). Returns its Claude id.
#[must_use]
pub fn launch_for(board: &Board, owner: &str, text: &str, now_ms: i64) -> Option<String> {
    let launches = board["agentLaunches"].as_array()?;
    let taken = |id: &str| {
        section(board, "agents").values().any(|a| {
            claude_id(a).as_deref() == Some(id)
                && (truthy(a, "leadBound") || a["source"] != "hook")
        })
    };
    let mine: Vec<&Value> = launches
        .iter()
        .filter(|l| {
            l["thread"].as_str() == Some(owner)
                && l["claudeId"].as_str().is_some_and(|id| !id.is_empty() && !taken(id))
                && within(&l["at"], now_ms, LAUNCH_MS)
        })
        .collect();
    let description = |l: &&Value| l["description"].as_str().unwrap_or_default().to_owned();
    best_fit(text, &mine, description, 2).and_then(|l| l["claudeId"].as_str().map(str::to_owned))
}
