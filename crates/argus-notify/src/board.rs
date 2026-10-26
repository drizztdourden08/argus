//! What the automatic pings read of a project's board, in the plain form the server fills in.

/// The bundle a build was handed as.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Bundle {
    /// The bundle title.
    pub title: Option<String>,
    /// The thread that built it.
    pub thread: Option<String>,
}

/// One test of the board.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct TestFact {
    /// The build tag the test carries.
    pub build: Option<String>,
    /// Trashed or archived.
    pub removed: bool,
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
    /// The tag of the build handed to the owner, if any.
    pub handed_build: Option<String>,
    /// When it was handed, ISO 8601.
    pub handed_at: Option<String>,
    /// The bundle of that tag.
    pub bundle: Option<Bundle>,
    /// The tests of the board.
    pub tests: Vec<TestFact>,
    /// The entries of the board.
    pub entries: Vec<EntryFact>,
}
