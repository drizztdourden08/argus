//! `coord agent done|fail|stop <A-id|name>`: the agent is finished. `done` takes an optional
//! result, `fail` needs the reason, `stop` also answers an owner's stop request.

use argus_board::agents::live_agent_ids;
use argus_board::json::{display, set};
use argus_board::limits::{optional_text, require_str};
use argus_board::store::stamp;
use argus_core::clock::now_iso;
use serde_json::Value;

use super::find::{Want, own};
use crate::cli::Call;
use crate::cli::exit::{CoordError, Outcome};
use crate::verbs::agent::base::{opt::word, write};

/// How an agent ends.
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Ending {
    /// `done`: finished its work.
    Done,
    /// `fail`: could not.
    Failed,
    /// `stop`: stopped.
    Stopped,
}

impl Ending {
    /// The verb that asks for it.
    fn verb(self) -> &'static str {
        match self {
            Ending::Done => "done",
            Ending::Failed => "fail",
            Ending::Stopped => "stop",
        }
    }

    fn status(self) -> &'static str {
        match self {
            Ending::Done => "done",
            Ending::Failed => "failed",
            Ending::Stopped => "stopped",
        }
    }
}

/// The result text of the call, checked before the board is touched.
fn result_of(call: &Call<'_>, ending: Ending) -> Result<Option<String>, CoordError> {
    let given = call.positional.get(1);
    match ending {
        Ending::Stopped => Ok(None),
        Ending::Failed if given.is_none_or(String::is_empty) => Err(CoordError::refuse(
            "say why it failed: coord agent fail <A-id|name> \"<reason>\".",
        )),
        Ending::Failed => Ok(Some(require_str("agentresult", word(&call.positional, 1))?)),
        Ending::Done => Ok(optional_text(
            "agentresult",
            &given.map_or(Value::Null, |g| Value::from(g.as_str())),
        )?),
    }
}

/// `coord agent done`, `fail` and `stop`.
///
/// # Errors
/// A missing reason, no such running agent, an agent of another thread.
pub fn run(call: &mut Call<'_>, ending: Ending) -> Outcome {
    let result = result_of(call, ending)?;
    write(call, |board, who, run| {
        let (id, agent) = own(board, who, run, (ending.verb(), Want::Live))?;
        set(agent, "status", ending.status());
        set(agent, "finishedAt", now_iso());
        if let Some(text) = &result {
            set(agent, "result", text.as_str());
        }
        let (name, thread) = (display(&agent["name"]), display(&agent["thread"]));
        stamp(board, "agents", &id, &who.actor)?;
        let running = live_agent_ids(board, Some(&thread)).len();
        let shown = result
            .as_ref()
            .map_or_else(String::new, |r| format!(" ({r})"));
        run.say(&format!(
            "{id}: {name} {}{shown}; {running} running on {thread}",
            ending.status()
        ));
        Ok(())
    })
}
