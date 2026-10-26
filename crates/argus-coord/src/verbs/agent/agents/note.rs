//! `coord agent note <A-id|name> "<text>"`: the line the board shows for a running agent.

use argus_board::json::set;
use argus_board::limits::require_str;
use argus_board::store::stamp;

use super::find::{Want, own};
use crate::cli::Call;
use crate::cli::exit::Outcome;
use crate::verbs::agent::base::{opt::word, write};

/// `coord agent note`.
///
/// # Errors
/// An empty or long note, no such running agent, an agent of another thread.
pub fn run(call: &mut Call<'_>) -> Outcome {
    let line = require_str("agentnote", word(&call.positional, 1))?;
    write(call, |board, who, run| {
        let (id, agent) = own(board, who, run, ("set", Want::Live))?;
        set(agent, "note", line.as_str());
        stamp(board, "agents", &id, &who.actor)?;
        run.say(&format!("{id}: {line}"));
        Ok(())
    })
}
