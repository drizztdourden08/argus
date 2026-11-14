//! The messages waiting for the child: kept while a turn runs or the child is down, and written as
//! ONE user message (the texts joined by a blank line) when the child can take it.

/// What separates two held messages in the combined one.
const SEPARATOR: &str = "\n\n";

/// The pending messages, oldest first.
#[derive(Debug, Default)]
pub struct Inbox {
    held: Vec<String>,
}

impl Inbox {
    /// Holds a message after the others.
    pub fn push(&mut self, text: String) {
        self.held.push(text);
    }

    /// Puts a combined message back in front of what came since, after a failed write or a crash.
    pub fn requeue(&mut self, text: String) {
        self.held.insert(0, text);
    }

    /// Everything held as one message, or `None` when nothing is held. The inbox is empty after.
    pub fn take_combined(&mut self) -> Option<String> {
        if self.held.is_empty() {
            return None;
        }
        let held = std::mem::take(&mut self.held);
        Some(held.join(SEPARATOR))
    }
}
