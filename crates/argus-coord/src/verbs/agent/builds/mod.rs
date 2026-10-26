//! `coord build`: bundles. A build tag with the tests that carry it, and the launcher the owner
//! starts it with. The owner says "the app I was handed is xyz" and tests everything tagged xyz.

mod close;
mod edit;
mod hand;
mod launch;
mod list;
mod parts;
mod remove;
mod rows;
mod status;
mod support;

use crate::cli::Call;
use crate::cli::exit::{CoordError, Outcome};

const VERBS: &str = "add, set, rm, close, reopen, ready, draft, hand, unhand, list";

/// Runs the sub-verb the first word names; the other words stay for it.
///
/// # Errors
/// A sub-verb that does not exist, or the refusal of the sub-verb.
pub fn run(call: &mut Call<'_>) -> Outcome {
    let sub = if call.positional.is_empty() {
        String::new()
    } else {
        call.positional.remove(0)
    };
    match sub.as_str() {
        "add" => edit::add(call),
        "set" => edit::set_fields(call),
        "rm" => remove::run(call),
        "close" => close::close(call),
        "reopen" => close::reopen(call),
        "ready" => status::run(call, true),
        "draft" => status::run(call, false),
        "hand" => hand::hand(call),
        "unhand" => hand::unhand(call),
        "list" => list::run(call),
        _ => Err(CoordError::refuse(format!(
            "build verb must be one of {VERBS}."
        ))),
    }
}
