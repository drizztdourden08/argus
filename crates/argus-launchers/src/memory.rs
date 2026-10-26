//! The in-memory storage: the reference for what a SQL store must do, and what the tests
//! and a database-less embedding use.

use std::collections::HashMap;
use std::sync::{Mutex, MutexGuard, PoisonError};

use argus_core::unique::unique;

use crate::attach::AttachmentRow;
use crate::record::{
    ListFilter, LogCommit, LogLine, LogRead, NewRun, ReadMode, RunPatch, RunRow, RunStatus,
    StoredLine,
};
use crate::store::{AttachmentStore, LogStore, RunStore};

#[derive(Default)]
struct Inner {
    rows: Vec<RunRow>,
    counters: HashMap<String, u64>,
    logs: HashMap<(String, String), Vec<StoredLine>>,
    attachments: HashMap<(String, String), Vec<AttachmentRow>>,
}

#[derive(Default)]
pub struct MemoryStore {
    inner: Mutex<Inner>,
}

impl MemoryStore {
    pub fn new() -> Self {
        Self::default()
    }

    fn lock(&self) -> MutexGuard<'_, Inner> {
        self.inner.lock().unwrap_or_else(PoisonError::into_inner)
    }

    /// Rewrites a stored row directly (tests age a run this way).
    pub fn rewrite(&self, scope: &str, rid: &str, change: impl FnOnce(&mut RunRow)) {
        if let Some(row) = self
            .lock()
            .rows
            .iter_mut()
            .find(|r| r.scope == scope && r.id == rid)
        {
            change(row);
        }
    }

    /// The number of stored log lines of a run, head and tail.
    pub fn stored_lines(&self, scope: &str, rid: &str) -> usize {
        self.lock()
            .logs
            .get(&(scope.to_owned(), rid.to_owned()))
            .map_or(0, Vec::len)
    }

    /// The number of attachment rows over all runs.
    pub fn attachment_count(&self) -> usize {
        self.lock().attachments.values().map(Vec::len).sum()
    }
}

fn matching<'a>(
    rows: &'a [RunRow],
    keep: impl Fn(&RunRow) -> bool + 'a,
) -> impl Iterator<Item = (usize, &'a RunRow)> {
    rows.iter().enumerate().filter(move |(_, r)| keep(r))
}

fn newest_first(mut found: Vec<(usize, &RunRow)>) -> Vec<RunRow> {
    found.sort_by(|a, b| {
        b.1.queued_at
            .cmp(&a.1.queued_at)
            .then_with(|| b.0.cmp(&a.0))
    });
    found.into_iter().map(|(_, r)| r.clone()).collect()
}

impl RunStore for MemoryStore {
    fn next_number(&self, scope: &str) -> u64 {
        let mut inner = self.lock();
        let counter = inner.counters.entry(scope.to_owned()).or_insert(0);
        *counter += 1;
        *counter
    }

    fn insert(&self, new: NewRun) -> RunRow {
        let row = RunRow {
            id: new.rid,
            scope: new.scope,
            name: new.name,
            kind: new.kind,
            status: RunStatus::Queued,
            exit_code: None,
            signal: None,
            pid: None,
            visible: new.visible,
            stdin: false,
            origin: new.origin,
            by: new.by,
            agent: new.agent,
            reason: new.reason,
            queued_at: new.queued_at,
            started_at: None,
            ended_at: None,
            log_bytes: 0,
            log_dropped: 0,
            log_seq: 0,
            orphan: false,
            error: None,
            last_line: None,
            cmd: new.cmd,
            cwd: new.cwd,
        };
        self.lock().rows.push(row.clone());
        row
    }

    fn get(&self, scope: &str, rid: &str) -> Option<RunRow> {
        let inner = self.lock();
        matching(&inner.rows, |r| r.scope == scope && r.id == rid)
            .next()
            .map(|(_, r)| r.clone())
    }

    fn update(&self, scope: &str, rid: &str, patch: &RunPatch) -> Option<RunRow> {
        let mut inner = self.lock();
        let row = inner
            .rows
            .iter_mut()
            .find(|r| r.scope == scope && r.id == rid)?;
        patch.apply_to(row);
        Some(row.clone())
    }

    fn list(&self, scope: Option<&str>, filter: &ListFilter) -> Vec<RunRow> {
        let inner = self.lock();
        let found = matching(&inner.rows, |r| {
            scope.is_none_or(|s| r.scope == s)
                && (filter.statuses.is_empty()
                    || filter.statuses.iter().any(|s| s == r.status.as_str()))
                && filter.name.as_deref().is_none_or(|n| r.name == n)
        })
        .collect();
        let mut rows = newest_first(found);
        rows.truncate(filter.row_limit());
        rows
    }

