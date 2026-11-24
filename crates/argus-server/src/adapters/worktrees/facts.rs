//! A project's board as the worktree service reads it: the rules, the keeps, the thread's
//! answers, the threads and the owner entries that matter to a removal.

use argus_core::lead::LEAD_ROLE;
use std::collections::{BTreeMap, HashMap, HashSet};

use argus_board::Board;
use argus_core::clock::parse_ms;
use argus_worktrees::{
    Answer, BoardFacts, EntryWorktree, ItemFact, Keep, OwnerEntry, Rule, ThreadInfo, WorktreeLink,
};
use serde_json::Value;

use super::links::{agents_of, links_of};

const ENDED: [&str; 2] = ["done", "archived"];

fn text(value: &Value) -> Option<String> {
    value.as_str().map(str::to_owned)
}

pub(super) fn rules_of(raw: &Value) -> Vec<Rule> {
    raw.as_array()
        .into_iter()
        .flatten()
        .filter_map(|r| serde_json::from_value(r.clone()).ok())
        .collect()
}

fn keeps_of(raw: &Value) -> HashMap<String, Keep> {
    let keep = |k: &Value| {
        Some(Keep {
            until_ms: parse_ms(k["until"].as_str()?)?,
            note: text(&k["note"]),
            by: text(&k["by"]),
        })
    };
    raw.as_object()
        .into_iter()
        .flatten()
        .filter_map(|(id, k)| Some((id.clone(), keep(k)?)))
        .collect()
}

fn answers_of(raw: &Value) -> HashMap<String, Answer> {
    let answer = |a: &Value| {
        Some(Answer {
            note: text(&a["note"])?,
            at_ms: parse_ms(a["at"].as_str()?)?,
            by: text(&a["by"]),
        })
    };
    raw.as_object()
        .into_iter()
        .flatten()
        .filter_map(|(id, a)| Some((id.clone(), answer(a)?)))
        .collect()
}

/// Registered and not ended, not a placeholder, not deleted.
pub(super) fn thread_live(t: &Value) -> bool {
    t.is_object()
        && !ENDED.contains(&t["status"].as_str().unwrap_or(""))
        && t["placeholder"] != true
        && t["deletedAt"].is_null()
}

fn threads_of(raw: &Value) -> BTreeMap<String, ThreadInfo> {
    raw.as_object()
        .into_iter()
        .flatten()
        .map(|(key, t)| {
            let id = t["id"].as_str().unwrap_or(key).to_owned();
            let info = ThreadInfo {
                id,
                lead: t["role"] == LEAD_ROLE,
                live: thread_live(t),
            };
            (key.clone(), info)
        })
        .collect()
}

fn option_text(option: &Value) -> String {
    option
        .as_str()
        .or_else(|| option["text"].as_str())
        .or_else(|| option["label"].as_str())
        .unwrap_or_default()
        .to_owned()
}

fn worktree_of(raw: &Value) -> Option<EntryWorktree> {
    raw.is_object().then(|| EntryWorktree {
        id: raw["id"].as_str().unwrap_or_default().to_owned(),
        path: raw["path"].as_str().unwrap_or_default().to_owned(),
        kind: raw["kind"].as_str().unwrap_or_default().to_owned(),
        message: text(&raw["message"]),
    })
}

fn entry_of(id: &str, e: &Value) -> OwnerEntry {
    OwnerEntry {
        id: id.to_owned(),
        author: e["author"].as_str().unwrap_or_default().to_owned(),
        via: text(&e["via"]),
        kind: e["kind"].as_str().unwrap_or_default().to_owned(),
        deleted: !e["deletedAt"].is_null(),
        created_ms: e["createdAt"].as_str().and_then(parse_ms),
        text: e["text"].as_str().unwrap_or_default().to_owned(),
        worktree: worktree_of(&e["worktree"]),
        reply_to: text(&e["replyTo"]),
        context: text(&e["context"]),
        options: e["options"]
            .as_array()
            .map(|o| o.iter().map(option_text).collect())
            .unwrap_or_default(),
    }
}

