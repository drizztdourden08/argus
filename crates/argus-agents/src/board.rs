//! The part of a project board these services look at: its agents and its threads.

use crate::record::{Record, id_of, is_set, text};

/// A board as the services see it, both collections in file order.
#[derive(Debug, Clone, Default)]
pub struct Board {
    /// The `agents` collection.
    pub agents: Vec<Record>,
    /// The `threads` collection.
    pub threads: Vec<Record>,
}

/// Whether a thread has ended.
#[must_use]
pub fn thread_ended(thread: &Record) -> bool {
    matches!(text(thread, "status"), Some("done" | "archived"))
}

impl Board {
    /// The thread with this id.
    #[must_use]
    pub fn thread(&self, id: &str) -> Option<&Record> {
        self.threads.iter().find(|t| id_of(t) == id)
    }

    /// The agent with this id, for writing.
    pub fn agent_mut(&mut self, id: &str) -> Option<&mut Record> {
        self.agents.iter_mut().find(|a| id_of(a) == id)
    }

    /// The first coordinator thread that has not ended.
    #[must_use]
    pub fn coordinator(&self) -> Option<String> {
        self.threads
            .iter()
            .find(|t| text(t, "role") == Some("coordinator") && !thread_ended(t))
            .map(|t| id_of(t).to_owned())
    }

    /// The thread to tell about an agent: its own while it lives, else the coordinator.
    #[must_use]
    pub fn wake_target(&self, agent: &Record) -> Option<String> {
        let own = text(agent, "thread").and_then(|id| self.thread(id));
        match own {
            Some(t) if !thread_ended(t) && !is_set(t, "placeholder") => Some(id_of(t).to_owned()),
            _ => self.coordinator(),
        }
    }
}
