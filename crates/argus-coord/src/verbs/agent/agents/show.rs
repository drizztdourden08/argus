//! `coord agent show <A-id|name>`: the whole record as JSON, with the source (`manual` when no
//! hook made it).

use argus_board::json::doc;
use serde_json::Value;

use super::find::{Want, find, is_id};
use crate::cli::Call;
use crate::cli::exit::{CoordError, Outcome};
use crate::verbs::agent::base::read;
use crate::who::caller;

/// `coord agent show`.
///
/// # Errors
/// No such agent, or no caller when the name needs one.
pub fn run(call: &mut Call<'_>) -> Outcome {
    read(call, |board, run| {
        let reference = run.positional.first().map(String::as_str);
        let by_name = !is_id(reference.unwrap_or_default()) && run.options.text("for").is_none();
        let who = if by_name {
            Some(caller(board, run.cwd, run.env, run.options)?)
        } else {
            None
        };
        let id = find(board, who.as_ref(), reference, run.options, Want::Any)?;
        let mut agent = doc(board, "agents", &id).cloned().unwrap_or(Value::Null);
        if agent["source"].is_null() {
            argus_board::json::set(&mut agent, "source", "manual");
        }
        let text = serde_json::to_string_pretty(&agent).map_err(|e| CoordError::refuse(e.to_string()))?;
        run.say(&text);
        Ok(())
    })
}