/// The owner's page posts and the worktree requests, and the questions the posts answer.
fn entries_of(raw: &Value) -> HashMap<String, OwnerEntry> {
    let Some(all) = raw.as_object() else {
        return HashMap::new();
    };
    let ours =
        |e: &Value| e["worktree"].is_object() || (e["author"] == "human" && e["via"] == "page");
    let asked: HashSet<&str> = all
        .values()
        .filter(|e| ours(e))
        .filter_map(|e| e["replyTo"].as_str())
        .collect();
    all.iter()
        .filter(|(id, e)| ours(e) || asked.contains(id.as_str()))
        .map(|(id, e)| (id.clone(), entry_of(id, e)))
        .collect()
}

/// Every item with its lane, whether it is in the trash and its thread. The text is what may name a worktree; the thread
/// is the lead the hygiene asks about a worktree linked to it.
fn items_of(raw: &Value) -> BTreeMap<String, ItemFact> {
    let item = |id: &String, rec: &Value| ItemFact {
        id: id.clone(),
        lane: rec["status"].as_str().unwrap_or("todo").to_owned(),
        trashed: !rec["deletedAt"].is_null(),
        thread: rec["thread"]
            .as_str()
            .filter(|t| !t.is_empty())
            .map(str::to_owned),
        text: format!(
            "{} {}",
            rec["title"].as_str().unwrap_or_default(),
            rec["description"].as_str().unwrap_or_default()
        ),
    };
    raw.as_object()
        .into_iter()
        .flatten()
        .filter(|(_, rec)| rec.is_object())
        .map(|(id, rec)| (id.clone(), item(id, rec)))
        .collect()
}

/// The explicit links of worktrees to items (`coord worktree link`), by worktree id.
fn worktree_links_of(raw: &Value) -> HashMap<String, WorktreeLink> {
    let link = |l: &Value| {
        Some(WorktreeLink {
            item: text(&l["item"]).filter(|i| !i.is_empty())?,
            by: text(&l["by"]),
            at_ms: l["at"].as_str().and_then(parse_ms),
        })
    };
    raw.as_object()
        .into_iter()
        .flatten()
        .filter_map(|(id, l)| Some((id.clone(), link(l)?)))
        .collect()
}

/// The facts of a board.
pub(super) fn facts_of(board: &Board) -> BoardFacts {
    BoardFacts {
        name: text(&board["name"]),
        rules: rules_of(&board["worktreeRules"]),
        keeps: keeps_of(&board["worktreeKeeps"]),
        answers: answers_of(&board["worktreeAnswers"]),
        threads: threads_of(&board["threads"]),
        entries: entries_of(&board["entries"]),
        agents: agents_of(board),
        links: links_of(board),
        items: items_of(&board["items"]),
        worktree_links: worktree_links_of(&board["worktreeLinks"]),
    }
}

// test-tag: critical
#[cfg(test)]
mod tests {
    //! K1: what the worktree service reads of items and links.
    use super::*;
    use serde_json::json;

    #[test]
    fn the_items_carry_their_lane_the_trash_and_their_text_and_the_links_their_item() {
        let board: Board = json!({
            "items": {
                "I-1": {"status": "doing", "title": "land agent-a1", "description": "soon"},
                "I-2": {"status": "done", "title": "old", "deletedAt": null},
                "I-3": {"status": "todo", "title": "gone", "deletedAt": "2026-11-01T10:00:00.000Z"}
            },
            "worktreeLinks": {
                "abc": {"item": "I-2", "by": "lead", "at": "2026-11-01T10:00:00.000Z"},
                "bad": {"by": "lead"}
            }
        });
        let facts = facts_of(&board);
        let lanes: Vec<(&str, bool, bool)> = facts
            .items
            .iter()
            .map(|(id, i)| (id.as_str(), i.closed(), i.trashed))
            .collect();
        assert_eq!(
            lanes,
            [
                ("I-1", false, false),
                ("I-2", true, false),
                ("I-3", true, true)
            ]
        );
        assert_eq!(facts.items["I-1"].text, "land agent-a1 soon");
        assert_eq!(facts.worktree_links.len(), 1);
        let link = &facts.worktree_links["abc"];
        assert_eq!(
            (link.item.as_str(), link.by.as_deref(), link.at_ms.is_some()),
            ("I-2", Some("lead"), true)
        );
    }
}
