//! What the services read and write besides the file system: the boards, the wake engine, the
//! activity log and the live agents docs. The server implements them; tests use fakes.

use crate::board::Board;
use crate::error::AgentsError;
use crate::fs::FileSystem;
use crate::record::Record;
use argus_core::clock::Clock;
use std::sync::Arc;

/// The project boards.
pub trait BoardStore: Send + Sync {
    /// The keys of every project.
    fn projects(&self) -> Vec<String>;
    /// The exported mirror of a board, the cheap place to look first.
    fn snapshot(&self, key: &str) -> Option<Board>;
    /// Runs `change` on the board under its lock and saves it. The implementation loads the file,
    /// calls `change` once, writes the agents it changed back by id and lets the sync service
    /// export the result like any other write.
    ///
    /// # Errors
    /// The board is locked for too long or cannot be saved.
    fn write(&self, key: &str, change: &mut dyn FnMut(&mut Board)) -> Result<(), AgentsError>;
}

/// A wake for a thread, as the wake engine takes it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WakeEvent {
    /// The board key the wake is about.
    pub scope: String,
    /// The thread to wake.
    pub thread: String,
    /// The wake reason, such as `agent-idle`.
    pub reason: String,
    /// What the wake is about, such as an agent id.
    pub subject: String,
    /// The sentence the thread reads.
    pub text: String,
}

/// The wake engine.
pub trait WakeSink: Send + Sync {
    /// Hands the wakes over; the engine coalesces and queues them.
    ///
    /// # Errors
    /// The engine refused them.
    fn submit(&self, events: &[WakeEvent]) -> Result<(), AgentsError>;
}

/// One line of the activity log.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ActivityLine {
    /// The line kind, always `agent` here.
    pub kind: String,
    /// `info` or `warn`.
    pub level: String,
    /// The one-liner.
    pub text: String,
    /// The board key.
    pub project: String,
    /// The thread the line is about.
    pub thread: Option<String>,
}

/// The activity log.
pub trait ActivitySink: Send + Sync {
    /// Adds a line.
    fn add(&self, line: ActivityLine);
}

/// The live agents docs of the doc store, where the watcher keeps its `live` field.
pub trait LiveDocs: Send + Sync {
    /// The data of an agents doc.
    fn get(&self, key: &str, id: &str) -> Option<Record>;
    /// Replaces the data of an agents doc.
    ///
    /// # Errors
    /// The store refused the write.
    fn set(&self, key: &str, id: &str, data: Record) -> Result<(), AgentsError>;
}

/// The ports every service shares.
#[derive(Clone)]
pub struct Ports {
    /// The boards.
    pub store: Arc<dyn BoardStore>,
    /// The file system.
    pub fs: Arc<dyn FileSystem>,
    /// The clock.
    pub clock: Arc<dyn Clock>,
    /// The wake engine; without it the reconciler only reports and closes on its own clock.
    pub wakes: Option<Arc<dyn WakeSink>>,
    /// The activity log.
    pub activity: Option<Arc<dyn ActivitySink>>,
}

impl Ports {
    /// The ports with no wake engine and no activity log.
    #[must_use]
    pub fn new(store: Arc<dyn BoardStore>, fs: Arc<dyn FileSystem>, clock: Arc<dyn Clock>) -> Self {
        Self {
            store,
            fs,
            clock,
            wakes: None,
            activity: None,
        }
    }

    /// Adds the wake engine.
    #[must_use]
    pub fn with_wakes(mut self, wakes: Arc<dyn WakeSink>) -> Self {
        self.wakes = Some(wakes);
        self
    }

    /// Adds the activity log.
    #[must_use]
    pub fn with_activity(mut self, activity: Arc<dyn ActivitySink>) -> Self {
        self.activity = Some(activity);
        self
    }

    /// Writes an activity line when there is a log.
    pub fn log_activity(&self, line: ActivityLine) {
        if let Some(sink) = &self.activity {
            sink.add(line);
        }
    }
}
