//! What the run service tells the rest of the dashboard: run docs for the pages, lines for
//! the live tail, and warnings for the server log.

use std::sync::{Mutex, PoisonError};

use serde_json::Value;

use crate::record::{LogEvent, RunRow};

pub trait RunEvents: Send + Sync {
    /// The run doc of a run changed (a status change, or progress at most four times a
    /// second).
    fn project(&self, row: &RunRow);

    /// The run doc of a deleted run goes away.
    fn unproject(&self, scope: &str, rid: &str);

    /// Rebuilds every run doc of a scope (boot): docs of runs that no longer exist go.
    fn reproject(&self, scope: &str, rows: &[RunRow]);

    /// A flushed line for the log SSE clients of that run.
    fn log_line(&self, event: &LogEvent);

    /// Something the server log should say; `detail` names the run.
    fn warn(&self, message: &str, detail: &str);
}

/// Tells nobody anything.
#[derive(Clone, Copy, Debug, Default)]
pub struct NullEvents;

impl RunEvents for NullEvents {
    fn project(&self, _row: &RunRow) {}
    fn unproject(&self, _scope: &str, _rid: &str) {}
    fn reproject(&self, _scope: &str, _rows: &[RunRow]) {}
    fn log_line(&self, _event: &LogEvent) {}
    fn warn(&self, _message: &str, _detail: &str) {}
}

#[derive(Default)]
struct Recorded {
    docs: Vec<(String, Value)>,
    removed: Vec<(String, String)>,
    lines: Vec<LogEvent>,
    warnings: Vec<(String, String)>,
}

/// Keeps what it was told, for tests and for a server that wants to forward it later.
#[derive(Default)]
pub struct RecordingEvents {
    recorded: Mutex<Recorded>,
}

impl RecordingEvents {
    pub fn new() -> Self {
        Self::default()
    }

    fn lock(&self) -> std::sync::MutexGuard<'_, Recorded> {
        self.recorded.lock().unwrap_or_else(PoisonError::into_inner)
    }

    /// Every run doc written, as `(scope, doc)` in order.
    pub fn docs(&self) -> Vec<(String, Value)> {
        self.lock().docs.clone()
    }

    /// The statuses of the docs of one run, in order.
    pub fn statuses(&self, scope: &str, rid: &str) -> Vec<String> {
        self.lock()
            .docs
            .iter()
            .filter(|(s, d)| s == scope && d["id"] == rid)
            .filter_map(|(_, d)| d["status"].as_str().map(str::to_owned))
            .collect()
    }

    /// The newest doc of a run.
    pub fn doc(&self, scope: &str, rid: &str) -> Option<Value> {
        self.lock()
            .docs
            .iter()
            .rev()
            .find(|(s, d)| s == scope && d["id"] == rid)
            .map(|(_, d)| d.clone())
    }

    /// The run docs a page would have now: the newest doc of each run of the scope.
    pub fn current(&self, scope: &str) -> Vec<Value> {
        let mut docs: Vec<Value> = Vec::new();
        for (_, doc) in self.lock().docs.iter().filter(|(s, _)| s == scope) {
            match docs.iter_mut().find(|d| d["id"] == doc["id"]) {
                Some(existing) => *existing = doc.clone(),
                None => docs.push(doc.clone()),
            }
        }
        docs
    }

    pub fn removed(&self) -> Vec<(String, String)> {
        self.lock().removed.clone()
    }

    pub fn lines(&self) -> Vec<LogEvent> {
        self.lock().lines.clone()
    }

    pub fn warnings(&self) -> Vec<(String, String)> {
        self.lock().warnings.clone()
    }
}

impl RunEvents for RecordingEvents {
    fn project(&self, row: &RunRow) {
        self.lock().docs.push((row.scope.clone(), row.to_doc()));
    }

    fn unproject(&self, scope: &str, rid: &str) {
        let mut recorded = self.lock();
        recorded
            .docs
            .retain(|(s, d)| !(s == scope && d["id"] == rid));
        recorded.removed.push((scope.to_owned(), rid.to_owned()));
    }

    fn reproject(&self, scope: &str, rows: &[RunRow]) {
        let mut recorded = self.lock();
        recorded.docs.retain(|(s, _)| s != scope);
        recorded
            .docs
            .extend(rows.iter().map(|r| (scope.to_owned(), r.to_doc())));
    }

    fn log_line(&self, event: &LogEvent) {
        self.lock().lines.push(event.clone());
    }

    fn warn(&self, message: &str, detail: &str) {
        self.lock()
            .warnings
            .push((message.to_owned(), detail.to_owned()));
    }
}
