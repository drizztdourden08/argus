//! A listener the tests read from.

use tokio::sync::mpsc::UnboundedReceiver;

use crate::engine::{Attached, ListenerMessage};
use crate::wake::WakeEvent;

/// An attached listener and everything it has received.
pub struct Tap {
    /// The handle to detach it.
    pub attached: Attached,
    rx: UnboundedReceiver<ListenerMessage>,
    events: Vec<WakeEvent>,
    replaced: usize,
}

impl Tap {
    pub(super) fn new(attached: Attached, rx: UnboundedReceiver<ListenerMessage>) -> Self {
        Self {
            attached,
            rx,
            events: Vec::new(),
            replaced: 0,
        }
    }

    fn drain(&mut self) {
        while let Ok(message) = self.rx.try_recv() {
            match message {
                ListenerMessage::Wake(event) => self.events.push(event),
                ListenerMessage::Replaced => self.replaced += 1,
            }
        }
    }

    /// Every event received so far.
    pub fn events(&mut self) -> &[WakeEvent] {
        self.drain();
        &self.events
    }

    /// How many events were received so far.
    pub fn count(&mut self) -> usize {
        self.events().len()
    }

    /// How many `Replaced` messages arrived.
    pub fn replaced(&mut self) -> usize {
        self.drain();
        self.replaced
    }

    /// The lines of every event received so far.
    pub fn lines(&mut self) -> Vec<Vec<String>> {
        self.events().iter().map(|e| e.lines.clone()).collect()
    }
}
