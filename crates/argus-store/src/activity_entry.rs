//! The activity log's rows: the level, the event a service reports, the row it becomes and the
//! doc the page reads.

use argus_core::{Coll, DocChange};
use serde_json::{Value, json};

/// The longest text of one line.
pub const TEXT_MAX: usize = 400;
const KIND_MAX: usize = 40;
const PROJECT_MAX: usize = 60;

/// How serious a line is.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ActivityLevel {
    /// Something happened.
    Info,
    /// Something needs a look.
    Warn,
    /// Something failed.
    Error,
}

impl ActivityLevel {
    /// The wire name and the stored value.
    #[must_use]
    pub fn as_str(self) -> &'static str {
        match self {
            ActivityLevel::Info => "info",
            ActivityLevel::Warn => "warn",
            ActivityLevel::Error => "error",
        }
    }

    /// The level a name stands for; `None` for a name that is not one.
    #[must_use]
    pub fn parse(name: &str) -> Option<ActivityLevel> {
        [
            ActivityLevel::Info,
            ActivityLevel::Warn,
            ActivityLevel::Error,
        ]
        .into_iter()
        .find(|l| l.as_str() == name)
    }

    /// The level a name stands for, `info` for a name that is not one.
    #[must_use]
    pub fn or_info(name: &str) -> ActivityLevel {
        Self::parse(name).unwrap_or(ActivityLevel::Info)
    }
}

/// An event a service reports.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NewActivity {
    /// What kind of event (`process-kill`, `wake-fault`...): a lower case word with dashes.
    pub kind: String,
    /// How serious it is.
    pub level: ActivityLevel,
    /// The sentence; blanks are folded and it is cut at 400 characters.
    pub text: String,
    /// The project it is about.
    pub project: Option<String>,
    /// The thread it is about.
    pub thread: Option<String>,
    /// When it happened, as a date text; now when absent or not a date.
    pub at: Option<String>,
}

impl NewActivity {
    /// An event with no project, thread or time.
    #[must_use]
    pub fn new(kind: impl Into<String>, level: ActivityLevel, text: impl Into<String>) -> Self {
        NewActivity {
            kind: kind.into(),
            level,
            text: text.into(),
            project: None,
            thread: None,
            at: None,
        }
    }

    /// The same event about a project.
    #[must_use]
    pub fn project(mut self, project: Option<String>) -> Self {
        self.project = project;
        self
    }

    /// The same event about a thread.
    #[must_use]
    pub fn thread(mut self, thread: Option<String>) -> Self {
        self.thread = thread;
        self
    }

    /// The same event at a given time.
    #[must_use]
    pub fn at(mut self, at: Option<String>) -> Self {
        self.at = at;
        self
    }
}

/// One stored line.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ActivityEntry {
    /// Its row number, rising for ever.
    pub n: i64,
    /// When it happened, ISO text.
    pub at: String,
    /// What kind of event.
    pub kind: String,
    /// How serious it is.
    pub level: ActivityLevel,
    /// The sentence.
    pub text: String,
    /// The project it is about.
    pub project: Option<String>,
    /// The thread it is about.
    pub thread: Option<String>,
}

impl ActivityEntry {
    /// The doc id: `a-<n>`.
    #[must_use]
    pub fn id(&self) -> String {
        format!("a-{}", self.n)
    }

    /// The doc the page reads.
    #[must_use]
    pub fn to_doc(&self) -> Value {
        json!({
            "id": self.id(),
            "n": self.n,
            "at": self.at,
            "kind": self.kind,
            "level": self.level.as_str(),
            "text": self.text,
            "project": self.project,
            "thread": self.thread,
        })
    }

    /// The change that sets its doc.
    #[must_use]
    pub fn set_change(&self) -> DocChange {
        DocChange::set(Coll::Activity, self.id(), self.to_doc())
    }
}

/// What a listing keeps.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ActivityFilter {
    /// How many lines at most (1 to 1000).
    pub limit: i64,
    /// Only this kind.
    pub kind: Option<String>,
    /// Only this level.
    pub level: Option<ActivityLevel>,
}

impl Default for ActivityFilter {
    fn default() -> Self {
        ActivityFilter {
            limit: 100,
            kind: None,
            level: None,
        }
    }
}

/// Blanks folded to one space, trimmed, cut at `max` characters with an ellipsis.
#[must_use]
pub fn one_line(text: &str, max: usize) -> String {
    let folded = text.split_whitespace().collect::<Vec<_>>().join(" ");
    if folded.chars().count() <= max {
        return folded;
    }
    let head: String = folded.chars().take(max.saturating_sub(1)).collect();
    format!("{head}\u{2026}")
}

/// Whether a kind is a lower case letter then up to 39 lower case letters, digits or dashes.
#[must_use]
pub fn valid_kind(kind: &str) -> bool {
    let mut chars = kind.chars();
    chars.next().is_some_and(|c| c.is_ascii_lowercase())
        && kind.len() <= KIND_MAX
        && chars.all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-')
}

/// A project or thread name as stored: cut at 60 characters, `None` when empty.
#[must_use]
pub fn short_name(name: Option<&str>) -> Option<String> {
    name.filter(|n| !n.is_empty())
        .map(|n| n.chars().take(PROJECT_MAX).collect())
}
