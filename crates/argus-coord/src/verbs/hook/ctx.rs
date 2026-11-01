//! What every hook works with: its input, its folder, the data folders and the environment.

use std::path::{Path, PathBuf};

use argus_board::store::{read_board, with_board};
use argus_board::{Board, BoardError};
use argus_core::Paths;
use serde_json::Value;

use crate::cli::{CoordError, Env, Options};
use crate::who::find_project;

/// One hook call.
pub struct Hook<'a> {
    /// The environment of the call.
    pub env: &'a Env,
    /// `COORD_HOME` and the dashboard home, resolved against the working directory.
    pub paths: Paths,
    /// The folder the hook is about: the input's `cwd`, else the working directory.
    pub cwd: PathBuf,
    /// The JSON the hook read on stdin.
    pub input: Value,
}

impl<'a> Hook<'a> {
    /// A hook call in `cwd` (already checked to exist), with paths resolved against `base`.
    pub fn new(env: &'a Env, base: &Path, cwd: PathBuf, input: Value) -> Self {
        Hook {
            paths: env.paths(base),
            env,
            cwd,
            input,
        }
    }

    /// The input's `session_id`, when it is a text that is not empty.
    pub fn session_id(&self) -> Option<&str> {
        self.input["session_id"].as_str().filter(|s| !s.is_empty())
    }

    /// Whether the call is a sub-agent's: its payload carries a non-empty `agent_id` (the rule
    /// of `hook.mjs`). A sub-agent's call neither marks its lead's activity nor takes its
    /// inbox.
    pub fn is_sub_agent(&self) -> bool {
        self.input["agent_id"]
            .as_str()
            .is_some_and(|id| !id.is_empty())
    }

    /// The project of the hook: found from its folder and session, never from options.
    pub fn project(&self) -> Result<String, CoordError> {
        find_project(self.env, &self.cwd, &Options::default(), self.session_id())
    }

    /// The board of a project, read and normalized; `None` when it has none.
    pub fn board(&self, project: &str) -> Result<Option<Board>, CoordError> {
        Ok(read_board(&self.paths.coord_home, project)?)
    }

    /// The project of the hook and its board; `None` when the project has no board.
    pub fn project_board(&self) -> Result<Option<(String, Board)>, CoordError> {
        let project = self.project()?;
        Ok(self.board(&project)?.map(|board| (project, board)))
    }

    /// Changes the board of the hook's project under its lock, when it has one and `admit`
    /// accepts a first look at it (a hook that has nothing to do never takes the lock).
    pub fn mutate_if(
        &self,
        admit: impl FnOnce(&Board) -> bool,
        f: impl FnOnce(&mut Board) -> Result<(), BoardError>,
    ) -> Result<(), CoordError> {
        match self.project_board()? {
            Some((project, peek)) if admit(&peek) => self.mutate(&project, f),
            _ => Ok(()),
        }
    }

    /// One locked read, change and save of a project's board. The board is saved whatever `f`
    /// returns, unless it fails.
    pub fn mutate<T>(
        &self,
        project: &str,
        f: impl FnOnce(&mut Board) -> Result<T, BoardError>,
    ) -> Result<T, CoordError> {
        Ok(with_board(&self.paths.coord_home, project, false, f)?)
    }
}
