//! The outside world of the accounts service, as traits so tests fake it (the way
//! `argus-procs` does). Errors are plain text for the owner. Later pieces extend the traits
//! they own; each is `Send + Sync` and synchronous, so a caller on the async runtime wraps a
//! call in `spawn_blocking`.

use std::path::Path;

use serde_json::Value;

use crate::types::{Account, Scope, Usage};

/// Reads an account's usage from the usage endpoint.
pub trait UsageSource: Send + Sync {
    /// The usage of the login that `access_token` belongs to. A 401 is an `Err` whose text
    /// starts with `unauthorized`, so the caller refreshes the token and retries once.
    fn fetch(&self, access_token: &str) -> Result<Usage, String>;
}

/// The CLI logins in the account folders. A token never leaves this trait except as the value
/// handed to [`UsageSource::fetch`]; it is never logged.
pub trait Credentials: Send + Sync {
    /// The current access token of the login in `cli_dir`.
    fn access_token(&self, cli_dir: &Path) -> Result<String, String>;
    /// Gets a new access token with the refresh token, writes it back and returns it.
    fn refresh(&self, cli_dir: &Path) -> Result<String, String>;
}

/// The Desktop app's chat records, archive indexes and tombstones, per scope.
pub trait Catalog: Send + Sync {
    /// The scopes that exist.
    fn scopes(&self) -> Result<Vec<Scope>, String>;
    /// The ids of the chat records of a scope.
    fn session_ids(&self, scope: &Scope) -> Result<Vec<String>, String>;
    /// One chat record.
    fn read_session(&self, scope: &Scope, id: &str) -> Result<Value, String>;
    /// Writes one chat record.
    fn write_session(&self, scope: &Scope, id: &str, record: &Value) -> Result<(), String>;
    /// The ids in the scope's archive index.
    fn archived(&self, scope: &Scope) -> Result<Vec<String>, String>;
    /// Replaces the scope's archive index.
    fn write_archived(&self, scope: &Scope, ids: &[String]) -> Result<(), String>;
}

/// The Desktop app's sidebar folders, kept in its LevelDB.
pub trait SidebarStore: Send + Sync {
    /// The three folder keys as one object, read from a copy so Claude may be running.
    fn read(&self) -> Result<Value, String>;
    /// Writes the three keys together; the Desktop app must be closed.
    fn write(&self, keys: &Value) -> Result<(), String>;
}

/// The Desktop app as a program on this PC.
pub trait DesktopApp: Send + Sync {
    /// Whether Claude Desktop runs.
    fn is_running(&self) -> bool;
    /// Whether a chat is in the middle of a turn.
    fn turn_in_progress(&self) -> bool;
    /// Closes the main window politely; never kills by name.
    fn close(&self) -> Result<(), String>;
    /// Starts the app again.
    fn start(&self) -> Result<(), String>;
}

/// The saved Desktop logins, one per account.
pub trait LoginVault: Send + Sync {
    /// Saves the login that is in place now as the login of `account`.
    fn save(&self, account: &Account) -> Result<(), String>;
    /// Puts the saved login of `account` in place.
    fn restore(&self, account: &Account) -> Result<(), String>;
    /// When the login of `account` was saved, ISO 8601, if it was.
    fn saved_at(&self, account: &Account) -> Option<String>;
}

/// The lead threads on the boards and their agents.
pub trait Threads: Send + Sync {
    /// The leads that are working now: in a turn, or with a live agent record.
    fn working_leads(&self) -> Vec<String>;
    /// Sends a wake with `text` to a thread.
    fn wake(&self, thread: &str, text: &str) -> Result<(), String>;
    /// A handover record, by id.
    fn read_record(&self, id: &str) -> Option<Value>;
    /// Writes a handover record.
    fn write_record(&self, id: &str, record: &Value) -> Result<(), String>;
}

/// Gets a turn started after Claude reopens, with one one-off scheduled task.
pub trait Resumer: Send + Sync {
    /// Schedules a task that sends `message` at `at_ms`; returns its id.
    fn schedule(&self, at_ms: i64, message: &str) -> Result<String, String>;
    /// Removes a scheduled task.
    fn cancel(&self, id: &str) -> Result<(), String>;
}

/// The time, so tests set it.
pub trait Clock: Send + Sync {
    /// Milliseconds since the Unix epoch.
    fn now_ms(&self) -> i64;
}
