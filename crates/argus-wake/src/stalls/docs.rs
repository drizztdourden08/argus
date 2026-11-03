//! The stall as the page and the CLI read it: one document per open stall in `_portfolio/stalls`.

use serde::Serialize;

use super::book::Open;
use crate::docs::Doc;
use crate::time::iso;

/// An open stall, in the shape of its page document.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct StallInfo {
    /// The stall key, `<project>:<kind>:<subject>:<sinceMs>`.
    pub id: String,
    /// The project key.
    pub project: String,
    /// The thread the stall is about; empty when the project has no lead.
    pub lead: String,
    /// `unread`, `unstarted`, `idle` or `silent`.
    pub kind: String,
    /// What it is about.
    pub subject: String,
    /// The line, with the age as of the last tick.
    pub text: String,
    /// When the condition began (ISO).
    pub since: String,
    /// When Argus first saw it (ISO).
    pub raised_at: String,
    /// When the Overlord acknowledged it (ISO).
    pub acked_at: Option<String>,
    /// Who acknowledged it.
    pub acked_by: Option<String>,
    /// What was done about it.
    pub ack_note: Option<String>,
}

impl StallInfo {
    pub(super) fn of(open: &Open) -> Self {
        Self {
            id: open.key.clone(),
            project: open.found.project.clone(),
            lead: open.found.lead.clone(),
            kind: open.found.kind.as_str().to_owned(),
            subject: open.found.subject.clone(),
            text: open.found.text.clone(),
            since: iso(open.found.since),
            raised_at: iso(open.raised_at),
            acked_at: open.ack.as_ref().map(|a| a.at.clone()),
            acked_by: open.ack.as_ref().map(|a| a.by.clone()),
            ack_note: open.ack.as_ref().map(|a| a.note.clone()),
        }
    }

    /// The page document of the stall.
    #[must_use]
    pub fn to_doc(&self) -> Doc {
        Doc {
            id: self.id.clone(),
            data: serde_json::to_value(self).unwrap_or_default(),
        }
    }
}
