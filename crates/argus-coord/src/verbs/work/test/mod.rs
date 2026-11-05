//! `coord test`: one doc per test, attached to a work item, in the story-events test-log shape:
//! what it covers, why (context), what changed, the setup and the run command, numbered steps,
//! what proves it works and what proves it is broken, notes, and the result with picture proof. A
//! passing result ticks the item's acceptance criteria the test covers. A test belongs to the
//! thread that owns its item: that thread changes it and records its results, the coordinator
//! can do it all for any thread.

mod add;
mod edit;
mod parts;
mod plan;
mod proof;
mod read;
mod result;
mod runs;
mod spec;

pub use runs::alias;

use super::group::{Group, Soft, nothing, serve};
use crate::cli::{Call, Outcome};

const GROUP: Group = Group {
    name: "test",
    verbs: &[
        ("add", add::add),
        ("set", add::set_fields),
        ("step", parts::step),
        ("setup", parts::setup),
        ("expect", parts::expect),
        ("proof", proof::proof),
        ("result", result::result),
        ("plan", plan::plan),
        ("run", runs::run),
        ("move", result::move_to),
        ("show", read::show),
        ("list", read::list),
    ],
    soft: Some(Soft {
        coll: "tests",
        after: nothing,
    }),
    names: "add, set, step, setup, expect, proof, result, move, plan, run, archive, unarchive, rm, restore, show, list",
    typed: None,
};

/// `coord test <sub-verb> ...`.
///
/// # Errors
/// Refused as the sub-verb refuses.
pub fn handle(call: &mut Call<'_>) -> Outcome {
    serve(call, &GROUP)
}
