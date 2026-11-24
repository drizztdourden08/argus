//! The record a stall rule produces: one stall that holds right now.

/// Which rule a stall comes from.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Kind {
    /// Rule 2: a wake nobody read.
    Unread,
    /// Rule 3: a dependency is open and its item is still `todo`.
    Unstarted,
    /// Rule 4: an idle lead with unblocked work.
    Idle,
    /// Rule 5: an item `doing` with no sign of work.
    Silent,
}

impl Kind {
    /// The wire name, as the stall key and the stall doc carry it.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Kind::Unread => "unread",
            Kind::Unstarted => "unstarted",
            Kind::Idle => "idle",
            Kind::Silent => "silent",
        }
    }
}

/// One stall found on a pass over the boards. The text is the line the Overlord reads.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Found {
    /// The rule.
    pub kind: Kind,
    /// The project key.
    pub project: String,
    /// The lead thread of the project; empty when it has none.
    pub lead: String,
    /// What the stall is about: `D-n` for a dependency, the lead thread, or `I-n` for an item.
    pub subject: String,
    /// When the stall began (epoch ms).
    pub since: i64,
    /// The line, with the age as of this pass.
    pub text: String,
}
