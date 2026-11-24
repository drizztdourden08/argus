//! Who acts: the thread acted as, its authority and the id recorded on every change.

/// The authority of a caller.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Role {
    /// The owner (the page entry being applied).
    Human,
    /// The project lead: changes anything except restoring from a backup.
    Lead,
    /// An ordinary thread: changes only its own records.
    Thread,
}

/// A caller of an op.
#[derive(Debug, Clone)]
pub struct Who {
    /// The id of the thread acted as, `None` for the owner.
    pub thread: Option<String>,
    /// The authority.
    pub role: Role,
    /// The id recorded on every change; defaults from the role and the thread.
    pub actor: Option<String>,
}

impl Who {
    /// The owner: `{ thread: null, role: 'human', actor: 'human' }`.
    #[must_use]
    pub fn human() -> Self {
        Who {
            thread: None,
            role: Role::Human,
            actor: Some("human".to_owned()),
        }
    }

    /// The id recorded on every change this caller makes.
    #[must_use]
    pub fn actor_of(&self) -> String {
        if let Some(actor) = &self.actor {
            return actor.clone();
        }
        match (self.role, &self.thread) {
            (Role::Human, _) => "human".to_owned(),
            (_, Some(thread)) => thread.clone(),
            _ => "unknown".to_owned(),
        }
    }

    /// The thread this caller acts as.
    pub(crate) fn self_id(&self) -> Option<&str> {
        self.thread.as_deref()
    }

    /// Whether this is the owner.
    pub(crate) fn is_human(&self) -> bool {
        self.role == Role::Human
    }

    /// How the owner or a thread is named in audit lines.
    pub(crate) fn label(&self) -> String {
        if self.is_human() {
            "owner".to_owned()
        } else {
            self.actor_of()
        }
    }
}
