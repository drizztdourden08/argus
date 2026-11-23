//! One line of the step log: the stage of the process it belongs to, how that stage stands, and
//! a plain sentence.

use serde::Serialize;

/// The stage of the process a line belongs to.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
pub(crate) enum Stage {
    /// What started the switch, and the checks before it.
    Detect,
    /// The minute before an automatic switch.
    Countdown,
    /// The handover cycle of the leads.
    Handover,
    /// The sync of the chats and folders.
    Sync,
    /// The Desktop steps: close, logins, resume, start.
    Switch,
    /// The checks after Claude started: the account, the sidebars, the resumes.
    Verify,
}

/// How the stage stands at that line.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
pub(crate) enum LineState {
    Running,
    Done,
    Failed,
    Skipped,
    Waiting,
}

/// A step-log line.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct Line {
    pub stage: Stage,
    pub state: LineState,
    pub text: String,
    pub detail: Option<String>,
}

impl Line {
    pub(crate) fn new(stage: Stage, how: LineState, text: impl Into<String>) -> Self {
        Line {
            stage,
            state: how,
            text: text.into(),
            detail: None,
        }
    }

    /// The line with a detail; an empty one is left out.
    #[must_use]
    pub(crate) fn with_detail(mut self, detail: &str) -> Self {
        if !detail.is_empty() {
            self.detail = Some(detail.to_owned());
        }
        self
    }
}
