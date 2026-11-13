//! Who may change the process rules, kill a process or clear a wake fault: the Overlord, the
//! coordinator thread of the board that holds the Overlord role (found by its role in
//! `boards.json`, an id or an alias; `portfolio` only in a home without the table). The dashboard
//! checks it again.

use argus_board::roles::{id_of, is_overlord};
use argus_core::boards::Role;

use super::scope::{coord_home, require_board};
use crate::cli::exit::CoordError;
use crate::cli::route::Call;
use crate::verbs::shared::caller::is_coordinator;
use crate::who::{caller, find_project};

/// The Overlord's board when no board holds the role yet: its old key.
const LEGACY: &str = "portfolio";

fn refusal(main: &str, what: &str, who: &str, advice: &str) -> CoordError {
    CoordError::refuse(format!(
        "only the Overlord (the coordinator thread of project {main}) can {what}; {who}. {advice}"
    ))
}

/// The actor id of the Overlord making this call.
///
/// # Errors
/// This session is not the Overlord: the refusal names what it asked for (`what`) and ends with
/// `advice`.
pub fn actor(call: &Call<'_>, what: &str, advice: &str) -> Result<String, CoordError> {
    let env = &call.inv.env;
    let home = coord_home(call);
    let main = id_of(&home, Role::Overlord).unwrap_or_else(|| LEGACY.to_owned());
    let own = find_project(env, &call.inv.cwd, &call.options, env.session_id()).ok();
    match own.as_deref() {
        Some(project) if project == main || is_overlord(&home, project) => {}
        Some(other) => {
            let who = format!("this session belongs to project {other}");
            return Err(refusal(&main, what, &who, advice));
        }
        None => {
            let who = "this session belongs to no project";
            return Err(refusal(&main, what, who, advice));
        }
    }
    let board = require_board(call, &main)?;
    let who = caller(&board, &call.inv.cwd, env, &call.options)?;
    if !is_coordinator(&who) {
        let text = format!("thread {} is not the coordinator", who.thread);
        return Err(refusal(&main, what, &text, advice));
    }
    Ok(who.actor)
}
