//! The edit of an item that names typed fields. A type, a parent, a stage and the type's own
//! fields depend on each other, so they are checked together, and the result is the complete set
//! of fields to write.

use serde_json::{Map, Value};

use super::keys::{is_key, named_in};
use super::kind::{check_type, is_typed, type_id, type_of};
use super::known::Known;
use super::outcome::Outcome;
use super::shift::{Change, shift};
use super::stage::current;
use super::{assignee, blocked_by, images, retype, tree, values};
use crate::content::check as content_check;
use crate::error::{BoardError, refuse};
use crate::json::{display, set as put_value};
use crate::ops::authorize::{Check, authorize};
use crate::ops::fields::{EditCtx, check, editable};
use crate::registry::ItemType;

/// The fields an edit writes, and what it let through.
pub(crate) struct Edited {
    pub next: Map<String, Value>,
    pub outcome: Outcome,
}

/// Whether an edit of an item goes through the typed rules: the item names a type or a stage, or
/// the edit does, or the edit sets a typed field.
pub(crate) fn applies(item: &Value, set: &Map<String, Value>) -> bool {
    is_typed(item) || named_in(set)
}

/// Whether the item, as this edit leaves it, follows the rules of its type.
fn follows_type(item: &Value, set: &Map<String, Value>) -> bool {
    let named = |key: &str| set.get(key).is_some_and(|v| !v.is_null());
    is_typed(item) || named("type") || named("stage")
}

struct Work<'a> {
    cx: &'a EditCtx<'a>,
    set: &'a Map<String, Value>,
    op: &'a Value,
    ty: &'static ItemType,
    id: String,
    follows: bool,
    known: Known,
    draft: Value,
    next: Map<String, Value>,
    outcome: Outcome,
}

