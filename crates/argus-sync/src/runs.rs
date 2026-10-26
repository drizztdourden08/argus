//! What the cards need from the runs service, which a later slice provides.

use serde_json::Value;

/// The runs of a project, as the portfolio cards show them.
pub trait RunsView: Send + Sync {
    /// How many runs of the project are running.
    fn running(&self, key: &str) -> usize;

    /// The newest run of a def as the page sees it (no command, no folder), or `None`.
    fn last_run(&self, key: &str, name: &str) -> Option<Value>;
}
