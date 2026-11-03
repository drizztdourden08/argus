//! `coord test plan add "<title>" [--parent I-n] [--bundle tag] [--case T-n]... [--for T]`.

use argus_board::json::display;
use argus_board::limits::require_text;
use serde_json::{Map, Value, json};

use super::send;
use crate::cli::{Outcome, Val};
use crate::verbs::work::crud::{op, owner_arg};
use crate::verbs::work::run::{Res, Run};

/// The `fields` of the new plan: its bundle and its cases.
fn fields_of(r: &Run<'_>) -> Value {
    let mut fields = Map::new();
    if let Some(bundle) = r.text_opt("bundle") {
        fields.insert("bundle".into(), json!(bundle));
    }
    let cases: Vec<Value> = r.options.many("case").iter().map(Val::to_json).collect();
    if !cases.is_empty() {
        fields.insert("cases".into(), Value::Array(cases));
    }
    Value::Object(fields)
}

fn set_of(r: &Run<'_>, title: &str, owner: &str) -> Map<String, Value> {
    let mut set = Map::from_iter([
        ("title".to_owned(), json!(title)),
        ("type".to_owned(), json!("test-plan")),
        ("thread".to_owned(), json!(owner)),
        ("fields".to_owned(), fields_of(r)),
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
    r.write(|board, r| {
        let who = r.begin(board)?;
        let owner = owner_arg(board, &who, r.options)?;
        let operation = create_op(set_of(r, &title, &owner), r)?;
        let res = send(board, &who, &operation)?;
        let id = display(&res.extra["created"]["id"]);
        let cases = r.options.many("case").len();
        r.say(&[&format!(
            "{id}: {title} (test plan, {owner}, {cases} case(s))"
        )]);
        Ok(())
    })
}
