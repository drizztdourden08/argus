//! `list_agents`: the agent records of a project, each with the runs that carry its id.

use serde_json::{Value, json};

use crate::args::{arg_str, flag};
use crate::backend::McpBackend;
use crate::json::{display, text, truthy};
use argus_core::collate;

const LIVE: [&str; 3] = ["working", "waiting", "stop-requested"];

fn is_live(agent: &Value) -> bool {
    text(agent, "status").is_some_and(|s| LIVE.contains(&s))
        && !truthy(agent, "archivedAt")
        && !truthy(agent, "deletedAt")
}

fn runs_of(runs: &[Value], agent: &Value) -> Vec<Value> {
    let Some(id) = text(agent, "id") else {
        return Vec::new();
    };
    let mut mine: Vec<Value> = runs
        .iter()
        .filter(|r| text(r, "agent") == Some(id))
        .cloned()
        .collect();
    mine.sort_by(|a, b| collate::compare(&display(b, "queuedAt"), &display(a, "queuedAt"), false));
    mine
}

/// The agents, with `runs` newest first, optionally only the live ones or one thread's. The docs are
/// the exported ones, so each also carries `items`, `master`, `blockedBy`, `where` and `whereSource`.
pub fn list_agents(backend: &dyn McpBackend, key: &str, args: &Value) -> Value {
    let live = flag(args, "live");
    let thread = arg_str(args, "thread");
    let runs = backend.docs(key, "runs");
    let agents: Vec<Value> = backend
        .docs(key, "agents")
        .into_iter()
        .filter(|a| (!live || is_live(a)) && thread.is_none_or(|t| text(a, "thread") == Some(t)))
        .map(|mut a| {
            let joined = json!(runs_of(&runs, &a));
            if let Some(map) = a.as_object_mut() {
                map.insert("runs".into(), joined);
            }
            a
        })
        .collect();
    json!({ "agents": agents })
}
