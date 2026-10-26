//! In-memory implementations of the storage traits: what the tests run on, and a stand-in for the
//! server slice until SQLite is plugged in. Clones share their data.

use std::collections::HashMap;

use indexmap::IndexMap;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex, MutexGuard, PoisonError};

use crate::activity::{ActivityLine, ActivitySink};
use crate::board::BoardView;
use crate::docs::{Doc, DocSink};
use crate::faults::{ClearFault, Fault, FaultStore, NewFault};
use crate::source::BoardSource;
use crate::store::{RowsQuery, StoreResult, WakeStore, prune_cutoff};
use crate::time::ms_or_zero;
use crate::wake::{FinishedEvent, WakeInput, WakeRow, WakeStatus};
use argus_core::StorageError;
use argus_core::unique::unique;

fn locked<T>(mutex: &Mutex<T>) -> MutexGuard<'_, T> {
    mutex.lock().unwrap_or_else(PoisonError::into_inner)
}

#[derive(Default)]
struct Rows {
    rows: Vec<WakeRow>,
    counter: u64,
}

/// The wakes table in memory.
#[derive(Clone, Default)]
pub struct MemoryStore {
    inner: Arc<Mutex<Rows>>,
    failing: Arc<AtomicBool>,
}

impl MemoryStore {
    /// An empty table.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Make every `insert` fail (or work again).
    pub fn set_failing(&self, on: bool) {
        self.failing.store(on, Ordering::SeqCst);
    }

    /// Every row, oldest first.
    #[must_use]
    pub fn all(&self) -> Vec<WakeRow> {
        locked(&self.inner).rows.clone()
    }

    fn queued_where(&self, keep: impl Fn(&WakeRow) -> bool) -> Vec<WakeRow> {
        let inner = locked(&self.inner);
        inner
            .rows
            .iter()
            .filter(|r| r.status == WakeStatus::Queued && keep(r))
            .cloned()
            .collect()
    }
}

impl WakeStore for MemoryStore {
    fn insert(&self, wake: &WakeInput, at: &str) -> StoreResult<WakeRow> {
        if self.failing.load(Ordering::SeqCst) {
            return Err(StorageError("insert refused".to_owned()));
        }
        let mut inner = locked(&self.inner);
        inner.counter += 1;
        let n = inner.counter;
        let row = WakeRow::queued(n, wake, at);
        inner.rows.push(row.clone());
        Ok(row)
    }

    fn finish(&self, lead: &WakeRow, others: &[WakeRow], event: &FinishedEvent) -> StoreResult<()> {
        let mut inner = locked(&self.inner);
        for row in &mut inner.rows {
            if row.id == lead.id {
                row.lines = Some(event.lines.clone());
                row.count = event.count;
                row.status = event.status;
                row.listeners = event.listeners;
                row.delivered_at.clone_from(&event.delivered_at);
            } else if others.iter().any(|o| o.id == row.id) {
                row.status = WakeStatus::Coalesced;
                row.merged_into = Some(lead.id.clone());
            }
        }
        Ok(())
    }

    fn mark_delivered(&self, id: &str, listeners: u32, at: &str) -> StoreResult<()> {
        let mut inner = locked(&self.inner);
        if let Some(row) = inner.rows.iter_mut().find(|r| r.id == id) {
            row.status = WakeStatus::Delivered;
            row.listeners = listeners;
            row.delivered_at = Some(at.to_owned());
        }
        Ok(())
    }

    fn queued_events(
        &self,
        scope: &str,
        thread: &str,
        since_iso: &str,
    ) -> StoreResult<Vec<WakeRow>> {
        Ok(self.queued_where(|r| {
            r.lines.is_some()
                && r.scope == scope
                && (thread == "*" || r.thread == thread)
                && r.at.as_str() >= since_iso
        }))
    }

    fn unflushed(&self) -> StoreResult<Vec<WakeRow>> {
        Ok(self.queued_where(|r| r.lines.is_none()))
    }

    fn get(&self, id: &str) -> StoreResult<Option<WakeRow>> {
        Ok(locked(&self.inner)
            .rows
            .iter()
            .find(|r| r.id == id)
            .cloned())
    }

    fn rows(&self, scope: &str, query: &RowsQuery) -> StoreResult<Vec<WakeRow>> {
        let inner = locked(&self.inner);
        let thread = query.thread.as_deref().filter(|t| *t != "*");
        Ok(inner
            .rows
            .iter()
            .rev()
            .filter(|r| r.scope == scope && thread.is_none_or(|t| r.thread == t))
            .take(query.limit)
            .cloned()
            .collect())
    }

    fn last_at(
        &self,
        scope: &str,
        thread: &str,
        reason: &str,
        subject: Option<&str>,
    ) -> StoreResult<i64> {
        let inner = locked(&self.inner);
        Ok(inner
            .rows
            .iter()
            .filter(|r| r.scope == scope && r.thread == thread && r.reason == reason)
            .filter(|r| subject.is_none_or(|s| r.subject.as_deref() == Some(s)))
            .map(|r| ms_or_zero(Some(&r.at)))
            .max()
            .unwrap_or(0))
    }

    fn scopes(&self) -> StoreResult<Vec<String>> {
        let inner = locked(&self.inner);
        Ok(unique(inner.rows.iter().map(|row| row.scope.clone())))
    }

