//! How a group of sub-verbs is served: the table of handlers, the four soft verbs of the groups
//! that have them, and the refusal that lists what the group knows.

use argus_board::Board;
use serde_json::Value;

use super::run::{Handler, Run};
use super::soft;
use crate::cli::{Call, CoordError, Outcome};

/// What a soft verb does to the board once a record changed state.
pub type After = fn(&mut Board, &Value);

/// The soft verbs (`archive`, `unarchive`, `rm`, `restore`) of one collection.
pub struct Soft {
    /// The collection.
    pub coll: &'static str,
    /// What runs after each record.
    pub after: After,
}

/// The soft verbs of a collection that needs nothing after a change.
pub fn nothing(_board: &mut Board, _doc: &Value) {}

/// A verb module: its name, its handlers and the sub-verbs it lists in the refusal.
pub struct Group {
    /// The module name, as `coord.mjs` calls it.
    pub name: &'static str,
    /// The handlers by sub-verb.
    pub verbs: &'static [(&'static str, Handler)],
    /// The soft verbs, when the group has them.
    pub soft: Option<Soft>,
    /// The sub-verbs in the order of the refusal.
    pub names: &'static str,
    /// The typed verb that is itself followed by a sub-verb (`entry`).
    pub typed: Option<&'static str>,
}

/// Serves one call: the sub-verb picks the handler.
///
/// # Errors
/// Refused for an unknown sub-verb, and whatever the handler refuses with.
pub fn serve(call: &mut Call<'_>, group: &Group) -> Outcome {
    let mut run = Run::of(call)?;
    let mut sub = run.shift();
    if group.typed.is_some() && sub.as_deref() == group.typed {
        sub = run.shift();
    }
    let sub = sub.unwrap_or_default();
    if let Some((_, handler)) = group.verbs.iter().find(|(name, _)| *name == sub) {
        return handler(&mut run);
    }
    match &group.soft {
        Some(kind) if soft::is_soft(&sub) => soft::run(&mut run, kind, &sub),
        _ => Err(CoordError::refuse(format!(
            "{} verb must be one of {}.",
            group.name, group.names
        ))),
    }
}
