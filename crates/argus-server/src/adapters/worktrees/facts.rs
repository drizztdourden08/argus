//! A project's board as the worktree service reads it: the rules, the keeps, the thread's
//! answers, the threads and the owner entries that matter to a removal.

use std::collections::{BTreeMap, HashMap, HashSet};

use argus_board::Board;
use argus_core::clock::parse_ms;
use argus_worktrees::{Answer, BoardFacts, EntryWorktree, Keep, OwnerEntry, Rule, ThreadInfo};
use serde_json::Value;

use super::links::{agents_of, links_of};

const ENDED: [&str; 2] = ["done", "archived"];

fn text(value: &Value) -> Option<String> {
    value.as_str().map(str::to_owned)
}

fn rules_of(raw: &Value) -> Vec<Rule> {
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
                coordinator: t["role"] == "coordinator",
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
    }
}