    fn prune(&self, now_ms: i64) -> StoreResult<Vec<String>> {
        let cutoff = prune_cutoff(now_ms);
        let mut inner = locked(&self.inner);
        let mut hit: Vec<String> = Vec::new();
        for row in inner.rows.iter().filter(|r| r.at < cutoff) {
            if !hit.contains(&row.scope) {
                hit.push(row.scope.clone());
            }
        }
        inner.rows.retain(|r| r.at >= cutoff);
        Ok(hit)
    }
}

type Collections = HashMap<(String, String), Vec<Doc>>;

/// The page documents in memory.
#[derive(Clone, Default)]
pub struct MemoryDocs {
    inner: Arc<Mutex<Collections>>,
}

impl MemoryDocs {
    /// No documents.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// The documents of a collection in a project.
    #[must_use]
    pub fn list(&self, scope: &str, collection: &str) -> Vec<Doc> {
        locked(&self.inner)
            .get(&(scope.to_owned(), collection.to_owned()))
            .cloned()
            .unwrap_or_default()
    }

    /// One document body.
    #[must_use]
    pub fn get(&self, scope: &str, collection: &str, id: &str) -> Option<serde_json::Value> {
        self.list(scope, collection)
            .into_iter()
            .find(|d| d.id == id)
            .map(|d| d.data)
    }
}

impl DocSink for MemoryDocs {
    fn replace(&self, scope: &str, collection: &str, docs: Vec<Doc>) -> StoreResult<()> {
        locked(&self.inner).insert((scope.to_owned(), collection.to_owned()), docs);
        Ok(())
    }
}

/// The `wake_faults` table in memory.
#[derive(Clone, Default)]
pub struct MemoryFaults {
    inner: Arc<Mutex<Vec<Fault>>>,
}

impl MemoryFaults {
    /// No faults.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }
}

impl FaultStore for MemoryFaults {
    fn open(&self, new: &NewFault) -> StoreResult<Option<Fault>> {
        let mut faults = locked(&self.inner);
        let taken = faults
            .iter()
            .any(|f| f.project == new.scope && f.thread == new.thread && f.cleared_at.is_none());
        if taken {
            return Ok(None);
        }
        let fault = Fault {
            id: i64::try_from(faults.len()).unwrap_or(i64::MAX) + 1,
            project: new.scope.clone(),
            thread: new.thread.clone(),
            since: new.since.clone(),
            reason: new.reason.clone(),
            wake_id: new.wake_id.clone(),
            detail: new.detail.clone(),
            cleared_at: None,
            cleared_by: None,
            note: None,
        };
        faults.push(fault.clone());
        Ok(Some(fault))
    }

    fn clear(&self, request: &ClearFault) -> StoreResult<Option<Fault>> {
        let mut faults = locked(&self.inner);
        let open = faults.iter_mut().find(|f| {
            f.project == request.scope && f.thread == request.thread && f.cleared_at.is_none()
        });
        Ok(open.map(|fault| {
            fault.cleared_at = Some(request.at.clone());
            fault.cleared_by = Some(request.by.clone());
            fault.note = Some(request.note.clone());
            fault.clone()
        }))
    }

    fn list(&self, all: bool) -> StoreResult<Vec<Fault>> {
        Ok(locked(&self.inner)
            .iter()
            .filter(|f| all || f.cleared_at.is_none())
            .cloned()
            .collect())
    }
}

/// The activity log in memory.
#[derive(Clone, Default)]
pub struct MemoryActivity {
    inner: Arc<Mutex<Vec<ActivityLine>>>,
}

impl MemoryActivity {
    /// An empty log.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Every line, oldest first.
    #[must_use]
    pub fn lines(&self) -> Vec<ActivityLine> {
        locked(&self.inner).clone()
    }
}

impl ActivitySink for MemoryActivity {
    fn add(&self, line: ActivityLine) {
        locked(&self.inner).push(line);
    }
}

/// The project boards in memory.
#[derive(Clone, Default)]
pub struct MemoryBoards {
    inner: Arc<Mutex<IndexMap<String, Arc<BoardView>>>>,
    retired: Arc<Mutex<Vec<String>>>,
}

impl MemoryBoards {
    /// No boards.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Put a board in place of the one of `scope`.
    pub fn set(&self, scope: &str, board: BoardView) {
        locked(&self.inner).insert(scope.to_owned(), Arc::new(board));
    }

    /// A retired project still has a board but is no longer listed in the portfolio.
    pub fn retire(&self, scope: &str) {
        locked(&self.retired).push(scope.to_owned());
    }

    /// Change the board of `scope` in place; nothing happens when it has none.
    pub fn edit(&self, scope: &str, change: impl FnOnce(&mut BoardView)) {
        if let Some(board) = locked(&self.inner).get_mut(scope) {
            change(Arc::make_mut(board));
        }
    }
}

impl BoardSource for MemoryBoards {
    fn scopes(&self) -> Vec<String> {
        let retired = locked(&self.retired).clone();
        locked(&self.inner)
            .keys()
            .filter(|k| !retired.contains(k))
            .cloned()
            .collect()
    }

    fn board(&self, scope: &str) -> Option<Arc<BoardView>> {
        locked(&self.inner).get(scope).cloned()
    }
}
