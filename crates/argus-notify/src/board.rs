//! What the automatic pings read of a project's board, in the plain form the server fills in.

/// A test run handed to the owner: ready, with a live build, assigned to him.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct HandedRun {
    /// The run id.
    pub id: String,
    /// The run title.
    pub title: Option<String>,
    /// The thread of its plan.
    pub thread: Option<String>,
    /// When it was handed (its last change), ISO 8601.
    pub at: Option<String>,
    /// How many tests the plan holds.
    pub tests: usize,
}

/// One entry of the board.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct EntryFact {
    /// The entry id.
    pub id: String,
    /// The entry kind; a `question` is `coord ask`.
    pub kind: String,
    /// The status; `open` until it is answered.
    pub status: String,
    /// The author; `human` is the owner.
    pub author: String,
    /// True when the question has options (`needs` is `decision`).
    pub needs_decision: bool,
    /// The thread that asked.
    pub thread: Option<String>,
    /// The question text.
    pub text: String,
    /// When it was asked, ISO 8601.
    pub created_at: Option<String>,
    /// Trashed or archived.
    pub removed: bool,
}

/// A project board as it was exported.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct BoardFacts {
    /// True when the portfolio board retired the project.
    pub retired: bool,
    /// The run handed to the owner, if any.
    pub handed: Option<HandedRun>,
    /// The entries of the board.
    pub entries: Vec<EntryFact>,
}
