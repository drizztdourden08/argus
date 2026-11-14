//! The words of the relay: who is a target, who can relay, what to do, what was done.

use serde_json::{Value, json};

use crate::time::iso;

/// A project and a thread.
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct ThreadKey {
    /// Project key.
    pub project: String,
    /// The thread.
    pub thread: String,
}

impl ThreadKey {
    /// The key of a thread.
    #[must_use]
    pub fn new(project: &str, thread: &str) -> Self {
        Self {
            project: project.to_owned(),
            thread: thread.to_owned(),
        }
    }
}

/// A thread to wake, with the CLI session id its last hook saw.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Ward {
    /// Which thread.
    pub key: ThreadKey,
    /// Its CLI session id, when known.
    pub session_id: Option<String>,
}

/// A thread that holds a queued wake and no listener.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RelayTarget {
    /// The thread.
    pub ward: Ward,
    /// When its oldest queued wake was made (epoch ms).
    pub since_ms: i64,
}

/// A thread with a v2 listener: it can carry a relay note.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RelayLive {
    /// The thread.
    pub key: ThreadKey,
    /// Is a turn running now.
    pub in_turn: bool,
    /// Is it the Overlord's main thread.
    pub main: bool,
}

/// What the server has to do.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Action {
    /// Post one note to `via` that names every target.
    Note {
        /// The relay.
        via: ThreadKey,
        /// The threads it is to wake.
        targets: Vec<Ward>,
    },
    /// Resume the target's session headless.
    Headless(Ward),
    /// Tell the owner the target could not be woken.
    Notice(Ward),
}

/// The kinds of attempt, in the order they escalate.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AttemptKind {
    /// A relay note was posted.
    Note,
    /// The headless resume was started.
    Headless,
    /// The owner was told.
    Notified,
}

impl AttemptKind {
    fn word(self) -> &'static str {
        match self {
            Self::Note => "note",
            Self::Headless => "headless",
            Self::Notified => "notified",
        }
    }
}

/// The last attempt made for a target.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LastAttempt {
    /// What was done.
    pub kind: AttemptKind,
    /// When (epoch ms).
    pub at_ms: i64,
    /// The relay thread of a note.
    pub via: Option<String>,
}

impl LastAttempt {
    /// The row field of `GET /api/wakes/due`: `kind`, `at` (ISO) and `via`.
    #[must_use]
    pub fn json(&self) -> Value {
        json!({ "kind": self.kind.word(), "at": iso(self.at_ms), "via": self.via })
    }
}

/// How the relay is switched (`wakeRelay.*`).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RelaySettings {
    /// The whole relay.
    pub enabled: bool,
    /// The headless resume.
    pub headless: bool,
}

impl Default for RelaySettings {
    fn default() -> Self {
        Self {
            enabled: true,
            headless: true,
        }
    }
}

impl RelaySettings {
    /// The settings of a parsed config.json: anything but a boolean keeps its default.
    #[must_use]
    pub fn from_json(config: &Value) -> Self {
        let flag = |key: &str| config.get(key).and_then(Value::as_bool).unwrap_or(true);
        Self {
            enabled: flag("enabled"),
            headless: flag("headless"),
        }
    }
}

/// Where the actions go; the server plugs it in. Calls come under the service lock, so they return fast.
pub trait RelayPort: Send + Sync {
    /// Post the relay note.
    fn note(&self, via: &ThreadKey, targets: &[Ward]);
    /// Start the headless resume.
    fn headless(&self, target: &Ward);
    /// Tell the owner.
    fn notice(&self, target: &Ward);
}