impl Work<'_> {
    /// Write a field into the item as it will be, and into the fields to save.
    fn put(&mut self, key: &str, value: Value) {
        put_value(&mut self.draft, key, value.clone());
        self.next.insert(key.to_owned(), value);
    }

    fn retyping(&self) -> bool {
        self.ty.id != type_id(self.cx.doc)
    }

    fn kind(&mut self) -> Result<(), BoardError> {
        if matches!(self.set.get("type"), None | Some(Value::Null)) {
            return Ok(());
        }
        if self.retyping() {
            retype::children_fit(self.cx.board, &self.id, self.ty)?;
        }
        self.put("type", Value::String(self.ty.id.clone()));
        Ok(())
    }

    fn parent(&mut self) -> Result<(), BoardError> {
        match self.set.get("parent") {
            Some(v) => {
                let parent =
                    tree::check_parent(self.cx.board, Some(self.id.as_str()), &self.ty.id, v)?;
                self.put("parent", parent);
            }
            None if self.retyping() => retype::parent_fits(self.cx.board, &self.draft, self.ty)?,
            None => {}
        }
        Ok(())
    }

    fn values(&mut self) -> Result<(), BoardError> {
        let given = values::given(self.set)?;
        if !given.is_empty() {
            let merged = values::merge(&self.known, self.ty, &self.cx.doc["fields"], &given)?;
            self.put("fields", merged);
        }
        if self.retyping() {
            values::require(self.ty, &self.draft["fields"])?;
        }
        if self.retyping() || given.contains_key("cases") {
            values::keep_a_case(self.ty, &self.draft["fields"])?;
        }
        Ok(())
    }

    fn hand_to(&mut self, thread: &str) -> Result<(), BoardError> {
        if self.cx.doc["thread"].as_str() == Some(thread) {
            return Ok(());
        }
        let check = Check::new("move", "items", Some(self.cx.doc)).to(Some(thread));
        authorize(self.cx.who, &check)?;
        self.put("thread", Value::String(thread.to_owned()));
        Ok(())
    }

    fn people(&mut self) -> Result<(), BoardError> {
        let Some(v) = self.set.get("assignee") else {
            return Ok(());
        };
        let who = assignee::check(self.cx.board, v)?;
        if let Some(thread) = assignee::thread_of(self.cx.board, &who) {
            self.hand_to(&thread)?;
        }
        self.put("assignee", who);
        Ok(())
    }

    fn blockers(&mut self) -> Result<(), BoardError> {
        if let Some(v) = self.set.get("blockedBy") {
            let links = blocked_by::check(self.cx.board, Some(self.id.as_str()), v)?;
            self.put("blockedBy", links);
        }
        Ok(())
    }

    /// A text field of an item that follows its type is checked by the registry.
    fn registry_text(&self, key: &str, v: &Value) -> Result<Value, BoardError> {
        let checked = content_check(key, v, &self.known.context())?;
        if key == "description" && checked.is_null() {
            return Ok(Value::String(String::new()));
        }
        Ok(checked)
    }

    fn texts(&mut self) -> Result<(), BoardError> {
        let set = self.set;
        for (key, v) in set.iter().filter(|(k, _)| !is_key(k)) {
            let value = match key.as_str() {
                "status" if self.follows => continue,
                "title" if self.follows => self.registry_text(key, v)?,
                "description" => self.registry_text(key, v)?,
                _ => check("items", key, v, self.cx)?,
            };
            self.put(key, value);
        }
        Ok(())
    }

    fn stage(&mut self) -> Result<(), BoardError> {
        let set = self.set;
        let change = Change {
            board: self.cx.board,
            ty: self.ty,
            item: &self.draft,
            before: self.cx.doc,
            who: self.cx.who,
            op: self.op,
        };
        let asked = if self.follows {
            shift(&change, set.get("status"), set.get("stage"))?
        } else {
            None
        };
        if let Some(found) = asked {
            self.outcome = found.outcome;
            self.put("status", Value::String(found.stage.lane.clone()));
            self.put("stage", Value::String(found.stage.id.clone()));
        }
        Ok(())
    }

    /// A retype keeps the lane: the stage becomes the one of the new type that sits in it.
    fn restage(&mut self) {
        if !self.retyping() || self.next.contains_key("stage") {
            return;
        }
        if let Some(stage) = current(self.ty, &self.draft).map(|s| s.id.clone()) {
            self.put("stage", Value::String(stage));
        }
    }
}

/// The type an edit leaves the item with: the one it sets, else the one the item has.
fn type_after(item: &Value, set: &Map<String, Value>) -> Result<&'static ItemType, BoardError> {
    match set.get("type").filter(|v| !v.is_null()) {
        Some(v) => check_type(v),
        None => type_of(item),
    }
}

/// The fields of an edit of an item, checked together.
pub(crate) fn item_edit(
    cx: &EditCtx,
    set: &Map<String, Value>,
    op: &Value,
) -> Result<Edited, BoardError> {
    for field in set.keys() {
        if !editable("items", field) {
            return Err(refuse(format!("{field} cannot be edited on items.")));
        }
    }
    let pictures = match set.get("images") {
        Some(v) => Some(images::check(v)?),
        None => None,
    };
    let numbers = images::numbers(pictures.as_ref().unwrap_or(&cx.doc["images"]));
    let mut work = Work {
        cx,
        set,
        op,
        ty: type_after(cx.doc, set)?,
        id: display(&cx.doc["id"]),
        follows: follows_type(cx.doc, set),
        known: Known::of(cx.board, numbers),
        draft: cx.doc.clone(),
        next: Map::new(),
        outcome: Outcome::default(),
    };
    if let Some(pictures) = pictures {
        work.put("images", pictures);
    }
    work.kind()?;
    work.parent()?;
    work.values()?;
    work.people()?;
    work.blockers()?;
    work.texts()?;
    work.stage()?;
    work.restage();
    Ok(Edited {
        next: work.next,
        outcome: work.outcome,
    })
}
