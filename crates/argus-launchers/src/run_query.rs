//! Reading runs: one run, a listing, the log of a run with its cursors, and the visible
//! definitions.

use crate::def::{RunDef, public_def};
use crate::error::RunError;
use crate::host::ProcessHost;
use crate::names::{check_key, check_rid};
use crate::record::{ListFilter, LogLine, LogRead, ReadMode, RunRow, Stream};
use crate::run_service::RunService;

const TAIL_DEFAULT: i64 = 200;
const TAIL_MAX: i64 = 5000;
const LIMIT_DEFAULT: i64 = 1000;
const LIMIT_MAX: i64 = 5000;

/// What a log read asks for: the newest `tail` lines, or those after the cursor `after`.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct LogOptions {
    /// `out`, `err` or `sys`; anything else reads every stream.
    pub stream: Option<String>,
    pub tail: Option<i64>,
    pub after: Option<i64>,
    pub limit: Option<i64>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LogPage {
    pub lines: Vec<LogLine>,
    /// The cursor for the following call.
    pub next: u64,
    pub dropped: u64,
}

fn clamp(value: i64, min: i64, max: i64) -> usize {
    usize::try_from(value.clamp(min, max)).unwrap_or_default()
}

fn stream_of(name: Option<&str>) -> Option<Stream> {
    match name {
        Some("out") => Some(Stream::Out),
        Some("err") => Some(Stream::Err),
        Some("sys") => Some(Stream::Sys),
        _ => None,
    }
}

impl<H: ProcessHost> RunService<H> {
    /// The run, or `None` for an unknown one or a malformed key or id.
    pub fn get(&self, key: &str, rid: &str) -> Option<RunRow> {
        if check_key(key).is_err() || check_rid(rid).is_err() {
            return None;
        }
        self.inner.deps.store.get(key, rid)
    }

    /// The run, or `unknown_run`.
    pub(crate) fn known_run(&self, key: &str, rid: &str) -> Result<RunRow, RunError> {
        self.get(key, rid)
            .ok_or_else(|| RunError::unknown_run(format!("no run {rid} in {key}")))
    }

    /// Runs of a project, newest first.
    pub fn list(&self, key: &str, filter: &ListFilter) -> Vec<RunRow> {
        self.inner.deps.store.list(Some(key), filter)
    }

    /// Runs of every project, newest first.
    pub fn list_all(&self, filter: &ListFilter) -> Vec<RunRow> {
        self.inner.deps.store.list(None, filter)
    }

    /// The lines of a run.
    pub fn logs(&self, key: &str, rid: &str, options: &LogOptions) -> Result<LogPage, RunError> {
        let run = self.known_run(key, rid)?;
        let stream = stream_of(options.stream.as_deref());
        if options.after.is_none() && options.tail.is_some() {
            let n = clamp(options.tail.unwrap_or(TAIL_DEFAULT), 1, TAIL_MAX);
            let lines = self.read_lines(key, rid, stream, ReadMode::Tail(n));
            let next = lines.last().map_or(0, |l| l.seq).max(run.log_seq);
            return Ok(LogPage {
                lines,
                next,
                dropped: run.log_dropped,
            });
        }
        let after = u64::try_from(options.after.unwrap_or(0).max(0)).unwrap_or(0);
        let limit = clamp(options.limit.unwrap_or(LIMIT_DEFAULT), 1, LIMIT_MAX);
        let lines = self.read_lines(key, rid, stream, ReadMode::After { after, limit });
        let next = lines.last().map_or(after, |l| l.seq);
        Ok(LogPage {
            lines,
            next,
            dropped: run.log_dropped,
        })
    }

    fn read_lines(
        &self,
        key: &str,
        rid: &str,
        stream: Option<Stream>,
        mode: ReadMode,
    ) -> Vec<LogLine> {
        self.inner
            .deps
            .store
            .read(key, rid, &LogRead { stream, mode })
    }

    /// The definitions of a project as a page may see them (no env values).
    pub fn defs(&self, key: &str) -> Result<Vec<RunDef>, RunError> {
        check_key(key)?;
        Ok(self
            .inner
            .deps
            .defs
            .defs(key)
            .iter()
            .map(public_def)
            .collect())
    }
}
