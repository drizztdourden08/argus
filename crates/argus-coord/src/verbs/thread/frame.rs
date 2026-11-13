//! The state a verb of this group runs with: the streams, the options, the data folders and the
//! project, split into borrowed parts so a mutation can print while it holds the board.

use std::io::Write;
use std::path::{Path, PathBuf};

use argus_board::Board;
use argus_board::error::BoardError;
use argus_board::store::{board_path, read_board, with_board};
use argus_core::Paths;
use serde_json::Value;

use crate::cli::exit::{CoordError, Outcome};
use crate::cli::{Call, Env, Options, out};
use crate::client::board_port;
use crate::verbs::shared::lock::board_refusal;
use crate::who::{Caller, caller, me, refuse_while_migrating};

pub use crate::verbs::shared::caller::{is_coordinator, who_of};

/// One call of a verb of this group.
pub struct Frame<'a> {
    /// Standard output.
    pub out: &'a mut dyn Write,
    /// Standard error.
    pub err: &'a mut dyn Write,
    /// The environment.
    pub env: &'a Env,
    /// The working directory.
    pub cwd: &'a Path,
    /// The data folders.
    pub paths: &'a Paths,
    /// The project key.
    pub project: &'a str,
    /// The options.
    pub options: &'a Options,
    /// The words of the verb still to read.
    pub args: &'a [String],
    /// The time the call started, in epoch milliseconds.
    pub now_ms: i64,
}

/// Runs `f` with the frame of a call. The router resolved the project for these verbs.
///
/// # Errors
/// Whatever `f` returns.
pub fn with_frame(call: &mut Call<'_>, f: impl FnOnce(&mut Frame<'_>) -> Outcome) -> Outcome {
    let project = call
        .project
        .clone()
        .ok_or_else(|| CoordError::refuse("no project for this call."))?;
    let inv = &mut *call.inv;
    let paths = inv.env.paths(&inv.cwd);
    let now_ms = inv.clock.now_ms();
    let mut frame = Frame {
        out: &mut inv.stdout,
        err: &mut inv.stderr,
        env: &inv.env,
        cwd: &inv.cwd,
        paths: &paths,
        project: &project,
        options: &call.options,
        args: &call.positional,
        now_ms,
    };
    f(&mut frame)
}

/// `withBoard` on any project of the home: the CLI flavour of the lock, whose refusal names the
/// lock file.
///
/// # Errors
/// Refused when the lock stays taken, when there is no board (and `create` is off), when `f`
/// refuses or the board cannot be saved.
pub fn locked<T>(
    home: &Path,
    project: &str,
    create: bool,
    f: impl FnOnce(&mut Board) -> Result<T, CoordError>,
) -> Result<T, CoordError> {
    refuse_while_migrating(home)?;
    with_board(home, project, create, |board| {
        f(board).map_err(|e| BoardError::Refused(e.to_string()))
    })
    .map_err(|e| board_refusal(e, home, project))
}

impl<'a> Frame<'a> {
    /// `console.log(lines.join('\n'))`.
    pub fn say(&mut self, lines: &[&str]) {
        out::say(&mut *self.out, lines);
    }

    /// `console.log(JSON.stringify(value, null, 2))`.
    pub fn json(&mut self, value: &Value) {
        out::as_json(&mut *self.out, value);
    }

    /// The folder of the boards.
    #[must_use]
    pub fn home(&self) -> &'a Path {
        let paths: &'a Paths = self.paths;
        &paths.coord_home
    }

    /// The board file of this project.
    #[must_use]
    pub fn board_file(&self) -> PathBuf {
        board_path(self.home(), self.project)
    }

    /// `options[name]` as a value: `null` when it is not given.
    #[must_use]
    pub fn value(&self, name: &str) -> Value {
        self.options.json(name).unwrap_or(Value::Null)
    }

    /// The word at a position.
    #[must_use]
    pub fn arg(&self, at: usize) -> Option<&'a str> {
        let args: &'a [String] = self.args;
        args.get(at).map(String::as_str)
    }

    /// `requireBoard`: the board of the project, read without the lock.
    ///
    /// # Errors
    /// Refused when the project has no board or it cannot be read.
    pub fn read_board(&self) -> Result<Board, CoordError> {
        read_board(self.home(), self.project)?
            .ok_or_else(|| BoardError::Missing(self.project.to_owned()).into())
    }

    /// Runs `f` under the board lock: load, mutate, save (`withBoard`). A refusal inside `f`
    /// saves nothing.
    ///
    /// # Errors
    /// Refused when the lock stays taken, when there is no board (and `create` is off), when
    /// `f` refuses or the board cannot be saved.
    pub fn mutate<T>(
        &mut self,
        create: bool,
        f: impl FnOnce(&mut Board, &mut Frame<'a>) -> Result<T, CoordError>,
    ) -> Result<T, CoordError> {
        let (home, project) = (self.home(), self.project);
        locked(home, project, create, |board| f(board, self))
    }

    /// The caller of the verb, without a heartbeat.
    ///
    /// # Errors
    /// As [`caller`].
    pub fn caller(&self, board: &Board) -> Result<Caller, CoordError> {
        caller(board, self.cwd, self.env, self.options)
    }

    /// The caller with its heartbeat refreshed: every verb a thread runs is a sign of life.
    ///
    /// # Errors
    /// As [`caller`].
    pub fn identify(&self, board: &mut Board) -> Result<Caller, CoordError> {
        let who = self.caller(board)?;
        me(board, self.cwd, self.env, self.options)?;
        Ok(who)
    }

    /// A relative time as the CLI prints it.
    #[must_use]
    pub fn ago(&self, time: &Value) -> String {
        out::ago(time.as_str(), self.now_ms)
    }

    /// Where the dashboard serves a page.
    #[must_use]
    pub fn dashboard_url(&self, path: &str) -> String {
        format!(
            "http://127.0.0.1:{}{path}",
            board_port(self.env, self.paths)
        )
    }
}
