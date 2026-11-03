//! Where a run tail writes. The tail core decides what to say (deduplicated lines, the run's
//! status, the end); a sink decides how it is written and what ending means.

use argus_launchers::{LogEvent, RunRow};
use serde_json::json;

use crate::sse::SseSender;

/// The output side of a run tail. Calls arrive in order and `line` is called with the tail's
/// cursor lock held, so an implementation must not call back into the tail.
pub(crate) trait TailSink: Send + Sync {
    /// One log line, never repeated and never out of order.
    fn line(&self, event: &LogEvent);
    /// The run's current record. Sent after replay and on every change of the run.
    fn status(&self, run: &RunRow);
    /// The tail is over: the final status was sent, or the run is gone. The core drops its
    /// listeners itself right after.
    fn end(&self);
}

/// The legacy run tail route: `log` and `status` events on a stream of its own, closed at the end.
pub(crate) struct LegacySink {
    sender: SseSender,
}

impl LegacySink {
    pub(crate) fn new(sender: SseSender) -> Self {
        Self { sender }
    }
}

impl TailSink for LegacySink {
    fn line(&self, event: &LogEvent) {
        let data =
            json!({ "seq": event.seq, "stream": event.stream, "at": event.at, "text": event.text });
        let _ = self
            .sender
            .event("log", &data, i64::try_from(event.seq).ok());
    }

    fn status(&self, run: &RunRow) {
        let _ = self.sender.event("status", &json!({ "run": run }), None);
    }

    fn end(&self) {
        self.sender.close();
    }
}
