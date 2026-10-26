//! The activity log lines the wake modules write, and a fault store that writes them.

use std::sync::Arc;

use crate::faults::{ClearFault, Fault, FaultStore, NewFault};
use crate::store::StoreResult;

/// How serious an activity line is.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Level {
    /// Something happened.
    Info,
    /// Something needs a look.
    Warn,
    /// Something failed.
    Error,
}

/// One line of the activity log.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ActivityLine {
    /// What kind of event (`wake-fault`, `kanban-stale`...).
    pub kind: String,
    /// How serious it is.
    pub level: Level,
    /// The line.
    pub text: String,
    /// The project it is about.
    pub project: Option<String>,
    /// The thread it is about.
    pub thread: Option<String>,
}

/// Where activity lines go.
pub trait ActivitySink: Send + Sync {
    /// Append one line.
    fn add(&self, line: ActivityLine);
}

fn fault_words(reason: &str) -> String {
    match reason {
        "no-listener" => "a wake found nobody listening".to_owned(),
        "no-reaction" => "woken, but no sign of life".to_owned(),
        "delivery-error" => "a wake could not be written to its listener".to_owned(),
        "not-listening" => "its wake listener is not running".to_owned(),
        "board-closed" => "its board is closed".to_owned(),
        other => format!("wake fault {other}"),
    }
}

/// The activity line of a wake fault that opened.
#[must_use]
pub fn fault_line(fault: &Fault) -> String {
    let detail = fault
        .detail
        .as_deref()
        .filter(|d| !d.is_empty())
        .map_or(String::new(), |d| format!(" ({d})"));
    format!(
        "{}/{}: {}{detail}",
        fault.project,
        fault.thread,
        fault_words(&fault.reason)
    )
}

/// The activity line of a wake fault that cleared.
#[must_use]
pub fn cleared_line(fault: &Fault) -> String {
    let by = fault.cleared_by.as_deref().unwrap_or("");
    let who = if by == "registry" {
        "by itself".to_owned()
    } else {
        format!("by {by}")
    };
    let note = fault
        .note
        .as_deref()
        .filter(|n| !n.is_empty())
        .map_or(String::new(), |n| format!(": {n}"));
    format!(
        "{}/{}: wake fault ({}) cleared {who}{note}",
        fault.project, fault.thread, fault.reason
    )
}

/// A fault store that logs every fault that opens or clears.
pub struct ActivityFaults {
    inner: Box<dyn FaultStore>,
    sink: Arc<dyn ActivitySink>,
}

impl ActivityFaults {
    /// Wrap a fault store.
    #[must_use]
    pub fn new(inner: Box<dyn FaultStore>, sink: Arc<dyn ActivitySink>) -> Self {
        Self { inner, sink }
    }

    fn log(&self, fault: &Fault, level: Level, text: String) {
        self.sink.add(ActivityLine {
            kind: "wake-fault".to_owned(),
            level,
            text,
            project: Some(fault.project.clone()),
            thread: Some(fault.thread.clone()),
        });
    }
}

impl FaultStore for ActivityFaults {
    fn open(&self, fault: &NewFault) -> StoreResult<Option<Fault>> {
        let opened = self.inner.open(fault)?;
        if let Some(f) = &opened {
            self.log(f, Level::Warn, fault_line(f));
        }
        Ok(opened)
    }

    fn clear(&self, request: &ClearFault) -> StoreResult<Option<Fault>> {
        let cleared = self.inner.clear(request)?;
        if let Some(f) = &cleared {
            self.log(f, Level::Info, cleared_line(f));
        }
        Ok(cleared)
    }

    fn list(&self, all: bool) -> StoreResult<Vec<Fault>> {
        self.inner.list(all)
    }
}
