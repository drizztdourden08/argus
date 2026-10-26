//! A board mutation of a verb: the board locked, read, changed and saved, with what the change
//! needs of the call (the caller, the streams to print on while the board is held).

use std::io::Write;
use std::path::PathBuf;

use argus_board::Board;
use argus_board::BoardError;
use argus_board::store::{read_board, with_board};

use crate::cli::exit::CoordError;
use crate::cli::route::Call;
use crate::cli::{Env, Options, out};
use crate::verbs::shared::lock::board_refusal;
use crate::who::{Caller, caller, me};

use super::link::paths_of;

/// What a change reads of the call while the board is held.
pub struct Scope<'a> {
    /// The environment.
    pub env: &'a Env,
    /// The working directory.
    pub cwd: &'a std::path::Path,
    /// The options.
    pub options: &'a Options,
    /// Standard output: a verb prints inside its mutation, before the save.
    pub out: &'a mut dyn Write,
}

impl Scope<'_> {
    /// The caller of the verb, with the real caller's heartbeat refreshed (`begin`).
    ///
    /// # Errors
    /// The caller cannot be told.
    pub fn begin(&self, board: &mut Board) -> Result<Caller, CoordError> {
        me(board, self.cwd, self.env, self.options)?;
        caller(board, self.cwd, self.env, self.options)
    }

    /// `console.log(lines.join('\n'))`.
    pub fn say(&mut self, lines: &[&str]) {
        out::say(self.out, lines);
    }
}

fn to_board(e: &CoordError) -> BoardError {
    BoardError::Refused(e.to_string())
}

/// The coord home of this call.
#[must_use]
pub fn coord_home(call: &Call<'_>) -> PathBuf {
    paths_of(call).coord_home
}

/// Runs `f` on the board of `key` under its lock: the change prints as it goes, the board is
/// saved when `f` succeeds.
///
/// # Errors
/// The board is missing (unless `create`), stays locked, or `f` refuses.
pub fn locked<T>(
    call: &mut Call<'_>,
    key: &str,
    create: bool,
    f: impl FnOnce(&mut Board, &mut Scope<'_>) -> Result<T, CoordError>,
) -> Result<T, CoordError> {
    let home = coord_home(call);
    let inv = &mut *call.inv;
    let mut scope = Scope {
        env: &inv.env,
        cwd: &inv.cwd,
        options: &call.options,
        out: &mut inv.stdout,
    };
    with_board(&home, key, create, |board| {
        f(board, &mut scope).map_err(|e| to_board(&e))
    })
    .map_err(|e| board_refusal(e, &home, key))
}

/// The board of a project, read.
///
/// # Errors
/// The project has no board, or the file cannot be read.
pub fn require_board(call: &Call<'_>, key: &str) -> Result<Board, CoordError> {
    let home = coord_home(call);
    read_board(&home, key)?.ok_or_else(|| BoardError::Missing(key.to_owned()).into())
}
