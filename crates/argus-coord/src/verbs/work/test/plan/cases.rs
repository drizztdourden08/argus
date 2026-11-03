//! `coord test plan cases I-n --add T-n [--before T-m] | --remove T-n`: the ordered cases of a
//! plan. Several `--add` and `--remove` may be given; the removals go first.

use argus_board::Board;
use argus_board::json::display;
use serde_json::{Map, Value, json};

use super::{cases_of, plan_of, send};
use crate::cli::{Options, Outcome};
use crate::verbs::work::crud::op;
use crate::verbs::work::run::{Res, Run, bad};
use crate::who::Caller;

/// The ids an option names, in the order given.
fn named(options: &Options, name: &str) -> Vec<String> {
    options
        .many(name)
        .iter()
        .map(|v| display(&v.to_json()))
        .collect()
}

fn remove_all(list: &mut Vec<String>, plan: &str, gone: &[String]) -> Res<()> {
    for case in gone {
        let Some(at) = list.iter().position(|c| c == case) else {
            return Err(bad(format!("{case} is not a case of {plan}.")));
        };
        list.remove(at);
    }
    Ok(())
}

fn add_all(
    list: &mut Vec<String>,
    plan: &str,
    (new, before): (&[String], Option<&str>),
) -> Res<()> {
    for case in new {
        if list.contains(case) {
            return Err(bad(format!("{case} is already a case of {plan}.")));
        }
        let at = match before {
            Some(next) => list.iter().position(|c| c == next).ok_or_else(|| {
                bad(format!(
                    "{next} is not a case of {plan}, so nothing goes before it."
                ))
            })?,
            None => list.len(),
        };
        list.insert(at, case.clone());
    }
    Ok(())
}

/// The cases after the removals and the additions.
fn reshaped(plan: &Value, options: &Options) -> Res<Vec<String>> {
    let (new, gone) = (named(options, "add"), named(options, "remove"));
    if new.is_empty() && gone.is_empty() {
        return Err(bad(
            "usage: coord test plan cases <I-n> --add <T-n> [--before <T-m>] | --remove <T-n>.",
        ));
    }
    let id = display(&plan["id"]);
    let mut list = cases_of(plan);
    remove_all(&mut list, &id, &gone)?;
    add_all(&mut list, &id, (&new, options.text("before")))?;
    Ok(list)
}

fn change(
    board: &mut Board,
    who: &Caller,
    (plan, list): (&Value, Vec<String>),
    options: &Options,
) -> Res<String> {
    let id = display(&plan["id"]);
    let set = Map::from_iter([("fields.cases".to_owned(), json!(list))]);
    let rest = Map::from_iter([("set".to_owned(), Value::Object(set))]);
    send(board, who, &op(("edit", "items", &id), rest, options)?)?;
    let now = plan_of(board, Some(&id))?;
    Ok(format!(
        "{id}: {} case(s): {}.",
        cases_of(&now).len(),
        cases_of(&now).join(", ")
    ))
}

/// `coord test plan cases`.
pub(super) fn cases(r: &mut Run<'_>) -> Outcome {
    r.write(|board, r| {
        let who = r.begin(board)?;
        let plan = plan_of(board, r.arg(0))?;
        let list = reshaped(&plan, r.options)?;
        let line = change(board, &who, (&plan, list), r.options)?;
        r.say(&[&line]);
        Ok(())
    })
}

/// Add a test to the end of a plan's cases (`coord test add ... --plan I-n`).
pub(in crate::verbs::work::test) fn append(
    board: &mut Board,
    who: &Caller,
    (plan_id, test): (&str, &str),
    options: &Options,
) -> Res<()> {
    let plan = plan_of(board, Some(plan_id))?;
    let mut list = cases_of(&plan);
    add_all(&mut list, plan_id, (&[test.to_owned()], None))?;
    change(board, who, (&plan, list), options).map(|_| ())
}
