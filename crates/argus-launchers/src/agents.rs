//! The Agents view join: every agents doc of a project with the run docs that carry its id
//! (a run started with `--agent A-n`, or `agent` over MCP or the page).

use serde_json::Value;

const LIVE_STATUSES: [&str; 3] = ["working", "waiting", "stop-requested"];

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct AgentFilter {
    /// Only agents that are working, waiting or being stopped, and not archived or deleted.
    pub live: bool,
    pub thread: Option<String>,
}

fn is_live(agent: &Value) -> bool {
    agent["status"]
        .as_str()
        .is_some_and(|s| LIVE_STATUSES.contains(&s))
        && agent["archivedAt"].is_null()
        && agent["deletedAt"].is_null()
}

fn queued_at(run: &Value) -> &str {
    run["queuedAt"].as_str().unwrap_or_default()
}

/// The agent docs that pass the filter, each with `runs`: the run docs that carry its id,
/// newest first.
pub fn agents_with_runs(agents: &[Value], runs: &[Value], filter: &AgentFilter) -> Vec<Value> {
    agents
        .iter()
        .filter(|a| !filter.live || is_live(a))
        .filter(|a| {
            filter
                .thread
                .as_deref()
                .is_none_or(|t| a["thread"].as_str() == Some(t))
        })
        .map(|agent| {
            let mut mine: Vec<Value> = runs
                .iter()
                .filter(|r| !r["agent"].is_null() && r["agent"] == agent["id"])
                .cloned()
                .collect();
            mine.sort_by(|a, b| queued_at(b).cmp(queued_at(a)));
            let mut joined = agent.clone();
            joined["runs"] = Value::Array(mine);
            joined
        })
        .collect()
}
