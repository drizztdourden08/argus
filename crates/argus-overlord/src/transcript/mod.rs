//! The transcript of a hosted session: every stream-json event, one redacted JSON line each, kept
//! by a [`TranscriptStore`]. The key is a known secret, so it is never stored, even when the model
//! prints it or a tool result quotes it. Reading is by line number, so a client resumes where it was.

mod line;
#[cfg(any(test, feature = "test-support"))]
mod memory;
mod name;
mod page;
mod stamp;

use argus_core::redact::Redactor;

use crate::ports::{ApiKey, TranscriptFault, TranscriptStore};

pub use line::redacted_line;
#[cfg(any(test, feature = "test-support"))]
pub use memory::MemoryTranscripts;
pub use name::{file_name, session_of, valid_session};
pub use page::{Page, lines_after};
pub use stamp::{KEEP_SESSIONS, Stamp, newest, stale};

/// The write and read side of the transcripts, over a store.
pub struct Transcript<S> {
    store: S,
    redactor: Redactor,
}

impl<S: TranscriptStore> Transcript<S> {
    /// A transcript that hides `key` wherever it shows up.
    #[must_use]
    pub fn new(store: S, key: &ApiKey) -> Self {
        Self {
            store,
            redactor: Redactor::new(&[key.expose().to_owned()]),
        }
    }

    /// Redacts one raw event line, appends it to the session file and answers the stored line.
    ///
    /// # Errors
    /// A fault when the session id is not usable or the store cannot write.
    pub fn record(&self, session: &str, raw: &str) -> Result<String, TranscriptFault> {
        let stored = redacted_line(&self.redactor, raw);
        self.store.append(session, &stored)?;
        Ok(stored)
    }

    /// The lines of a session after the first `after`.
    ///
    /// # Errors
    /// A fault when the store cannot read.
    pub fn read_after(&self, session: &str, after: usize) -> Result<Page, TranscriptFault> {
        self.store.read_after(session, after)
    }

    /// Deletes the session files past the last [`KEEP_SESSIONS`]; answers how many went.
    ///
    /// # Errors
    /// A fault when the store cannot list or delete.
    pub fn prune(&self) -> Result<usize, TranscriptFault> {
        let doomed = stale(&self.store.list()?, KEEP_SESSIONS);
        for session in &doomed {
            self.store.delete(session)?;
        }
        Ok(doomed.len())
    }
}

#[cfg(test)]
mod tests;
