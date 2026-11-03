//! `coord item tree [I-n] [--type t] [--open] [--assignee who|me]`: the items as a tree, one line
//! each: `I-90 [epic] active  ME  Release 2  7/10`, the children indented under their parent.

use std::collections::{HashMap, HashSet};

use argus_board::Board;
use argus_board::json::{display, truthy};
use argus_board::registry::{Progress, registry};
use serde_json::Value;

use super::filters::{assignee_of, narrowed, type_of};
use super::marks::mark_of;
use crate::cli::Outcome;
use crate::verbs::work::crud::find_live;
use crate::verbs::work::listing::visible_docs;
use crate::verbs::work::place::by_rank;
use crate::verbs::work::run::{Res, Run};

/// The most levels printed, a guard against a tree that is not one.
const DEPTH: usize = 12;

/// The shown items by parent id, and the ids the filters keep.
struct Forest {
    children: HashMap<String, Vec<Value>>,
    keep: HashSet<String>,
}

fn id_of(item: &Value) -> String {
    display(&item["id"])
}

fn ranked(mut items: Vec<Value>) -> Vec<Value> {
    items.sort_by(by_rank);
    items
}

impl Forest {
    fn of(shown: &[Value], kept: &[Value]) -> Forest {
        let mut children: HashMap<String, Vec<Value>> = HashMap::new();
        for item in shown {
            if let Some(parent) = item["parent"].as_str() {
                children
                    .entry(parent.to_owned())
                    .or_default()
                    .push(item.clone());
            }
        }
        Forest {
            children: children.into_iter().map(|(k, v)| (k, ranked(v))).collect(),
            keep: kept.iter().map(id_of).collect(),
        }
    }

    fn below(&self, id: &str) -> &[Value] {
        self.children.get(id).map_or(&[][..], Vec::as_slice)
    }

    /// Done over all but the dropped children, or nothing without children.
    fn kids(&self, id: &str) -> String {
        let live: Vec<&Value> = self
            .below(id)
            .iter()
            .filter(|c| c["status"] != "dropped")
            .collect();
        let done = live.iter().filter(|c| c["status"] == "done").count();
        if live.is_empty() {
            String::new()
        } else {
            format!("{done}/{}", live.len())
        }
    }
}

fn criteria(item: &Value) -> String {
    let list = item["ac"].as_array().map_or(&[][..], Vec::as_slice);
    let done = list.iter().filter(|c| truthy(c, "done")).count();
    if list.is_empty() {
        String::new()
    } else {
        format!("{done}/{}", list.len())
    }
}

/// What the card of the type shows as progress.
fn progress(forest: &Forest, item: &Value) -> String {
    let kind = registry()
        .item_type(type_of(item))
        .map_or(Progress::None, |t| t.card.progress);
    match kind {
        Progress::Ac => criteria(item),
        Progress::Children => forest.kids(&id_of(item)),
        Progress::ChildrenAc => {
            let kids = forest.kids(&id_of(item));
            if kids.is_empty() {
                criteria(item)
            } else {
                kids
            }
        }
        Progress::Run | Progress::Checks | Progress::None => String::new(),
    }
}

fn line(forest: &Forest, item: &Value, depth: usize) -> String {
    let stage = item["stage"].as_str().or(item["status"].as_str());
    let head = format!(
        "{}{} [{}] {}  {}  {}",
        "  ".repeat(depth),
        id_of(item),
        type_of(item),
        stage.unwrap_or_default(),
        mark_of(assignee_of(item)),
        display(&item["title"])
    );
    match progress(forest, item).as_str() {
        "" => head,
        shown => format!("{head}  {shown}"),
    }
}

fn walk(forest: &Forest, item: &Value, depth: usize, out: &mut Vec<String>) {
    let shown = forest.keep.contains(&id_of(item));
    if shown {
        out.push(line(forest, item, depth));
    }
    if depth >= DEPTH {
        return;
    }
    let next = depth + usize::from(shown);
    for child in forest.below(&id_of(item)) {
        walk(forest, child, next, out);
    }
}

/// The items the tree starts from: the one named, else every item with no shown parent.
fn roots(board: &Board, r: &Run<'_>, shown: &[Value]) -> Res<Vec<Value>> {
    let named = r
        .arg(0)
        .or_else(|| r.text_opt("open").filter(|t| t.starts_with("I-")));
    if let Some(id) = named {
        return Ok(vec![find_live(board, "items", Some(id))?]);
    }
    let ids: HashSet<String> = shown.iter().map(id_of).collect();
    let top = shown
        .iter()
        .filter(|i| i["parent"].as_str().is_none_or(|p| !ids.contains(p)))
        .cloned()
        .collect();
    Ok(ranked(top))
}

/// `coord item tree`.
pub(super) fn tree(r: &mut Run<'_>) -> Outcome {
    r.read(|board, r| {
        let shown = visible_docs(board, "items", r.options);
        let kept = narrowed(board, r, shown.clone())?;
        let forest = Forest::of(&shown, &kept);
        let mut lines: Vec<String> = Vec::new();
        for root in roots(board, r, &shown)? {
            walk(&forest, &root, 0, &mut lines);
        }
        if lines.is_empty() {
            lines.push("no items.".to_owned());
        }
        r.say(&[&lines.join("\n")]);
        Ok(())
    })
}
