//! Typed items. An item may carry a `type`, a `parent`, an `assignee`, a `stage`, the type's own
//! `fields`, the `blockedBy` links and `images`; the registry says which values are valid. An item
//! that names none of them is a plain task and behaves exactly as it always did.
//!
//! * [`edit`]: the edit of an item, the typed fields checked together.
//! * [`create`]: a new item that names typed fields.
//! * [`shift`]: an item changing stage, for an edit and for a move.
//! * [`enter`]: the flow of a type, the gates of a stage and the done rule.
//! * [`note`]: the note entry.

mod assignee;
mod blocked_by;
mod create;
mod description;
mod edit;
mod enter;
mod force;
mod images;
mod keys;
mod kind;
mod known;
mod needs;
mod note;
mod outcome;
mod retype;
mod shift;
mod stage;
mod tree;
mod values;

pub use note::create_note;

pub(crate) use assignee::{hint as assignee_hint, stamp_todo, todo as todo_assignee};
pub(crate) use blocked_by::check as blocked_by_list;
pub(crate) use create::{item as create_item, wanted as wants_create};
pub(crate) use description::written as description_text;
pub(crate) use edit::{applies as edits, item_edit};
pub(crate) use note::rewrite as note_text;
pub(crate) use outcome::Outcome;
pub(crate) use shift::on_move;
