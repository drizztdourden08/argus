//! What the engine tells the registry: listeners coming and going, windows closing, events replayed.

/// An event a late listener took.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReplayedEvent {
    /// The event (first wake) id.
    pub id: String,
    /// Its thread.
    pub thread: String,
    /// When it was made (ISO).
    pub at: String,
    /// Why it was made.
    pub reason: String,
    /// Its text.
    pub text: String,
}

/// One thing that happened in the engine.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EngineNote {
    /// A listener connected.
    Attached {
        /// Project key.
        scope: String,
    },
    /// A listener went away.
    Detached {
        /// Project key.
        scope: String,
    },
    /// A window closed.
    Flushed {
        /// Project key.
        scope: String,
        /// Target thread.
        thread: String,
        /// The event (lead wake) id.
        id: String,
        /// When the lead wake was made (ISO).
        at: String,
        /// The lead wake's reason.
        reason: String,
        /// The lead wake's text.
        text: String,
        /// How many listeners took the event.
        sent: u32,
    },
    /// Queued events went to a late listener.
    Replayed {
        /// Project key.
        scope: String,
        /// The events it took.
        events: Vec<ReplayedEvent>,
    },
    /// Writing to a listener threw.
    DeliveryError {
        /// Project key.
        scope: String,
        /// Target thread.
        thread: String,
        /// The event id.
        id: String,
        /// What the write said.
        err: String,
    },
}
