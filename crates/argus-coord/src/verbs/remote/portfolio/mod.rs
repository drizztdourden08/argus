//! `coord portfolio`: Argus, the overview of every project, one level above every project board
//! (`verbs/portfolio.mjs`). Its store is `<coord home>/_portfolio/board.json`, board-shaped so the
//! lock is shared. The ids keep the old word; only the displayed names changed.

mod registry;
mod send;
mod show;
mod tick;

use argus_board::Board;
use argus_board::portfolio::shape;
use argus_core::ids::PORTFOLIO;

use super::support::scope::{Scope, locked};
use crate::cli::exit::{CoordError, Outcome};
use crate::cli::route::Call;

const SUBS: &str = "init, add, remove, send, tick, show, page, url, pushed";

/// The portfolio board locked, given its shape, changed by `f` and saved.
fn with_portfolio<T>(
    call: &mut Call<'_>,
    create: bool,
    f: impl FnOnce(&mut Board, &mut Scope<'_>) -> Result<T, CoordError>,
) -> Result<T, CoordError> {
    locked(call, PORTFOLIO, create, |board, scope| {
        shape(board);
        f(board, scope)
    })
}

/// `coord portfolio <init|add|remove|send|tick|show>`.
///
/// # Errors
/// A refusal.
pub fn handle(call: &mut Call<'_>) -> Outcome {
    let sub = (!call.positional.is_empty()).then(|| call.positional.remove(0));
    match sub.as_deref() {
        Some("init") => registry::init(call),
        Some("add") => registry::add(call),
        Some("remove") => registry::remove(call),
        Some("send") => send::send(call),
        Some("tick") => tick::tick(call),
        Some("show") => show::show(call),
        Some("page" | "url" | "pushed") => {
            show::retired(call);
            Ok(())
        }
        _ => Err(CoordError::refuse(format!(
            "portfolio verb must be one of {SUBS}."
        ))),
    }
}
