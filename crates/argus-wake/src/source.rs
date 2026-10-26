//! Where the wake logic reads the project boards from.

use std::sync::Arc;

use crate::board::BoardView;

/// The boards of every project on this machine.
pub trait BoardSource: Send + Sync {
    /// The project keys that have a board (the portfolio board included).
    fn scopes(&self) -> Vec<String>;

    /// The board of a project as the last export left it.
    fn board(&self, scope: &str) -> Option<Arc<BoardView>>;
}

/// What the agent reminders ask the live Claude tasks.
pub trait AgentLive: Send + Sync {
    /// When the agent last showed a sign of life in its transcript (ISO), when Claude knows it.
    fn last_activity_at(&self, scope: &str, agent: &str) -> Option<String>;
}

/// An [`AgentLive`] that knows nothing.
#[derive(Debug, Clone, Copy, Default)]
pub struct NoLive;

impl AgentLive for NoLive {
    fn last_activity_at(&self, _scope: &str, _agent: &str) -> Option<String> {
        None
    }
}
