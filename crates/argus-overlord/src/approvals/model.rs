//! One approval and the words around it.

use serde::Serialize;

/// Where an approval stands.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Status {
    Pending,
    Allowed,
    Denied,
    /// Nobody answered in time, or the child went away.
    Expired,
}

impl Status {
    /// The word kept in the table.
    #[must_use]
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Pending => "pending",
            Self::Allowed => "allowed",
            Self::Denied => "denied",
            Self::Expired => "expired",
        }
    }
}

/// What the owner chose.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Decision {
    Allow,
    Deny,
}

impl Decision {
    /// `allow` or `deny`.
    #[must_use]
    pub fn parse(word: &str) -> Option<Self> {
        match word {
            "allow" => Some(Self::Allow),
            "deny" => Some(Self::Deny),
            _ => None,
        }
    }
}

/// One asked tool call. `message` is the owner's reason for a denial and is not kept in the table.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Approval {
    pub id: String,
    pub at: String,
    pub tool: String,
    pub input_summary: String,
    pub status: Status,
    pub decided_at: Option<String>,
    pub decided_by: Option<String>,
    pub message: Option<String>,
    /// When it was asked, in milliseconds, for the timeout.
    #[serde(skip)]
    pub requested_ms: i64,
}
