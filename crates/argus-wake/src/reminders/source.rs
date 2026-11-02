//! The notices that stay open until somebody resolves them.

/// One open notice the Overlord must resolve.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OpenNotice {
    /// Its reason (`worktree-stale`, `remote-sync-due`).
    pub reason: String,
    /// The project it is about.
    pub project: String,
    /// What it is about inside the project.
    pub subject: String,
}

/// Where the open notices of one reason come from. The server plugs in the ones the wake crate cannot see.
pub trait NoticeSource: Send + Sync {
    /// What is open now.
    fn open(&self, now_ms: i64) -> Vec<OpenNotice>;
}
