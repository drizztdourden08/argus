//! What every work verb runs with: the streams and options of the call, the arguments after the
//! sub-verb, the board folder, and the two ways to reach the board (a locked write, a plain
//! read). A verb prints inside its write, before the save, as the Node verbs do.

use std::path::{Path, PathBuf};

use argus_board::store::{board_dir, read_board, with_board};
use argus_board::{Board, BoardError};
use serde_json::Value;

use crate::cli::{Call, CoordError, Invocation, Options, Outcome};
use crate::who::{Caller, caller, me};

/// A board step that can refuse.
pub type Res<T> = Result<T, BoardError>;

/// One sub-verb of a group.
pub type Handler = fn(&mut Run<'_>) -> Outcome;

/// A refusal with its message.
pub fn bad(text: impl Into<String>) -> BoardError {
    BoardError::Refused(text.into())
}

fn lift(e: CoordError) -> BoardError {
    match e {
        CoordError::Refused(message) => BoardError::Refused(message),
        CoordError::Exit(code) => BoardError::Refused(format!("exit {code}")),
    }
}

/// The state of one work verb.
pub struct Run<'a> {
    /// The streams, environment, working directory and clock.
    pub inv: &'a mut Invocation,
    /// The options of the call.
    pub options: &'a Options,
    /// The words after the sub-verb.
    pub args: Vec<String>,
    /// The project key.
    pub project: String,
    /// The folder of the boards.
    pub home: PathBuf,
}

impl<'a> Run<'a> {
    /// The run of a call: the words as they were typed (the sub-verb included).
    ///
    /// # Errors
    /// Refused when the router gave no project.
    pub fn of(call: &'a mut Call<'_>) -> Result<Self, CoordError> {
        let project = call
            .project
            .clone()
            .ok_or_else(|| CoordError::refuse("no project for this verb."))?;
        Ok(Run {
            home: call.inv.env.paths(&call.inv.cwd).coord_home,
            args: call.positional.clone(),
            options: &call.options,
            inv: &mut *call.inv,
            project,
        })
    }

    /// Takes the first word off the arguments.
    pub fn shift(&mut self) -> Option<String> {
        (!self.args.is_empty()).then(|| self.args.remove(0))
    }

    /// The word at a position.
    #[must_use]
    pub fn arg(&self, i: usize) -> Option<&str> {
        self.args.get(i).map(String::as_str)
    }

    /// The word at a position as JSON: a text, or `null` when missing.
    #[must_use]
    pub fn arg_json(&self, i: usize) -> Value {
        self.arg(i).map_or(Value::Null, Value::from)
    }

    /// An option as JSON: `true`, a text, a list, or `null` when missing.
    #[must_use]
    pub fn opt(&self, name: &str) -> Value {
        self.options.json(name).unwrap_or(Value::Null)
    }

    /// An option that is a text.
    #[must_use]
    pub fn text_opt(&self, name: &str) -> Option<&str> {
        self.options.text(name)
    }

    /// `console.log(lines.join('\n'))`.
    pub fn say(&mut self, lines: &[&str]) {
        self.inv.say(lines);
    }

    /// The time, for the relative ages of a listing.
    #[must_use]
    pub fn now_ms(&self) -> i64 {
        self.inv.clock.now_ms()
    }

    /// The caller of a verb that writes: its heartbeat refreshed, then who it acts as.
    ///
    /// # Errors
    /// Refused as the caller lookup refuses.
    pub fn begin(&self, board: &mut Board) -> Res<Caller> {
        let (cwd, env) = (&self.inv.cwd, &self.inv.env);
        me(board, cwd, env, self.options).map_err(lift)?;
        caller(board, cwd, env, self.options).map_err(lift)
    }

    /// The caller of a verb that only reads.
    ///
    /// # Errors
    /// Refused as the caller lookup refuses.
    pub fn who(&self, board: &Board) -> Res<Caller> {
        caller(board, &self.inv.cwd, &self.inv.env, self.options).map_err(lift)
    }

    /// Runs a change under the board lock: read, change, save.
    ///
    /// # Errors
    /// Whatever the change refuses with, a lock that stays taken, a missing board.
    pub fn write<T>(
        &mut self,
        change: impl FnOnce(&mut Board, &mut Run<'_>) -> Res<T>,
    ) -> Result<T, CoordError> {
        let (home, key) = (self.home.clone(), self.project.clone());
        with_board(&home, &key, false, |board| change(board, self))
            .map_err(|e| refusal(e, &home, &key))
    }

    /// Reads the board without the lock.
    ///
    /// # Errors
    /// A missing board, or whatever the reader refuses with.
    pub fn read<T>(
        &mut self,
        show: impl FnOnce(&Board, &mut Run<'_>) -> Res<T>,
    ) -> Result<T, CoordError> {
        let Some(board) = read_board(&self.home, &self.project)? else {
            return Err(BoardError::Missing(self.project.clone()).into());
        };
        Ok(show(&board, self)?)
    }
}

fn refusal(e: BoardError, home: &Path, key: &str) -> CoordError {
    match e {
        BoardError::Locked(_) => CoordError::refuse(format!(
            "board is locked by another coord call ({}); retry.",
            board_dir(home, key).join("board.lock").display()
        )),
        other => other.into(),
    }
}
