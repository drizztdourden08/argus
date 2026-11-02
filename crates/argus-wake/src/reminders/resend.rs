//! The coalesced line, sent again every `resend_ms` while anything stays open.

use super::{OpenNotice, open_line};
use crate::faults::Fault;

/// When the Overlord last heard the line.
pub struct Resend {
    every_ms: i64,
    last: Option<i64>,
}

impl Resend {
    /// A resend every `every_ms`.
    #[must_use]
    pub fn new(every_ms: i64) -> Self {
        Self {
            every_ms,
            last: None,
        }
    }

    /// The line due now, or `None`. The first sight of something open starts the clock (a fault itself
    /// woke the Overlord at once), and nothing open stops it.
    pub fn due(&mut self, now: i64, faults: &[&Fault], notices: &[OpenNotice]) -> Option<String> {
        let Some(line) = open_line(now, faults, notices) else {
            self.last = None;
            return None;
        };
        let last = *self.last.get_or_insert(now);
        if now - last < self.every_ms {
            return None;
        }
        self.last = Some(now);
        Some(line)
    }
}
