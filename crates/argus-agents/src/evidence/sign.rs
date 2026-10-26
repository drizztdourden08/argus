//! The signs of life of one record: its transcript, its background task, and the worktree it works in.

use super::Env;
use super::session::Session;
use super::state::{task_state, transcript_state};
use super::walk::newest_file_mtime;
use crate::board::Board;
use crate::record::{Record, claude_id_of, filled, text};
use std::path::{Path, PathBuf};

/// One sign of life: where it was seen and when it was last written.
#[derive(Debug, Clone, Copy)]
pub(crate) struct Sign {
    pub kind: &'static str,
    pub mtime_ms: i64,
}

/// The newest sign of life of a record.
#[derive(Debug, Default)]
pub(crate) struct LastSign {
    /// The newest of the signs; 0 when there is no sign at all.
    pub last_ms: i64,
    pub signs: Vec<Sign>,
    /// Claude says the turn is over (meta finished or stopped by the user, or an end turn).
    pub ended: bool,
    /// The agent's transcript or task grew within the stall time.
    pub running: bool,
}

/// How far to look.
#[derive(Debug, Default, Clone, Copy)]
pub(crate) struct Look<'a> {
    /// The worktree is walked only when the transcript and the task are quiet longer than this (the walk is the dear one).
    pub need_beyond: Option<i64>,
    /// The id to look for instead of the record's own.
    pub claude_id: Option<&'a str>,
}

/// The worktree folder a record works in, when it names one: an absolute path, or a name under the thread's checkout
/// (`<root>/.claude/worktrees/<name>`, `<root>/<name>`, or beside it). `None` when none of them is a folder.
fn worktree_dir(env: &Env, board: &Board, agent: &Record) -> Option<PathBuf> {
    let name = filled(agent, "worktree")?.trim();
    if name.is_empty() {
        return None;
    }
    let root = text(agent, "thread")
        .and_then(|id| board.thread(id))
        .and_then(|t| filled(t, "root"));
    let tries: Vec<PathBuf> = if Path::new(name).is_absolute() {
        vec![PathBuf::from(name)]
    } else {
        root.map(Path::new)
            .map(|root| {
                let beside = root.parent().map(|p| p.join(name));
                [
                    Some(root.join(".claude").join("worktrees").join(name)),
                    Some(root.join(name)),
                    beside,
                ]
                .into_iter()
                .flatten()
                .collect()
            })
            .unwrap_or_default()
    };
    tries
        .into_iter()
        .find(|t| env.fs.stat(t).is_some_and(|s| s.is_dir))
}

/// The newest sign of life of `agent` at `at`.
pub(crate) fn last_sign(
    env: &Env,
    session: &Session,
    board: &Board,
    agent: &Record,
    at: i64,
    look: Look,
) -> LastSign {
    let id = look
        .claude_id
        .map(str::to_owned)
        .or_else(|| claude_id_of(agent));
    let mut out = LastSign::default();
    let own = filled(agent, "transcriptFile").and_then(|f| transcript_state(env, Path::new(f), at));
    let found = || {
        let file = session.find_file(id.as_deref()?)?;
        transcript_state(env, &file, at)
    };
    if let Some(state) = own.or_else(found) {
        out.signs.push(Sign {
            kind: "transcript",
            mtime_ms: state.mtime_ms,
        });
        (out.ended, out.running) = (state.ended, state.alive);
    }
    let task = id
        .as_deref()
        .and_then(|id| session.task_file(id))
        .and_then(|f| task_state(env, &f, at));
    if let Some(state) = task {
        out.signs.push(Sign {
            kind: "task",
            mtime_ms: state.mtime_ms,
        });
        out.running |= state.alive;
    }
    out.last_ms = out.signs.iter().map(|s| s.mtime_ms).max().unwrap_or(0);
    if let Some(beyond) = look.need_beyond
        && !out.running
        && at - out.last_ms > beyond
        && let Some(dir) = worktree_dir(env, board, agent)
    {
        let mtime_ms = newest_file_mtime(env.fs, &dir, at - beyond);
        if mtime_ms > 0 {
            out.signs.push(Sign {
                kind: "worktree",
                mtime_ms,
            });
            out.last_ms = out.last_ms.max(mtime_ms);
        }
    }
    out
}
