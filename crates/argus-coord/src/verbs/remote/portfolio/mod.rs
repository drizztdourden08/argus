//! `coord portfolio`: Argus, the overview of every project, one level above every project board
//! (`verbs/portfolio.mjs`). Its store is `<coord home>/_portfolio/board.json`, board-shaped so the
//! lock is shared. The ids keep the old word; only the displayed names changed.

mod registry;
mod send;
mod show;
mod tick;

use argus_board::Board;
use std::path::Path;

use argus_board::portfolio::{project_keys, shape, summarize};
use argus_core::ids::PORTFOLIO;
use serde_json::{Value, json};

use super::support::scope::{Scope, locked};
use crate::cli::exit::{CoordError, Outcome};
use crate::cli::route::Call;
use crate::verbs::remote::support::link::take_sub;

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

/// The cards of every project the registry of `p` and the boards on disk know.
fn cards_of(home: &Path, p: &Board) -> Vec<Value> {
    let registry = json!({ "projects": p["projects"] });
    project_keys(home, &registry)
        .iter()
        .map(|key| summarize(home, key, &registry, None))
        .collect()
}

/// `coord portfolio <init|add|remove|send|tick|show>`.
///
/// # Errors
/// A refusal.
pub fn handle(call: &mut Call<'_>) -> Outcome {
    let sub = take_sub(call);
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
