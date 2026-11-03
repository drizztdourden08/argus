//! The Play definition of a build (`board.runDefs["launcher-<name>"]`) while the build is closed:
//! dropped from the run definitions, kept in `board.closedPlays` so a reopen puts it back as it
//! was.

use crate::Board;
use crate::json::{map_mut, section};
use crate::launchers::launcher_def_name;
use crate::store::touch;

const STASH: &str = "closedPlays";

/// Takes the Play definition of `name` off the run definitions and keeps it. A definition made
/// by hand under that run name (one that does not name this launcher) is never touched.
pub(super) fn drop_play(board: &mut Board, name: &str) {
    let def_name = launcher_def_name(name);
    let owned = section(board, "runDefs")
        .get(&def_name)
        .is_some_and(|def| def["launcher"].as_str() == Some(name));
    if !owned {
        return;
    }
    if let Some(def) = map_mut(board, "runDefs").shift_remove(&def_name) {
        map_mut(board, STASH).insert(name.to_owned(), def);
    }
    touch(board, "runDefs", &def_name);
}

/// Puts the kept Play definition of `name` back, unless a definition has taken its place.
pub(super) fn restore_play(board: &mut Board, name: &str) {
    let Some(def) = map_mut(board, STASH).shift_remove(name) else {
        return;
    };
    let def_name = launcher_def_name(name);
    if section(board, "runDefs").contains_key(&def_name) {
        return;
    }
    map_mut(board, "runDefs").insert(def_name.clone(), def);
    touch(board, "runDefs", &def_name);
}
