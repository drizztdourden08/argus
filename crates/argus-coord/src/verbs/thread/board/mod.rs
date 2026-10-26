//! `coord board <verb>` (`verbs/board.mjs`): create the board of a project, name it, print a
//! summary, point the Browser pane at it, and open it on the dashboard.

mod init;
mod settings;
mod shortcut;
mod show;
mod summary;

use super::frame::{Frame, with_frame};
use crate::cli::exit::{CoordError, NOT_PORTED, Outcome};
use crate::cli::route::Call;

type Verb = fn(&mut Frame<'_>) -> Outcome;

fn page(frame: &mut Frame<'_>) -> Outcome {
    let url = frame.dashboard_url(&format!("/p/{}/", frame.project));
    frame.say(&[&format!("retired: the dashboard serves this page at {url}")]);
    Ok(())
}

/// Opening needs the dashboard; its own verb starts the server when it is down. That verb
/// belongs to the server clients, so the Node verb runs until it is ported (exit 120, before
/// anything is touched).
fn open(_frame: &mut Frame<'_>) -> Outcome {
    Err(CoordError::Exit(NOT_PORTED))
}

fn path(frame: &mut Frame<'_>) -> Outcome {
    let file = frame.board_file().to_string_lossy().into_owned();
    frame.say(&[&file]);
    Ok(())
}

const VERBS: [(&str, Verb); 9] = [
    ("init", init::run),
    ("open", open),
    ("page", page),
    ("url", page),
    ("show", show::run),
    ("path", path),
    ("name", settings::name),
    ("prefixes", settings::prefixes),
    ("shortcut", shortcut::run),
];

/// The verb `board`.
///
/// # Errors
/// Refused as the verb it runs refuses.
pub fn run(call: &mut Call<'_>) -> Outcome {
    with_frame(call, |frame| {
        let found = frame
            .arg(0)
            .and_then(|sub| VERBS.iter().find(|(name, _)| name == &sub));
        let Some((_, verb)) = found else {
            let names: Vec<&str> = VERBS.iter().map(|(name, _)| *name).collect();
            return Err(CoordError::refuse(format!(
                "board verb must be one of {}.",
                names.join(", ")
            )));
        };
        frame.args = &frame.args[1..];
        verb(frame)
    })
}
