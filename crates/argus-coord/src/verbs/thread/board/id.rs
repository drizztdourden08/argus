//! `board id`: the id of this board, its display name and its old keys.

use argus_board::names::display_name;
use argus_core::boards;

use crate::cli::exit::Outcome;
use crate::verbs::thread::frame::Frame;

/// The verb `board id`: prints `<id> <name> <aliases comma separated>`. A board with no table
/// entry has its folder as id, its board name as name and no aliases.
///
/// # Errors
/// Never; a board with no file is shown by its id.
pub fn run(frame: &mut Frame<'_>) -> Outcome {
    let (home, id) = (frame.home(), frame.project);
    let board_name = frame
        .read_board()
        .ok()
        .and_then(|b| b["name"].as_str().map(str::to_owned));
    let name = display_name(home, id, board_name.as_deref(), None);
    let aliases = boards::load(home)
        .ok()
        .and_then(|t| t.boards.get(id).map(|b| b.aliases.join(",")))
        .unwrap_or_default();
    frame.say(&[format!("{id} {name} {aliases}").trim_end()]);
    Ok(())
}
