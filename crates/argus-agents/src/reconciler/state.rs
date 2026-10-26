//! Reading one transcript or one task output: is it growing, did its turn end.

use super::Env;
use super::session::{meta_path, read_object};
use crate::record::Record;
use crate::transcript::{Role, last_turn};
use serde_json::Value;
use std::path::{Path, PathBuf};

const TASK_TAIL_BYTES: u64 = 512;

/// What a transcript or a task output shows at one moment.
#[derive(Debug)]
pub(super) struct Observed {
    pub file: PathBuf,
    pub mtime_ms: i64,
    /// Size of the transcript; 0 for a task output, whose last turn is never read.
    pub size: u64,
    pub ended: bool,
    pub stopped: bool,
    pub ended_at: i64,
    pub quiet_ms: i64,
    pub alive: bool,
}

fn flag(meta: Option<&Record>, key: &str) -> bool {
    meta.and_then(|m| m.get(key)) == Some(&Value::Bool(true))
}

fn turn_ended(env: &Env, file: &Path, size: u64) -> bool {
    let Some(last) = last_turn(env.fs, file, size) else {
        return false;
    };
    let stop = last.stop.as_deref().unwrap_or("");
    last.role == Role::Assistant && matches!(stop, "end_turn" | "stop_sequence")
}

/// The state of a sub-agent transcript that exists, else `None`.
pub(super) fn transcript_state(env: &Env, file: &Path, at: i64) -> Option<Observed> {
    let stat = env.fs.stat(file)?;
    let meta_file = meta_path(file);
    let meta_stat = env.fs.stat(&meta_file);
    let meta = meta_stat.and_then(|_| read_object(env.fs, &meta_file));
    let stopped = flag(meta.as_ref(), "stoppedByUser");
    let mut ended = stopped;
    let mut ended_at = stat.mtime_ms;
    let slack = env.opts.meta_slack_ms;
    let finished_in_step = meta_stat
        .filter(|m| flag(meta.as_ref(), "finished") && stat.mtime_ms <= m.mtime_ms + slack);
    if let Some(m) = finished_in_step {
        ended = true;
        ended_at = stat.mtime_ms.max(m.mtime_ms);
    } else if !ended {
        let unflagged = meta.as_ref().is_none_or(|m| !m.contains_key("finished"));
        let settled = at - stat.mtime_ms >= env.opts.settle_ms;
        ended = unflagged && settled && turn_ended(env, file, stat.size);
    }
    let quiet_ms = at - stat.mtime_ms;
    Some(Observed {
        file: file.to_path_buf(),
        mtime_ms: stat.mtime_ms,
        size: stat.size,
        ended,
        stopped,
        ended_at,
        quiet_ms,
        alive: !ended && quiet_ms <= env.opts.stall_ms,
    })
}

fn is_exit_code(text: &str) -> bool {
    let digits = text.strip_prefix('-').unwrap_or(text);
    !digits.is_empty() && digits.chars().all(|c| c.is_ascii_digit())
}

fn exit_marker(tail: &str) -> bool {
    let Some(body) = tail.trim_end().strip_suffix(']') else {
        return false;
    };
    let Some((_, inner)) = body.rsplit_once('[') else {
        return false;
    };
    inner == "killed"
        || inner
            .strip_prefix("exited with code ")
            .is_some_and(is_exit_code)
}

/// The state of a background task output: alive while it grows, a shell's exit marker ends it.
pub(super) fn task_state(env: &Env, file: &Path, at: i64) -> Option<Observed> {
    let stat = env.fs.stat(file)?;
    let n = stat.size.min(TASK_TAIL_BYTES);
    let tail = env
        .fs
        .read_at(file, stat.size - n, usize::try_from(n).ok()?)
        .map(|bytes| String::from_utf8_lossy(&bytes).into_owned());
    let ended = tail.is_ok_and(|t| exit_marker(&t));
    let quiet_ms = at - stat.mtime_ms;
    Some(Observed {
        file: file.to_path_buf(),
        mtime_ms: stat.mtime_ms,
        size: 0,
        ended,
        stopped: false,
        ended_at: stat.mtime_ms,
        quiet_ms,
        alive: !ended && quiet_ms <= env.opts.stall_ms,
    })
}
