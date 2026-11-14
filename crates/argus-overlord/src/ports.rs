//! The ports the hosted Overlord needs from the outside world.

use std::fmt;
use std::future::Future;
use std::path::PathBuf;

use crate::session::env::ChildEnv;
use crate::spend::{Last, SpendRow};
use crate::transcript::{Page, Stamp};

/// The Anthropic key. `Debug` prints a redaction; only `expose` gives the text.
#[derive(Clone, PartialEq, Eq)]
pub struct ApiKey(String);

impl ApiKey {
    /// Wraps a key.
    #[must_use]
    pub fn new(value: impl Into<String>) -> Self {
        Self(value.into())
    }

    /// The text, for the one place that puts it in the child env.
    #[must_use]
    pub fn expose(&self) -> &str {
        &self.0
    }
}

impl fmt::Debug for ApiKey {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("ApiKey(<redacted>)")
    }
}

/// Why the key store failed. Carries a reason, never the key.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SecretFault(pub String);

impl fmt::Display for SecretFault {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

impl std::error::Error for SecretFault {}

/// Where the one API key is kept. The real one is DPAPI in `argus-native`.
pub trait SecretStore {
    /// Saves the key, replacing any earlier one.
    ///
    /// # Errors
    /// A fault when the key cannot be kept.
    fn save(&self, key: &str) -> Result<(), SecretFault>;

    /// The key, or `None` when none is saved.
    ///
    /// # Errors
    /// A fault when the key cannot be read.
    fn load(&self) -> Result<Option<ApiKey>, SecretFault>;

    /// Removes the key.
    ///
    /// # Errors
    /// A fault when the key cannot be removed.
    fn clear(&self) -> Result<(), SecretFault>;

    /// The last 4 characters of the key, the only view of it besides `load`.
    ///
    /// # Errors
    /// A fault when the key cannot be read.
    fn hint(&self) -> Result<Option<String>, SecretFault>;
}

/// What a child is started with.
#[derive(Debug, Clone)]
pub struct ChildSpec {
    pub program: PathBuf,
    pub args: Vec<String>,
    /// The whole env of the child; the parent's is not inherited.
    pub env: ChildEnv,
    pub cwd: PathBuf,
    /// Where stderr lines go. The adapter replaces the key with a redaction first.
    pub stderr_log: PathBuf,
}

/// Why a host call failed. Carries a reason, never the key.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HostFault(pub String);

impl fmt::Display for HostFault {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

impl std::error::Error for HostFault {}

/// Lines of one output stream of the child, ended by `None` when the stream closes.
pub trait LineStream: Send {
    /// The next line without its newline, or `None` at the end.
    fn next_line(&mut self) -> impl Future<Output = Option<String>> + Send;
}

/// A started child: its pid and its stdout lines (stderr goes to the log).
pub struct SpawnedChild<S> {
    pub pid: u32,
    pub stdout: S,
}

/// Starts the `claude` child and ends it. Every call is scoped to a pid the host started, never
/// to an image name.
pub trait ChildHost: Send + Sync + 'static {
    type Lines: LineStream;

    /// Starts the child hidden, with piped stdin and stdout.
    ///
    /// # Errors
    /// A fault when the process cannot start.
    fn spawn(&self, spec: &ChildSpec) -> Result<SpawnedChild<Self::Lines>, HostFault>;

    /// Writes one line (a newline is added) to the stdin of `pid`.
    ///
    /// # Errors
    /// A fault when `pid` is not ours or its stdin is closed.
    fn write_line(
        &self,
        pid: u32,
        line: &str,
    ) -> impl Future<Output = Result<(), HostFault>> + Send;

    /// Closes the stdin of `pid`, which asks a stream-json child to finish.
    fn close_stdin(&self, pid: u32) -> impl Future<Output = ()> + Send;

    /// Closes stdin, waits for the exit up to a grace period, then ends the tree of `pid`.
    fn stop(&self, pid: u32) -> impl Future<Output = ()> + Send;

    /// Ends the tree of `pid` at once.
    fn kill_tree(&self, pid: u32) -> impl Future<Output = ()> + Send;
}

/// Why the transcript store failed. Carries a reason, never an event.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TranscriptFault(pub String);

impl fmt::Display for TranscriptFault {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

impl std::error::Error for TranscriptFault {}

/// Where the redacted event lines of each session are kept, one file per session.
pub trait TranscriptStore: Send + Sync {
    /// Appends one line (a newline is added) to the session file, creating it.
    ///
    /// # Errors
    /// A fault when the session id is not usable or the file cannot be written.
    fn append(&self, session: &str, line: &str) -> Result<(), TranscriptFault>;

    /// The complete lines after the first `after`; an empty page when the session has no file.
    ///
    /// # Errors
    /// A fault when the file cannot be read.
    fn read_after(&self, session: &str, after: usize) -> Result<Page, TranscriptFault>;

    /// Every session file with its modified time.
    ///
    /// # Errors
    /// A fault when the folder cannot be read.
    fn list(&self) -> Result<Vec<Stamp>, TranscriptFault>;

    /// Deletes the session file.
    ///
    /// # Errors
    /// A fault when the file cannot be removed.
    fn delete(&self, session: &str) -> Result<(), TranscriptFault>;
}

/// Why the spend ledger failed. Carries a reason, never a row.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SpendFault(pub String);

impl fmt::Display for SpendFault {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

impl std::error::Error for SpendFault {}

/// Where the cost of each turn is kept, and whether the month was warned about.
pub trait SpendLedger: Send + Sync {
    /// Adds one row.
    ///
    /// # Errors
    /// A fault when the row cannot be kept.
    fn record(&self, row: &SpendRow) -> Result<(), SpendFault>;

    /// The sum of the deltas of the month (`YYYY-MM`).
    ///
    /// # Errors
    /// A fault when the ledger cannot be read.
    fn month_total(&self, month: &str) -> Result<f64, SpendFault>;

    /// The newest row of the session as its last cumulative value.
    ///
    /// # Errors
    /// A fault when the ledger cannot be read.
    fn last(&self, session: &str) -> Result<Option<Last>, SpendFault>;

    /// True when the month already had its warning.
    ///
    /// # Errors
    /// A fault when the flag cannot be read.
    fn warned(&self, month: &str) -> Result<bool, SpendFault>;

    /// Marks the month as warned.
    ///
    /// # Errors
    /// A fault when the flag cannot be kept.
    fn set_warned(&self, month: &str) -> Result<(), SpendFault>;
}

#[cfg(test)]
mod tests {
    use super::*;

    // test-tag: critical
    #[test]
    fn the_api_key_debug_is_redacted() {
        assert_eq!(
            format!("{:?}", ApiKey::new("sk-ant-secret")),
            "ApiKey(<redacted>)"
        );
    }
}
