//! The size window of a run's log: the first `log_head_bytes` are kept for good (`head`),
//! then as much of the newest output as fits in `log_cap_bytes`; older middle lines are
//! dropped and counted. The window plans a batch before the store commits it and only
//! accepts the plan afterwards, so a failed commit leaves it consistent with the store.

use std::collections::VecDeque;

use crate::ansi::plain_text;
use crate::record::{LogCommit, LogLine, RunLimits, RunRow, StoredLine, Stream};

const LAST_LINE_MAX: usize = 200;

/// A line waiting for the next flush.
#[derive(Clone, Debug)]
pub struct PendingLine {
    pub stream: Stream,
    pub at: String,
    pub text: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct Counters {
    seq: u64,
    kept: u64,
    dropped: u64,
    head_bytes: u64,
    last_line: Option<String>,
}

/// The stored size of a line: its bytes and the newline.
pub(crate) fn size_of(text: &str) -> u64 {
    u64::try_from(text.len())
        .unwrap_or(u64::MAX)
        .saturating_add(1)
}

/// What the status line shows of a program's own line: plain text, at most 200 characters.
fn last_line_of(text: &str) -> Option<String> {
    let plain = plain_text(text);
    let plain = plain.trim();
    if plain.is_empty() {
        return None;
    }
    if plain.chars().count() > LAST_LINE_MAX {
        let cut: String = plain.chars().take(LAST_LINE_MAX - 1).collect();
        return Some(format!("{cut}~"));
    }
    Some(plain.to_owned())
}

/// A planned batch: the commit for the store and what the window becomes when it succeeds.
pub struct Plan {
    pub commit: LogCommit,
    next: Counters,
    added: Vec<(u64, u64)>,
    shift: usize,
}

#[derive(Debug)]
pub struct LogWindow {
    counters: Counters,
    tail: VecDeque<(u64, u64)>,
}

impl LogWindow {
    /// `start` carries the counters of the run row (a new run: all zero).
    pub fn new(start: &RunRow, limits: &RunLimits) -> Self {
        Self {
            counters: Counters {
                seq: start.log_seq,
                kept: start.log_bytes,
                dropped: start.log_dropped,
                head_bytes: start.log_bytes.min(limits.log_head_bytes),
                last_line: start.last_line.clone(),
            },
            tail: VecDeque::new(),
        }
    }

    pub fn plan(&self, limits: &RunLimits, lines: &[PendingLine]) -> Plan {
        let mut next = self.counters.clone();
        let mut stored = Vec::with_capacity(lines.len());
        let mut added = Vec::new();
        for line in lines {
            let size = size_of(&line.text);
            let head = next.head_bytes < limits.log_head_bytes;
            next.seq += 1;
            next.kept += size;
            if head {
                next.head_bytes += size;
            } else {
                added.push((next.seq, size));
            }
            if line.stream != Stream::Sys {
                next.last_line = last_line_of(&line.text).or(next.last_line);
            }
            stored.push(StoredLine {
                line: LogLine {
                    seq: next.seq,
                    stream: line.stream,
                    at: line.at.clone(),
                    text: line.text.clone(),
                },
                head,
            });
        }
        let (shift, cut) = self.evict(&mut next, &added, limits.log_cap_bytes);
        Plan {
            commit: LogCommit {
                lines: stored,
                drop_up_to: cut,
                log_bytes: next.kept,
                log_dropped: next.dropped,
                log_seq: next.seq,
                last_line: next.last_line.clone(),
            },
            next,
            added,
            shift,
        }
    }

    /// Drops the oldest non-head lines until the kept bytes fit the cap. Returns how many
    /// lines were dropped and the `seq` up to which the store deletes.
    fn evict(&self, next: &mut Counters, added: &[(u64, u64)], cap: u64) -> (usize, Option<u64>) {
        let total = self.tail.len() + added.len();
        let mut shift = 0;
        let mut cut = None;
        while next.kept > cap && shift < total {
            let (seq, size) = self
                .tail
                .get(shift)
                .copied()
                .unwrap_or_else(|| added[shift - self.tail.len()]);
            next.kept = next.kept.saturating_sub(size);
            next.dropped += 1;
            cut = Some(seq);
            shift += 1;
        }
        (shift, cut)
    }

    /// Takes the planned batch as committed.
    pub fn accept(&mut self, plan: &Plan) {
        self.counters = plan.next.clone();
        self.tail.extend(plan.added.iter().copied());
        self.tail.drain(..plan.shift);
    }
}
