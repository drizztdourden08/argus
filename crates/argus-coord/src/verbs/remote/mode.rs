//! `coord remote [status|local|remote]`: the GitHub switch of `<COORD_HOME>/remote.json`.
//! `local` freezes every call to GitHub, `remote` is the owner's go; only the owner sets it.

use std::path::PathBuf;

use argus_core::remote::{LOCAL, REMOTE, Switch, read_remote, remote_file, write_remote};

use crate::cli::exit::{CoordError, Outcome};
use crate::cli::route::Call;

const USAGE: &str = "usage: coord remote [status|local|remote].";

fn describe(state: &Switch) -> String {
    let since = match state.since.as_deref().filter(|s| !s.is_empty()) {
        Some(at) => match state.by.as_deref().filter(|b| !b.is_empty()) {
            Some(by) => format!(" since {at} (set by {by})"),
            None => format!(" since {at}"),
        },
        None => " (no file: the default while the freeze lasts)".to_owned(),
    };
    let meaning = if state.mode == LOCAL {
        "GitHub is not used: no gh, no API, no push, no publish, no GitHub registry."
    } else {
        "GitHub is used again."
    };
    format!("remote mode: {}{since}. {meaning}", state.mode)
}

fn home_of(call: &Call<'_>) -> PathBuf {
    call.inv.env.paths(&call.inv.cwd).coord_home
}

/// `--as` when it is a non-empty text, else the user name of the environment.
fn setter_of(call: &Call<'_>) -> Option<String> {
    call.options
        .text("as")
        .filter(|a| !a.is_empty())
        .or_else(|| call.inv.env.get("USERNAME"))
        .map(str::to_owned)
}

fn set(call: &mut Call<'_>, mode: &str) -> Outcome {
    let home = home_of(call);
    let by = setter_of(call);
    let state =
        write_remote(&home, mode, by.as_deref()).map_err(|e| CoordError::refuse(e.to_string()))?;
    let file = remote_file(&home);
    call.inv
        .say(&[&format!("{} ({})", describe(&state), file.display())]);
    Ok(())
}

fn status(call: &mut Call<'_>) -> Outcome {
    let text = describe(&read_remote(&home_of(call)));
    call.inv.say(&[&text]);
    Ok(())
}

/// `coord remote [status|local|remote]`.
///
/// # Errors
/// A refusal for an unknown word, or a write that fails.
pub fn handle(call: &mut Call<'_>) -> Outcome {
    let word = call.positional.first().cloned();
    match word.as_deref() {
        None | Some("status") => status(call),
        Some(LOCAL) => set(call, LOCAL),
        Some(REMOTE) => set(call, REMOTE),
        Some(_) => Err(CoordError::refuse(USAGE)),
    }
}
