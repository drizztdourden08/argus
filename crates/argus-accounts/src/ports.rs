//! The outside world of the accounts service, as traits so tests fake it (the way
//! `argus-procs` does). Errors are plain text for the owner. Later pieces extend the traits
//! they own; each is `Send + Sync` and synchronous, so a caller on the async runtime wraps a
//! call in `spawn_blocking`.

use std::path::{Path, PathBuf};

use serde_json::Value;

use crate::handover::{AgentLoad, Cycle, Lead, WakeKind};
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
    /// When that access token expires, epoch milliseconds, if the login file says.
    fn expires_at(&self, cli_dir: &Path) -> Option<i64>;
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
    /// When a chat record file was last written, in milliseconds since the Unix epoch.
    fn session_mtime_ms(&self, scope: &Scope, id: &str) -> Result<i64, String>;
    /// The chat ids that have a tombstone (`deleted_<uuid>`) in the scope, as `local_<uuid>`.
    fn tombstones(&self, scope: &Scope) -> Result<Vec<String>, String>;
    /// Deletes a chat the way the app does: writes its tombstone `deleted_<uuid>` holding
    /// `at_ms` as text, then removes the record. The transcript is never touched.
    fn delete_session(&self, scope: &Scope, id: &str, at_ms: i64) -> Result<(), String>;
    /// The SHA-256 (lower hex) of a file of the scope, `None` when it does not exist. `file` is
    /// a bare file name: `local_<uuid>.json` or `archived-sessions.idx`.
    fn file_sha256(&self, scope: &Scope, file: &str) -> Result<Option<String>, String>;
}

/// The Claude Code transcripts, shared by all accounts.
pub trait Transcripts: Send + Sync {
    /// Every transcript file named `<cli_session_id>.jsonl`, over all projects.
    fn find(&self, cli_session_id: &str) -> Vec<PathBuf>;
    /// Whether the transcript holds a readable, unambiguous chat.
    fn is_valid(&self, path: &Path) -> bool;
}

/// The Desktop app's sidebar folders, kept in its `LevelDB`.
pub trait SidebarStore: Send + Sync {
    /// The three folder keys as one object, read from a copy so Claude may be running.
    fn read(&self) -> Result<Value, String>;
    /// Writes the three keys together; the Desktop app must be closed. `scanned` is the value
    /// the plan was made from; the write is refused when the live keys no longer equal it.
    fn write(&self, keys: &Value, scanned: &Value) -> Result<(), String>;
}

/// What a cancelled wait for the quit answers.
pub const QUIT_CANCELLED: &str = "cancelled while waiting for Claude Desktop to quit";
/// The detail shown while Desktop must be quit by hand.
pub const QUIT_BY_HAND: &str =
    "Quit Desktop from its tray icon (right-click, Quit); Argus continues as soon as it has quit";

/// How a close goes on when the polite close is not enough.
pub struct Quit<'a> {
    /// End the Claude processes by PID when the polite close failed.
    pub force: bool,
    /// Whether the owner cancelled; asked while waiting.
    pub cancelled: &'a dyn Fn() -> bool,
    /// Told the detail text when the close starts waiting for the owner.
    pub waiting: &'a mut dyn FnMut(&str),
}

/// The Desktop app as a program on this PC.
pub trait DesktopApp: Send + Sync {
    /// Whether Claude Desktop runs.
    fn is_running(&self) -> bool;
    /// Whether a chat is in the middle of a turn.
    fn turn_in_progress(&self) -> bool;
    /// Closes the app: politely first, then (with `quit.force`) by PID for the Claude processes
    /// only, never by name. A close that leaves the app in the tray waits for the owner to quit
    /// it, tells `quit.waiting` why, and stops when `quit.cancelled` turns true.
    ///
    /// # Errors
    /// [`QUIT_CANCELLED`] when cancelled, else why the app did not quit.
    fn close(&self, quit: &mut Quit) -> Result<(), String>;
    /// Starts the app again. Succeeds only once the app holds its lock (it runs).
    fn start(&self) -> Result<(), String>;
    /// Whether the app wrote its `config.json` after `since_ms` (epoch milliseconds).
    fn config_touched_since(&self, since_ms: i64) -> bool;
    /// The installed Claude package version, if it can be read.
    fn version(&self) -> Option<String>;
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
    /// The leads of every board, the Overlord included, that are working now: inside a turn,
    /// or with a live agent record. Idle leads are left out.
    fn working_leads(&self) -> Vec<Lead>;
    /// Wakes a lead's thread; `subject` is the `H-n` the wake is about.
    fn wake(
        &self,
        lead: (&str, &str),
        kind: WakeKind,
        subject: &str,
        text: &str,
    ) -> Result<(), String>;
    /// Leaves the Overlord a notice (not a wake) that a handover is ready.
    fn notice(&self, subject: &str, text: &str) -> Result<(), String>;
    /// The live agents of a lead with their model and, when known, their context size, for the
    /// estimator's prior. A port that cannot tell answers with none.
    fn agent_loads(&self, _board: &str, _thread: &str) -> Vec<AgentLoad> {
        Vec::new()
    }
}

/// Where the handover cycle and its counters are kept.
pub trait HandoverStore: Send + Sync {
    /// The next number of a counter (`SW` or `H`), starting at 1, kept across restarts.
    fn next_number(&self, counter: &str) -> Result<u64, String>;
    /// The last cycle, when there was one.
    fn load(&self) -> Result<Option<Cycle>, String>;
    /// Keeps the cycle and shows it: the `cycle` doc and one `Handovers` doc per lead.
    fn save(&self, cycle: &Cycle) -> Result<(), String>;
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
