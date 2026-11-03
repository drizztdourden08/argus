//! `coord item block <I> --on <id> | --todo "<what>" --who <who>` and `coord item unblock <I>
//! [--from <id>]`. A blocker is an id of this board: a record that is open, or a to-do made here
//! for whoever must act. Free text is refused with what to do instead.

use argus_board::Board;
use serde_json::{Map, Value, json};

use super::edit::{Plan, Say, edit_item};
use crate::cli::Outcome;
use crate::verbs::work::crud::apply_result;
use crate::verbs::work::run::{Res, Run, bad};
use crate::who::Caller;

const USAGE: &str = "usage: coord item block <I-n> --on <I-n|T-n|H-n|Q-n|D-n> | --todo \"<what>\" --who <owner|thread|lead:key>.";

/// What blocks the item.
enum Blocker {
    /// A record of this board.
    Link(String),
    /// A to-do to make for `who`.
    Todo { what: String, who: String },
}

/// `PREFIX-digits`: the shape of every id on a board.
pub(super) fn is_id(word: &str) -> bool {
    word.split_once('-').is_some_and(|(prefix, n)| {
        !prefix.is_empty()
            && prefix.chars().all(|c| c.is_ascii_uppercase())
            && !n.is_empty()
            && n.chars().all(|c| c.is_ascii_digit())
    })
}

/// The refusal of an `--on` that is not an id of this board.
fn refuse_on(on: &str, item: &str) -> String {
    if let Some((key, _)) = on.split_once('/').filter(|(_, id)| is_id(id)) {
        return format!("{on} is on another board: coord dep ask {key} \"<what>\" --item {item}.");
    }
    format!(
        "a blocker is an id, not text: \"{on}\". Link a record of this board with --on <I-n|T-n|H-n|Q-n|D-n>, or make a to-do for whoever must act: coord item block {item} --todo \"<what>\" --who owner|<thread>|lead:<key>. Another lead's work is coord dep ask <key> \"<what>\" --item {item}."
    )
}

fn given(r: &Run<'_>, name: &str) -> Option<String> {
    r.text_opt(name)
        .map(argus_core::js::trim)
        .filter(|t| !t.is_empty())
        .map(str::to_owned)
}

fn wanted(r: &Run<'_>) -> Res<Blocker> {
    let item = r.arg(0).unwrap_or_default();
    match (given(r, "on"), given(r, "todo"), given(r, "who")) {
        (Some(on), None, None) if is_id(&on) => Ok(Blocker::Link(on)),
        (Some(on), None, None) => Err(bad(refuse_on(&on, item))),
        (None, Some(what), Some(who)) => Ok(Blocker::Todo { what, who }),
        (None, Some(_), None) => Err(bad(
            "--todo needs --who <owner|thread|lead:key>: who must act.",
        )),
        _ => Err(bad(USAGE)),
    }
}

/// The ids that block the item now.
fn links_of(item: &Value) -> Vec<Value> {
    item["blockedBy"].as_array().cloned().unwrap_or_default()
}

pub(super) fn linked(item: &Value, link: &str) -> Value {
    let mut links = links_of(item);
    if !links.iter().any(|l| l.as_str() == Some(link)) {
        links.push(json!(link));
    }
    Value::Array(links)
}

/// Makes the to-do about the item for `who` and gives its id.
pub(super) fn make_todo(
    board: &mut Board,
    by: &Caller,
    item: &Value,
    (what, who): (&str, &str),
) -> Res<String> {
    let operation = json!({
        "kind": "create", "target": {"c": "actions"},
        "set": {"text": what, "item": item["id"], "assignee": who},
    });
    let res = apply_result(board, by, &operation)?;
    res.extra
        .get("created")
        .and_then(|c| c["id"].as_str())
        .map(str::to_owned)
        .ok_or_else(|| bad("the to-do was not created."))
}

fn plan_for(wanted: Blocker) -> impl FnOnce(&mut Board, &Caller, &Value) -> Res<Plan> {
    move |board, by, item| {
        let (link, line) = match &wanted {
            Blocker::Link(id) => (id.clone(), format!("blocked on {id}.")),
            Blocker::Todo { what, who } => {
                let id = make_todo(board, by, item, (what.as_str(), who.as_str()))?;
                let line = format!("blocked on {id} (to-do for {who}).");
                (id, line)
            }
        };
        let set = Map::from_iter([("blockedBy".to_owned(), linked(item, &link))]);
        Ok(Plan {
            set,
            say: Say::Line(line),
        })
    }
}

/// `coord item block <I> --on <id> | --todo "<what>" --who <who>`.
pub(super) fn block(r: &mut Run<'_>) -> Outcome {
    let wanted = wanted(r)?;
    edit_item(r, Vec::new(), plan_for(wanted))
}

/// One link dropped, refused when the item is not blocked on it.
fn drop_one(item: &Value, from: &str) -> Res<Plan> {
    let rest: Vec<Value> = links_of(item)
        .into_iter()
        .filter(|l| l.as_str() != Some(from))
        .collect();
    if rest.len() == links_of(item).len() {
        let id = item["id"].as_str().unwrap_or_default();
        return Err(bad(format!("{id} is not blocked on {from}.")));
    }
    Ok(Plan {
        set: Map::from_iter([("blockedBy".to_owned(), Value::Array(rest))]),
        say: Say::Line(format!("no longer blocked on {from}.")),
    })
}

/// Every link and the legacy text dropped.
fn drop_all(item: &Value) -> Plan {
    let mut set = Map::from_iter([("blocker".to_owned(), Value::Null)]);
    if !links_of(item).is_empty() {
        set.insert("blockedBy".to_owned(), json!([]));
    }
    Plan {
        set,
        say: Say::Line("blocker cleared.".to_owned()),
    }
}

/// `coord item unblock <I> [--from <id>]`.
pub(super) fn unblock(r: &mut Run<'_>) -> Outcome {
    let from = given(r, "from");
    edit_item(r, Vec::new(), move |_, _, item| match &from {
        Some(id) => drop_one(item, id),
        None => Ok(drop_all(item)),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    // test-tag: critical
    #[test]
    fn an_id_is_a_prefix_and_digits_and_nothing_else() {
        assert!(is_id("I-12"));
        assert!(is_id("D-3"));
        assert!(!is_id("external: owner, a test"));
        assert!(!is_id("I-"));
        assert!(!is_id("-3"));
        assert!(!is_id("i-3"));
        assert!(!is_id("brock/I-3"));
    }

    #[test]
    fn a_text_gets_the_hint_and_another_board_the_dep_ask() {
        let text = refuse_on("external: owner, a go", "I-4");
        assert!(text.contains("a blocker is an id, not text"), "{text}");
        assert!(text.contains("coord item block I-4 --todo"), "{text}");
        let other = refuse_on("brock/I-9", "I-4");
        assert_eq!(
            other,
            "brock/I-9 is on another board: coord dep ask brock \"<what>\" --item I-4."
        );
    }
}
