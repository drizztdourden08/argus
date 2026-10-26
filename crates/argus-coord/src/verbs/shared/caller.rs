//! The caller of a verb as the board ops see it.

use argus_board::ops::{Role, Who};

use crate::who::Caller;

/// The caller as the `who` of `apply_op` and `authorize`.
#[must_use]
pub fn who_of(c: &Caller) -> Who {
    Who {
        thread: Some(c.thread.clone()),
        role: c.role,
        actor: Some(c.actor.clone()),
    }
}

/// Whether the caller leads the project.
#[must_use]
pub fn is_coordinator(c: &Caller) -> bool {
    c.role == Role::Coordinator
}
