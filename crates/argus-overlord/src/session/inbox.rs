//! The messages waiting for the child: kept while a turn runs or the child is down, and written as
//! ONE user message (the texts joined by a blank line) when the child can take it.
//!
//! The held messages sit behind a shared cell so the owner's Queue view can read them through a
//! [`HeldView`] while the supervisor owns the inbox.

use std::sync::{Arc, Mutex, MutexGuard, PoisonError};

use argus_core::clock::now_iso;
use serde::Serialize;

/// What separates two held messages in the combined one.
const SEPARATOR: &str = "\n\n";

/// One held message and when it was queued (ISO).
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Held {
    pub text: String,
    pub at: String,
}

type Cell = Arc<Mutex<Vec<Held>>>;

fn open(cell: &Cell) -> MutexGuard<'_, Vec<Held>> {
    cell.lock().unwrap_or_else(PoisonError::into_inner)
}

/// The pending messages, oldest first.
#[derive(Debug, Default)]
pub struct Inbox {
    held: Cell,
}

/// A read-only look at an inbox, cheap to clone and to keep.
#[derive(Debug, Clone)]
pub struct HeldView {
    held: Cell,
}

impl HeldView {
    /// The held messages, oldest first.
    #[must_use]
    pub fn snapshot(&self) -> Vec<Held> {
        open(&self.held).clone()
    }
}

impl Inbox {
    /// A view that follows this inbox.
    #[must_use]
    pub fn view(&self) -> HeldView {
        HeldView {
            held: Arc::clone(&self.held),
        }
    }

    /// Holds a message after the others.
    pub fn push(&mut self, text: String) {
        open(&self.held).push(Held {
            text,
            at: now_iso(),
        });
    }

    /// Puts a combined message back in front of what came since, after a failed write or a crash.
    pub fn requeue(&mut self, text: String) {
        let held = Held {
            text,
            at: now_iso(),
        };
        open(&self.held).insert(0, held);
    }

    /// Everything held as one message, or `None` when nothing is held. The inbox is empty after.
    pub fn take_combined(&mut self) -> Option<String> {
        let held = std::mem::take(&mut *open(&self.held));
        if held.is_empty() {
            return None;
        }
        let texts: Vec<String> = held.into_iter().map(|h| h.text).collect();
        Some(texts.join(SEPARATOR))
    }
}

// test-tag: critical
#[cfg(test)]
mod specs {
    use super::*;

    #[test]
    fn a_view_sees_the_held_messages_until_they_are_taken() {
        let mut inbox = Inbox::default();
        let view = inbox.view();
        assert_eq!(view.snapshot().len(), 0);
        inbox.push("one".to_owned());
        inbox.push("two".to_owned());
        let texts: Vec<String> = view.snapshot().into_iter().map(|h| h.text).collect();
        assert_eq!(texts, ["one", "two"]);
        assert_eq!(inbox.take_combined().as_deref(), Some("one\n\ntwo"));
        assert_eq!(view.snapshot().len(), 0);
    }

    #[test]
    fn a_requeued_message_goes_in_front() {
        let mut inbox = Inbox::default();
        inbox.push("later".to_owned());
        inbox.requeue("first".to_owned());
        assert_eq!(inbox.take_combined().as_deref(), Some("first\n\nlater"));
    }
}
