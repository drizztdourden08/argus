//! What the worktree hygiene reads from a board besides the rules: the sub-agent records, and the
//! open things that may use a worktree (items in progress, pull requests, launchers, bundles).

use argus_board::Board;
use argus_core::clock::parse_ms;
use argus_worktrees::{AgentFact, Link};
use serde_json::{Map, Value};

use super::facts::thread_live;

/// The only status of an item that holds a worktree: someone is working on it.
const WORKING_ITEM: &str = "doing";

fn text(value: &Value) -> Option<String> {
    value
        .as_str()
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .map(str::to_owned)
}

fn section<'a>(board: &'a Board, name: &str) -> impl Iterator<Item = (&'a String, &'a Value)> {
    board[name]
        .as_object()
        .map(Map::iter)
        .into_iter()
        .flatten()
        .filter(|(_, rec)| rec["deletedAt"].is_null())
}

fn agent_of(id: &str, rec: &Value) -> AgentFact {
    AgentFact {
        id: text(&rec["id"]).unwrap_or_else(|| id.to_owned()),
        thread: text(&rec["thread"]),
        status: rec["status"].as_str().unwrap_or_default().to_owned(),
        worktree: text(&rec["worktree"]),
        claude_id: text(&rec["claudeId"]).or_else(|| text(&rec["hookId"])),
        ended_ms: rec["finishedAt"].as_str().and_then(parse_ms),
        item: text(&rec["item"]),
    }
}

/// The sub-agent records of a board.
pub(super) fn agents_of(board: &Board) -> Vec<AgentFact> {
    section(board, "agents")
        .map(|(id, rec)| agent_of(id, rec))
        .collect()
}

/// Is the item in progress and assigned to a live thread? Any other open item that names a worktree does not hold it.
fn item_works(board: &Board, item: &Value) -> bool {
    item["status"] == WORKING_ITEM
        && text(&item["thread"]).is_some_and(|t| thread_live(&board["threads"][t.as_str()]))
}

fn item_links(board: &Board) -> impl Iterator<Item = Link> + '_ {
    section(board, "items")
        .filter(|(_, i)| item_works(board, i))
        .map(|(id, i)| Link {
            what: format!("item {id}"),
            text: Some(format!(
                "{} {}",
                i["title"].as_str().unwrap_or_default(),
                i["description"].as_str().unwrap_or_default()
            )),
            ..Link::default()
        })
}

fn pr_links(board: &Board) -> impl Iterator<Item = Link> + '_ {
    section(board, "prs")
        .filter(|(_, p)| p["status"] == "open")
        .map(|(id, p)| Link {
            what: id.clone(),
            branch: text(&p["head"]),
            ..Link::default()
        })
}

fn launcher_links(board: &Board) -> impl Iterator<Item = Link> + '_ {
    section(board, "launchers")
        .filter(|(_, l)| l["retiredAt"].is_null())
        .map(|(name, l)| Link {
            what: format!("launcher {name}"),
            path: text(&l["cwd"]),
            branch: text(&l["branch"]),
            ..Link::default()
        })
}

fn bundle_links(board: &Board) -> impl Iterator<Item = Link> + '_ {
    section(board, "builds")
        .filter(|(_, b)| b["closedAt"].is_null() && b["status"] != "closed")
        .map(|(tag, b)| Link {
            what: format!("bundle {tag}"),
            path: text(&b["cwd"]),
            ..Link::default()
        })
}

/// The items in progress of a live thread, the open pull requests, the launchers not retired and the bundles not closed.
pub(super) fn links_of(board: &Board) -> Vec<Link> {
    item_links(board)
        .chain(pr_links(board))
        .chain(launcher_links(board))
        .chain(bundle_links(board))
        .collect()
}

// test-tag: critical
#[cfg(test)]
mod tests {
    //! K1: which open things and agents link to a worktree.
    use super::*;
    use serde_json::json;

    #[test]
    fn the_open_things_and_the_agents_are_read() {
        let board: Board = json!({
            "agents": {"A-1": {"id": "A-1", "thread": "lead", "status": "done", "claudeId": "a1",
                "worktree": null, "finishedAt": "2026-11-01T10:00:00.000Z", "item": "I-1"}},
            "threads": {"lead": {"id": "lead", "status": "active"}, "gone": {"id": "gone", "status": "done"}},
            "items": {"I-1": {"status": "doing", "thread": "lead", "title": "land agent-a1"},
                "I-2": {"status": "done", "thread": "lead", "title": "agent-a1 again"},
                "I-3": {"status": "todo", "thread": "lead", "title": "agent-a1 later"},
                "I-4": {"status": "doing", "thread": "gone", "title": "agent-a1 by an ended thread"},
                "I-5": {"status": "doing", "title": "agent-a1 by nobody"}},
            "prs": {"PR-1": {"status": "open", "head": "agent-a1"}, "PR-2": {"status": "merged", "head": "x"}},
            "launchers": {"web": {"cwd": "X:/p/w"}, "old": {"cwd": "X:/p/o", "retiredAt": "2026-10-01T00:00:00.000Z"}},
            "builds": {"b1": {"cwd": "X:/p/b"}, "b2": {"cwd": "X:/p/c", "closedAt": "2026-10-01T00:00:00.000Z"}}
        });
        let agents = agents_of(&board);
        assert_eq!(
            (
                agents[0].claude_id.as_deref(),
                agents[0].ended(),
                agents[0].ended_ms.is_some(),
                agents[0].item.as_deref()
            ),
            (Some("a1"), true, true, Some("I-1"))
        );
        let whats: Vec<String> = links_of(&board).into_iter().map(|l| l.what).collect();
        assert_eq!(whats, ["item I-1", "PR-1", "launcher web", "bundle b1"]);
    }
}
