//! A new item that names typed fields: the type, parent, assignee, stage, own fields, blockers
//! and images, checked together, with the title, description and criteria checked by the
//! registry.

use serde_json::{Map, Value, json};

use super::keys::valued_in;
use super::kind::{DEFAULT_TYPE, check_type, named};
use super::known::Known;
use super::outcome::Outcome;
use super::shift::{Change, shift};
use super::{assignee, blocked_by, images, tree, values};
use crate::Board;
use crate::content::check as content_check;
use crate::error::BoardError;
use crate::ops::Who;
use crate::ops::target::{ITEM_STATUSES, one_of};
use crate::registry::ItemType;

/// Whether a create gives a typed field a value.
pub(crate) fn wanted(set: &Map<String, Value>) -> bool {
    valued_in(set)
}

struct Fresh<'a> {
    board: &'a Board,
    set: &'a Map<String, Value>,
    ty: &'static ItemType,
    known: Known,
    out: Map<String, Value>,
}

impl Fresh<'_> {
    fn keep(&mut self, key: &str, value: Value) {
        self.out.insert(key.to_owned(), value);
    }

    fn keep_set(&mut self, key: &str, value: Value) {
        if !value.is_null() {
            self.keep(key, value);
        }
    }

    fn given(&self, key: &str) -> Option<&Value> {
        self.set.get(key).filter(|v| !v.is_null())
    }

    fn texts(&mut self) -> Result<(), BoardError> {
        let ctx = self.known.context();
        let get = |key: &str| self.set.get(key).cloned().unwrap_or(Value::Null);
        let title = content_check("title", &get("title"), &ctx)?;
        let description = content_check("description", &get("description"), &ctx)?;
        let ac = content_check("ac", &get("ac"), &ctx)?;
        let criteria: Vec<Value> = ac
            .as_array()
            .into_iter()
            .flatten()
            .zip(1_u32..)
            .map(|(text, n)| json!({"n": n, "text": text, "done": false}))
            .collect();
        self.keep("title", title);
        self.keep(
            "description",
            Value::String(description.as_str().unwrap_or_default().to_owned()),
        );
        self.keep("ac", Value::Array(criteria));
        Ok(())
    }

    fn links(&mut self) -> Result<(), BoardError> {
        if let Some(v) = self.given("parent") {
            let parent = tree::check_parent(self.board, None, &self.ty.id, v)?;
            self.keep_set("parent", parent);
        }
        if let Some(v) = self.given("assignee") {
            let who = assignee::check(self.board, v)?;
            self.keep_set("assignee", who);
        }
        if let Some(v) = self.given("blockedBy") {
            let links = blocked_by::check(self.board, None, v)?;
            self.keep("blockedBy", links);
        }
        Ok(())
    }

    fn own_fields(&mut self) -> Result<(), BoardError> {
        let given = values::given(self.set)?;
        let merged = values::merge(&self.known, self.ty, &Value::Null, &given)?;
        values::require(self.ty, &merged)?;
        if !given.is_empty() {
            self.keep("fields", merged);
        }
        Ok(())
    }

    /// The lane, and the stage when one is named. A stage other than the first is checked as a
    /// move from the first.
    fn place(&mut self, who: &Who, op: &Value) -> Result<Outcome, BoardError> {
        let set = self.set;
        let status = match set.get("status") {
            Some(v) => one_of(&ITEM_STATUSES, v, "status")?,
            None => json!("todo"),
        };
        self.keep("status", status);
        let draft = Value::Object(self.out.clone());
        let start = json!({"status": "todo"});
        let change = Change {
            board: self.board,
            ty: self.ty,
            item: &draft,
            before: &start,
            who,
            op,
        };
        let Some(found) = shift(&change, set.get("status"), set.get("stage"))? else {
            return Ok(Outcome::default());
        };
        self.keep("status", json!(found.stage.lane));
        if self.given("stage").is_some() {
            self.keep("stage", json!(found.stage.id));
        }
        Ok(found.outcome)
    }
}

fn type_of_new(set: &Map<String, Value>) -> Result<&'static ItemType, BoardError> {
    match set.get("type").filter(|v| !v.is_null()) {
        Some(v) => check_type(v),
        None => named(DEFAULT_TYPE),
    }
}

/// The fields of a new item that names typed fields, ready to store with its thread and origin.
pub(crate) fn item(
    board: &Board,
    (who, op): (&Who, &Value),
    set: &Map<String, Value>,
    (owner, origin): (&str, &str),
) -> Result<(Map<String, Value>, Outcome), BoardError> {
    let pictures = images::check(set.get("images").unwrap_or(&Value::Null))?;
    let mut fresh = Fresh {
        board,
        set,
        ty: type_of_new(set)?,
        known: Known::of(board, images::numbers(&pictures)),
        out: Map::new(),
    };
    fresh.texts()?;
    fresh.keep("thread", json!(owner));
    fresh.keep("origin", json!(origin));
    if set.get("type").is_some_and(|v| !v.is_null()) {
        fresh.keep("type", json!(fresh.ty.id));
    }
    fresh.links()?;
    fresh.own_fields()?;
    if pictures.as_array().is_some_and(|p| !p.is_empty()) {
        fresh.keep("images", pictures);
    }
    let outcome = fresh.place(who, op)?;
    Ok((fresh.out, outcome))
}
