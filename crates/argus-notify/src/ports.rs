//! What the service reads and writes, as small traits the server and the tests provide.

use std::error::Error;
use std::fmt;
use std::sync::Arc;

use crate::clock::Clock;
use crate::config::NotifyConfig;
use crate::deliver::Delivery;
use crate::record::{Filter, NewNotification, Notification};

/// A storage failure: the message says which.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StoreError(pub String);

impl fmt::Display for StoreError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

impl Error for StoreError {}

/// The table `notifications`, its docs and the marker of the automatic pings.
pub trait NotifyStore: Send + Sync {
    /// Inserts the notification as the next row, keeps the newest `KEEP_ROWS` rows and the same
    /// docs, and returns the stored notification.
    ///
    /// # Errors
    ///
    /// When the database refuses the write.
    fn insert(&self, new: NewNotification) -> Result<Notification, StoreError>;

    /// The notifications that match, newest first.
    ///
    /// # Errors
    ///
    /// When the database cannot be read.
    fn list(&self, filter: &Filter) -> Result<Vec<Notification>, StoreError>;

    /// Writes the docs of the stored notifications again, after a start or a hard resync.
    ///
    /// # Errors
    ///
    /// When the docs cannot be written.
    fn mirror(&self) -> Result<(), StoreError>;

    /// The JSON of the markers (kv `notify:seen`), if any were saved.
    fn seen_load(&self) -> Option<String>;

    /// Saves the JSON of the markers.
    ///
    /// # Errors
    ///
    /// When the value cannot be written.
    fn seen_save(&self, json: &str) -> Result<(), StoreError>;
}

/// The `notify` key of `config.json`.
pub trait ConfigStore: Send + Sync {
    /// The config now in effect.
    fn load(&self) -> NotifyConfig;

    /// Writes the config; on an error nothing changed.
    ///
    /// # Errors
    ///
    /// When `config.json` cannot be written.
    fn save(&self, config: &NotifyConfig) -> Result<(), StoreError>;
}

/// The display name of a project, for titles.
pub trait ProjectNames: Send + Sync {
    /// The name the board gives the project, if it has one.
    fn name(&self, key: &str) -> Option<String>;
}

/// How serious an activity line is.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ActivityLevel {
    /// Something happened.
    Info,
    /// Something went wrong.
    Warn,
}

/// One line of the activity log, kind `note`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ActivityLine {
    /// How serious it is.
    pub level: ActivityLevel,
    /// The sentence.
    pub text: String,
    /// The project it is about.
    pub project: Option<String>,
    /// The thread it is about.
    pub thread: Option<String>,
}

/// The activity log.
pub trait ActivitySink: Send + Sync {
    /// Adds a line.
    fn note(&self, line: ActivityLine);
}

/// The fan-out to the page: the server pushes each recorded notification over SSE.
pub trait PageEvents: Send + Sync {
    /// A notification was recorded.
    fn recorded(&self, notification: &Notification);
}

/// Everything the service talks to.
pub struct Ports {
    /// The table and the markers.
    pub store: Arc<dyn NotifyStore>,
    /// The config.
    pub config: Arc<dyn ConfigStore>,
    /// The clock.
    pub clock: Arc<dyn Clock>,
    /// The project names.
    pub names: Arc<dyn ProjectNames>,
    /// The activity log.
    pub activity: Arc<dyn ActivitySink>,
    /// The page fan-out.
    pub events: Arc<dyn PageEvents>,
    /// The OS toast and the tray fallback.
    pub delivery: Delivery,
}
