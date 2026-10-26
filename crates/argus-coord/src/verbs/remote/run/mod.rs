//! `coord run`: what a lead lets the owner and agents launch from the dashboard, and the commands
//! to start, watch and stop them (`verbs/run.mjs`).
//!
//! `define`, `undefine` and `defs` work on the board file alone. The others go through the
//! dashboard server and start it when it is down; a dashboard that will not come up is exit 3
//! with the plain message.

mod define;
mod follow;
mod names;
mod show;
mod start;

use argus_board::BoardError;

use super::support::link::connect;
use crate::cli::exit::{CoordError, Outcome};
use crate::cli::route::Call;
use crate::client::{Client, ClientError};

/// Why a server verb of `run` ended: a refusal of its own, or the server's answer.
pub(super) enum Fail {
    Coord(CoordError),
    Client(ClientError),
}

impl From<CoordError> for Fail {
    fn from(e: CoordError) -> Self {
        Fail::Coord(e)
    }
}

impl From<ClientError> for Fail {
    fn from(e: ClientError) -> Self {
        Fail::Client(e)
    }
}

impl From<BoardError> for Fail {
    fn from(e: BoardError) -> Self {
        Fail::Coord(e.into())
    }
}

/// The result of a server verb of `run`.
pub(super) type Res<T> = Result<T, Fail>;

type ServerVerb = fn(&mut Call<'_>, &Client) -> Res<()>;

const SUBS: &str = "define, undefine, defs, start, stop, status, list, logs";

fn online(call: &mut Call<'_>, verb: ServerVerb) -> Outcome {
    let done = connect(call, true)
        .map_err(Fail::Client)
        .and_then(|client| verb(call, &client));
    match done {
        Ok(()) => Ok(()),
        Err(Fail::Coord(e)) => Err(e),
        Err(Fail::Client(e @ ClientError::Down(_))) => {
            call.inv.warn(&format!("coord: {e}"));
            Err(CoordError::Exit(3))
        }
        Err(Fail::Client(e)) => Err(e.into()),
    }
}

/// `coord run <define|undefine|defs|start|stop|status|list|logs>`.
///
/// # Errors
/// A refusal, the dashboard not running (exit 3) or the run's own exit code.
pub fn handle(call: &mut Call<'_>) -> Outcome {
    let sub = (!call.positional.is_empty()).then(|| call.positional.remove(0));
    match sub.as_deref() {
        Some("define") => define::define(call),
        Some("undefine") => define::undefine(call),
        Some("defs") => define::defs(call),
        Some("start") => online(call, start::start),
        Some("stop") => online(call, start::stop),
        Some("status") => online(call, show::status),
        Some("list") => online(call, show::list),
        Some("logs") => online(call, follow::logs),
        _ => Err(CoordError::refuse(format!(
            "run verb must be one of {SUBS}."
        ))),
    }
}
