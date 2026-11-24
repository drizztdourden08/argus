//! The assignee an agent holds (I-206): `agent:<id>` on each item it works on. The end of the agent
//! hands each item it still holds back to its thread; a takeover hands them to the absorbing agent.
//! Every path that ends an agent (coord verbs, reaper, reconciler, takeover) goes through here.

use serde_json::Value;

use crate::Board;
use crate::agent_links::items_of;
use crate::error::BoardError;
use crate::json::{display, doc, doc_mut, set, truthy};
use crate::store::stamp;

/// The assignee text of an agent.
#[must_use]
pub fn tag(agent_id: &str) -> String {
    format!("agent:{agent_id}")
}

/// The items an agent still holds, each with the thread they go back to. Empty when the agent's
/// thread is gone. An item in the trash or taken by someone else is not held.
#[must_use]
pub fn handbacks(board: &Board, agent_id: &str) -> Vec<(String, String)> {
    let Some(agent) = doc(board, "agents", agent_id) else {
        return Vec::new();
    };
    let thread = display(&agent["thread"]);
    if doc(board, "threads", &thread).is_none() {
        return Vec::new();
    }
    let held = |item: &String| {
        doc(board, "items", item)
            .is_some_and(|i| !truthy(i, "deletedAt") && display(&i["assignee"]) == tag(agent_id))
    };
    items_of(agent)
        .into_iter()
        .filter(held)
        .map(|item| (item, thread.clone()))
        .collect()
}

/// Moves every item `from` still holds to `to`.
///
/// # Errors
/// An item doc cannot be stamped.
pub fn transfer(board: &mut Board, from: &str, to: &str, actor: &str) -> Result<(), BoardError> {
    let items = doc(board, "agents", from).map(items_of).unwrap_or_default();
    for item in items {
        let held = doc(board, "items", &item).is_some_and(|i| display(&i["assignee"]) == tag(from));
        if !held {
            continue;
        }
        if let Some(target) = doc_mut(board, "items", &item) {
            set(target, "assignee", Value::from(tag(to)));
        }
        stamp(board, "items", &item, actor)?;
    }
    Ok(())
}

// test-tag: critical
#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn board() -> Board {
        json!({
            "threads": {"lead": {"id": "lead"}},
            "agents": {
                "A-1": {"id": "A-1", "thread": "lead", "items": ["I-1", "I-2", "I-3"]},
                "A-2": {"id": "A-2", "thread": "lead", "items": ["I-1"]},
            },
            "items": {
                "I-1": {"id": "I-1", "assignee": "agent:A-1", "rev": 1},
                "I-2": {"id": "I-2", "assignee": "worker"},
                "I-3": {"id": "I-3", "assignee": "agent:A-1", "deletedAt": "2026-11-01"},
            },
        })
    }

    #[test]
    fn an_agent_hands_back_only_the_live_items_it_still_holds() {
        let held = handbacks(&board(), "A-1");
        assert_eq!(held, vec![("I-1".to_owned(), "lead".to_owned())]);
        assert_eq!(handbacks(&board(), "A-9").len(), 0);
    }

    #[test]
    fn an_agent_whose_thread_is_gone_hands_back_nothing() {
        let mut gone = board();
        gone["threads"] = json!({});
        assert_eq!(handbacks(&gone, "A-1").len(), 0);
    }

    #[test]
    fn a_transfer_gives_the_held_items_to_the_absorbing_agent() {
        let mut moved = board();
        transfer(&mut moved, "A-1", "A-2", "lead").unwrap();
        assert_eq!(moved["items"]["I-1"]["assignee"], "agent:A-2");
        assert_eq!(moved["items"]["I-1"]["rev"], 2);
        assert_eq!(moved["items"]["I-2"]["assignee"], "worker");
    }
}