    fn live_by_name(&self, scope: &str, name: &str) -> Vec<RunRow> {
        let inner = self.lock();
        let mut found: Vec<_> = matching(&inner.rows, |r| {
            r.scope == scope && r.name == name && r.status.is_live()
        })
        .collect();
        found.reverse();
        found.into_iter().map(|(_, r)| r.clone()).collect()
    }

    fn live_rows(&self) -> Vec<RunRow> {
        let inner = self.lock();
        matching(&inner.rows, |r| r.status.is_live())
            .map(|(_, r)| r.clone())
            .collect()
    }

    fn queued_rows(&self) -> Vec<RunRow> {
        let inner = self.lock();
        let mut found: Vec<_> = matching(&inner.rows, |r| r.status == RunStatus::Queued).collect();
        found.sort_by(|a, b| a.1.queued_at.cmp(&b.1.queued_at).then(a.0.cmp(&b.0)));
        found.into_iter().map(|(_, r)| r.clone()).collect()
    }

    fn groups(&self) -> Vec<(String, String)> {
        let inner = self.lock();
        unique(
            inner
                .rows
                .iter()
                .map(|row| (row.scope.clone(), row.name.clone())),
        )
    }

    fn scopes(&self) -> Vec<String> {
        let inner = self.lock();
        unique(inner.rows.iter().map(|row| row.scope.clone()))
    }

    fn finished(&self, scope: &str, name: &str) -> Vec<RunRow> {
        let inner = self.lock();
        newest_first(
            matching(&inner.rows, |r| {
                r.scope == scope && r.name == name && !r.status.is_live()
            })
            .collect(),
        )
    }

    fn of_scope(&self, scope: &str) -> Vec<RunRow> {
        let inner = self.lock();
        matching(&inner.rows, |r| r.scope == scope)
            .map(|(_, r)| r.clone())
            .collect()
    }

    fn remove(&self, scope: &str, rid: &str) {
        let mut inner = self.lock();
        let key = (scope.to_owned(), rid.to_owned());
        inner.logs.remove(&key);
        inner.attachments.remove(&key);
        inner.rows.retain(|r| !(r.scope == scope && r.id == rid));
    }
}

impl LogStore for MemoryStore {
    fn commit(&self, scope: &str, rid: &str, commit: &LogCommit) -> Result<(), String> {
        let mut inner = self.lock();
        let row = inner
            .rows
            .iter_mut()
            .find(|r| r.scope == scope && r.id == rid)
            .ok_or_else(|| format!("no run {rid} in {scope}"))?;
        row.log_bytes = commit.log_bytes;
        row.log_dropped = commit.log_dropped;
        row.log_seq = commit.log_seq;
        row.last_line.clone_from(&commit.last_line);
        let lines = inner
            .logs
            .entry((scope.to_owned(), rid.to_owned()))
            .or_default();
        lines.extend(commit.lines.iter().cloned());
        if let Some(cut) = commit.drop_up_to {
            lines.retain(|l| l.head || l.line.seq > cut);
        }
        Ok(())
    }

    fn read(&self, scope: &str, rid: &str, read: &LogRead) -> Vec<LogLine> {
        let inner = self.lock();
        let Some(stored) = inner.logs.get(&(scope.to_owned(), rid.to_owned())) else {
            return Vec::new();
        };
        let mut lines = stored
            .iter()
            .map(|s| &s.line)
            .filter(|l| read.stream.is_none_or(|s| l.stream == s))
            .cloned()
            .collect::<Vec<_>>();
        lines.sort_by_key(|l| l.seq);
        match read.mode {
            ReadMode::Tail(n) => lines.split_off(lines.len().saturating_sub(n)),
            ReadMode::After { after, limit } => lines
                .into_iter()
                .filter(|l| l.seq > after)
                .take(limit)
                .collect(),
        }
    }
}

impl AttachmentStore for MemoryStore {
    fn attachments_of(&self, scope: &str, rid: &str) -> Vec<AttachmentRow> {
        self.lock()
            .attachments
            .get(&(scope.to_owned(), rid.to_owned()))
            .cloned()
            .unwrap_or_default()
    }

    fn add_attachment(&self, scope: &str, rid: &str, row: AttachmentRow) {
        self.lock()
            .attachments
            .entry((scope.to_owned(), rid.to_owned()))
            .or_default()
            .push(row);
    }
}
