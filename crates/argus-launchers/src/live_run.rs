//! The live record of a running process: its log writer, its stdin pipe, what a stop or a
//! timeout decided, and the signals the supervisor listens to.

use std::sync::{Mutex, MutexGuard, PoisonError};
use std::time::Duration;

use tokio::sync::{Notify, watch};
use tokio::time::Instant;

use crate::def::RunDef;
use crate::host::BoxWrite;
use crate::log_writer::LogWriter;
use crate::redact::Redactor;

/// Decisions taken from outside the supervisor, and the throttle of progress docs.
#[derive(Debug)]
pub(crate) struct Control {
    pub stop_by: Option<String>,
    pub timed_out: bool,
    pub dirty: bool,
    pub last_project: Instant,
}

impl Control {
    pub(crate) fn new() -> Self {
        Self {
            stop_by: None,
            timed_out: false,
            dirty: false,
            last_project: Instant::now(),
        }
    }
}

pub(crate) struct LiveRun {
    pub id: String,
    pub scope: String,
    pub rid: String,
    pub name: String,
    pub def: RunDef,
    pub visible: bool,
    pub interactive: bool,
    pub launched_at_ms: i64,
    pub pid: Option<u32>,
    pub redact: Redactor,
    pub writer: Mutex<LogWriter>,
    pub control: Mutex<Control>,
    pub stdin: tokio::sync::Mutex<Option<BoxWrite>>,
    pub force: Notify,
    pub release: Notify,
    pub done: watch::Sender<bool>,
}

impl LiveRun {
    pub(crate) fn writer(&self) -> MutexGuard<'_, LogWriter> {
        self.writer.lock().unwrap_or_else(PoisonError::into_inner)
    }

    pub(crate) fn control(&self) -> MutexGuard<'_, Control> {
        self.control.lock().unwrap_or_else(PoisonError::into_inner)
    }

    /// A line from the dashboard itself, redacted like the program's own.
    pub(crate) fn sys(&self, text: &str) {
        self.writer().sys(text);
    }

    /// Resolves once the run reached a final state (or `limit` passed).
    pub(crate) async fn wait_done(&self, limit: Duration) {
        let mut done = self.done.subscribe();
        let _ = tokio::time::timeout(limit, done.wait_for(|finished| *finished)).await;
    }
}
