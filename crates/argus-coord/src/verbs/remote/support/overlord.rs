//! Who may change the process rules, kill a process or clear a wake fault: the Overlord, the
//! coordinator thread of project `portfolio`. The dashboard checks it again.

use argus_board::ops::Role;

use super::scope::require_board;
use crate::cli::exit::CoordError;
use crate::cli::route::Call;
use crate::who::{caller, find_project};

const MAIN: &str = "portfolio";

fn refusal(what: &str, who: &str, advice: &str) -> CoordError {
    CoordError::refuse(format!(
        "only the Overlord (the coordinator thread of project {MAIN}) can {what}; {who}. {advice}"
    ))
}

/// The actor id of the Overlord making this call.
///
/// # Errors
/// This session is not the Overlord: the refusal names what it asked for (`what`) and ends with
/// `advice`.
pub fn actor(call: &Call<'_>, what: &str, advice: &str) -> Result<String, CoordError> {
    let env = &call.inv.env;
    let own = find_project(env, &call.inv.cwd, &call.options, env.session_id()).ok();
    match own.as_deref() {
        Some(MAIN) => {}
        Some(other) => {
            let who = format!("this session belongs to project {other}");
            return Err(refusal(what, &who, advice));
        }
        None => {
            let who = "this session belongs to no project";
            return Err(refusal(what, who, advice));
        }
    }
    let board = require_board(call, MAIN)?;
    let who = caller(&board, &call.inv.cwd, env, &call.options)?;
    if who.role != Role::Coordinator {
        let text = format!("thread {} is not the coordinator", who.thread);
        return Err(refusal(what, &text, advice));
    }
    Ok(who.actor)
}
