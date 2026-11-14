//! The listener of the hosted Overlord: a [`ListenerSink`] that hands each wake to the child's inbox
//! as the same text the hook listener prints, instead of writing it to a stream.
//!
//! The sink knows nothing of the child. It is given a `push` function that returns whether the
//! child took the text; `false` means the child is gone, and the engine then keeps the wake pending
//! and replays it when the next child attaches.

use crate::engine::{Delivery, ListenerSink};
use crate::line::wake_line;
use crate::wake::WakeEvent;

/// What the sink calls with a wake line: true when the child's inbox took it.
pub type Push = Box<dyn Fn(String) -> bool + Send>;

/// A listener whose events become user messages of the hosted child.
pub struct HostedSink {
    push: Push,
}

impl HostedSink {
    /// A sink that gives every wake line to `push`.
    #[must_use]
    pub fn new(push: Push) -> Self {
        Self { push }
    }
}

impl ListenerSink for HostedSink {
    fn deliver(&mut self, event: &WakeEvent) -> Delivery {
        let data = match serde_json::to_string(event) {
            Ok(data) => data,
            Err(err) => return Delivery::Failed(err.to_string()),
        };
        if (self.push)(wake_line(&data, &event.thread)) {
            Delivery::Sent
        } else {
            Delivery::Gone
        }
    }

    fn replaced(&mut self) {}
}
