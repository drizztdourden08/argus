//! `board init`: create the board of a project, or say where it already is.

use argus_board::json::set;
use argus_board::store::read_board;
use argus_core::js;

use crate::cli::exit::Outcome;
use crate::verbs::thread::frame::Frame;

/// The verb `board init [--name <Name>] [--force]`.
///
/// # Errors
/// Refused when the board cannot be read or written.
pub fn run(frame: &mut Frame<'_>) -> Outcome {
    let existing = read_board(frame.home(), frame.project)?;
    let file = frame.board_file().to_string_lossy().into_owned();
    let page = frame.dashboard_url(&format!("/p/{}/", frame.project));
    if existing.is_some() && !frame.options.flag("force") {
        frame.say(&[&format!(
            "board for {} already exists at {file} (page: {page}).",
            frame.project
        )]);
        return Ok(());
    }
    let name = frame.options.text("name").map(|n| js::trim(n).to_owned());
    frame.mutate(true, |board, _| {
        if let Some(name) = name {
            set(board, "name", name);
        }
        Ok(())
    })?;
    frame.say(&[
        &format!("board created for {}: {file}", frame.project),
        "",
        "Next:",
        "  1. coord thread register lead --role lead --session \"<this session title>\"",
        &format!("  2. coord board open   (the page: {page})"),
    ]);
    Ok(())
}
