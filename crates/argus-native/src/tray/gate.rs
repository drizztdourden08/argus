//! Lets the page open once for a double click: the two clicks of a double click, and a click
//! right after the menu item, ask for the page twice within moments.
// test-tag: critical

use std::time::{Duration, Instant};

/// How long after one open another is ignored.
pub const OPEN_PAUSE: Duration = Duration::from_secs(2);

/// Remembers when the page was last asked for.
#[derive(Debug, Default)]
pub struct OpenGate {
    last: Option<Instant>,
}

impl OpenGate {
    /// True when the page should open now, and records it.
    pub fn allow(&mut self, now: Instant) -> bool {
        let open = self
            .last
            .is_none_or(|last| now.saturating_duration_since(last) >= OPEN_PAUSE);
        if open {
            self.last = Some(now);
        }
        open
    }
}

#[cfg(test)]
mod tests {
    use std::time::{Duration, Instant};

    use super::OpenGate;

    #[test]
    fn a_second_open_within_two_seconds_is_ignored_and_a_later_one_goes_through() {
        let mut gate = OpenGate::default();
        let start = Instant::now();
        assert!(gate.allow(start));
        assert!(!gate.allow(start + Duration::from_millis(300)));
        assert!(!gate.allow(start + Duration::from_millis(1999)));
        assert!(gate.allow(start + Duration::from_secs(2)));
    }
}
