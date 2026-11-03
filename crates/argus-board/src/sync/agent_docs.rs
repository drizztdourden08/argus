//! The agent side of the page documents: an agent with the items it works on, where it runs and
//! what it waits on, and an item with the agents working on it.

use serde_json::{Value, json};

use crate::Board;
use crate::agent_links::{blocked_by_of, items_of, where_of};
use crate::agents::is_live;
use crate::json::{section, set, truthy};

/// An agent as the page sees it: every link field present. `items` falls back to the older
/// `item`; `where` is the worktree the lead named, `master`, or the folder `agent-<claudeId>`
/// (`whereSource` says `manual` or `auto`); nothing is written back to disk.
pub(super) fn public_agent(doc: &Value) -> Value {
    let mut out = doc.clone();
    let (place, source) = where_of(doc);
    set(&mut out, "items", json!(items_of(doc)));
    set(&mut out, "master", truthy(doc, "master"));
    set(&mut out, "blockedBy", json!(blocked_by_of(doc)));
    set(&mut out, "where", place);
    set(&mut out, "whereSource", source);
    out
}

/// The agents working on an item: the live ones that list it, as `{id, name, status}`.
pub(super) fn working_on(board: &Board, item: &str) -> Value {
    let rows: Vec<Value> = section(board, "agents")
        .values()
        .filter(|a| is_live(a) && items_of(a).iter().any(|i| i == item))
        .map(|a| json!({"id": a["id"], "name": a["name"], "status": a["status"]}))
        .collect();
    json!(rows)
}
