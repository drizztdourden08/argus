//! How a verb reaches the board: [`write`] takes the lock, reads the board, hands the caller and
//! the streams to the verb and saves what it changed; [`read`] reads without a lock. The verb
//! prints inside its closure, so its lines come before a failed save, as in the Node CLI.

use std::io::Write;
use std::path::{Path, PathBuf};

use argus_board::Board;
use argus_board::BoardError;
use argus_board::store::{read_board, with_board};

use crate::cli::exit::CoordError;
use crate::cli::out::say;
use crate::cli::{Call, Env, Invocation, Options};
use crate::verbs::shared::lock::board_refusal;
use crate::who::{Caller, caller, me, refuse_while_migrating};

/// What a verb works with besides the board.
pub struct Run<'a> {
    /// The words after the sub-verb.
    pub positional: &'a [String],
    /// The options.
    pub options: &'a Options,
    /// The working directory.
    pub cwd: &'a Path,
    /// The environment.
    pub env: &'a Env,
    /// Standard output.
    pub out: &'a mut dyn Write,
    /// The time, in epoch milliseconds, `Date.now()` of the verb.
    pub now_ms: i64,
}

impl Run<'_> {
    /// `say(text)`: the text and a newline on stdout.
    pub fn say(&mut self, text: &str) {
        say(self.out, &[text]);
    }
}

fn board_error(e: &CoordError) -> BoardError {
    BoardError::Refused(e.to_string())
}

/// `begin`: the caller of a verb, with the real caller's heartbeat refreshed.
fn begin(board: &mut Board, run: &Run<'_>) -> Result<Caller, CoordError> {
    me(board, run.cwd, run.env, run.options)?;
    caller(board, run.cwd, run.env, run.options)
}

/// The coord home, the project key and the working set of a call.
fn open<'a>(call: &'a mut Call<'_>) -> (PathBuf, &'a str, Run<'a>) {
    let Call {
        inv,
        positional,
        options,
        project,
    } = call;
    let Invocation {
        env,
        cwd,
        stdout,
        clock,
        ..
    } = &mut **inv;
    let home = env.paths(cwd).coord_home;
    let run = Run {
        positional,
        options,
        cwd,
        env,
        out: &mut **stdout,
        now_ms: clock.now_ms(),
    };
    (home, project.as_deref().unwrap_or_default(), run)
}

/// Runs a verb that changes the board: under the board lock, the caller known, the board saved
/// when the verb succeeds.
///
/// # Errors
/// What the verb refuses with, a board without a file, a lock that stays taken.
pub fn write<T>(
    call: &mut Call<'_>,
    verb: impl FnOnce(&mut Board, &Caller, &mut Run<'_>) -> Result<T, CoordError>,
) -> Result<T, CoordError> {
    let (home, key, mut run) = open(call);
    refuse_while_migrating(&home)?;
    with_board(&home, key, false, |board| {
        let who = begin(board, &run).map_err(|e| board_error(&e))?;
        verb(board, &who, &mut run).map_err(|e| board_error(&e))
    })
    .map_err(|e| board_refusal(e, &home, key))
}

/// Runs a verb that only reads the board (`requireBoard`): no lock, nothing saved.
///
/// # Errors
/// What the verb refuses with, a board without a file.
pub fn read<T>(
    call: &mut Call<'_>,
    verb: impl FnOnce(&Board, &mut Run<'_>) -> Result<T, CoordError>,
) -> Result<T, CoordError> {
    let (home, key, mut run) = open(call);
    let board = read_board(&home, key)?.ok_or_else(|| BoardError::Missing(key.to_owned()))?;
    verb(&board, &mut run)
}
