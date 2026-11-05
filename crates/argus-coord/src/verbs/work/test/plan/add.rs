//! `coord test plan add "<title>" --case T-n [--case T-m]... [--parent I-n] [--build <launcher>]
//! [--for T]`. A plan always holds cases; `--build` also starts its first run on that launcher.

use argus_board::json::display;
use argus_board::limits::require_text;
use serde_json::{Map, Value, json};

use super::run::start_op;
use super::send;
use crate::cli::{Outcome, Val};
use crate::verbs::work::crud::{op, owner_arg};
use crate::verbs::work::run::{Res, Run, bad};

/// The cases of the new plan: at least one.
fn cases_of(r: &Run<'_>) -> Res<Vec<Value>> {
    let cases: Vec<Value> = r.options.many("case").iter().map(Val::to_json).collect();
    if cases.is_empty() {
        return Err(bad(
            "a test plan keeps at least one case: pass --case T-n (coord test add makes the case).",
        ));
    }
    Ok(cases)
}

fn set_of(r: &Run<'_>, (title, owner): (&str, &str), cases: &[Value]) -> Map<String, Value> {
    let mut set = Map::from_iter([
        ("title".to_owned(), json!(title)),
        ("type".to_owned(), json!("test-plan")),
        ("thread".to_owned(), json!(owner)),
        ("fields".to_owned(), json!({ "cases": cases })),
    ]);
    if let Some(parent) = r.text_opt("parent") {
        set.insert("parent".into(), json!(parent));
    }
    set
}

fn create_op(set: Map<String, Value>, r: &Run<'_>) -> Res<Value> {
    let rest = Map::from_iter([("set".to_owned(), Value::Object(set))]);
    op(("create", "items", ""), rest, r.options)
}

/// `coord test plan add`.
pub(super) fn add(r: &mut Run<'_>) -> Outcome {
    let title = require_text("title", &r.arg_json(0))?;
    if r.text_opt("bundle").is_some() {
        return Err(
            bad("--bundle is gone: a run carries its build, pass --build <launcher>.").into(),
        );
    }
    r.write(|board, r| {
        let who = r.begin(board)?;
        let owner = owner_arg(board, &who, r.options)?;
        let cases = cases_of(r)?;
        let count = cases.len();
        let operation = create_op(set_of(r, (&title, &owner), &cases), r)?;
        let res = send(board, &who, &operation)?;
        let id = display(&res.extra["created"]["id"]);
        r.say(&[&format!(
            "{id}: {title} (test plan, {owner}, {count} case(s))"
        )]);
        if r.text_opt("build").is_some() {
            let started = start_op(board, &who, &id, r)?;
            r.say(&[&format!("{started}.")]);
        }
        Ok(())
    })
}
