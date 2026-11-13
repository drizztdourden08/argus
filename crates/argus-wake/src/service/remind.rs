//! The reminders of what stays open: the Overlord's coalesced line, and the owner's notice after an hour.

use std::sync::Arc;

use super::WakeService;
use crate::faults::Fault;
use crate::reminders::{NoticeSource, OpenNotice, OwnerSink};
use crate::wake::WakeInput;

/// The reason of the line the Overlord hears every 15 minutes.
const RESEND_REASON: &str = "open-resend";

impl WakeService {
    /// Where an hour-old fault is told to the owner.
    pub fn set_owner_sink(&mut self, sink: Arc<dyn OwnerSink>) {
        self.owner = Some(sink);
    }

    /// One more place open notices come from (the server plugs in `worktree-stale`).
    pub fn add_notice_source(&mut self, source: Arc<dyn NoticeSource>) {
        self.notice_sources.push(source);
    }

    /// The reminders due now: the owner hears of each fault open for an hour, the Overlord hears one
    /// coalesced line every 15 minutes while anything stays open, an unacknowledged stall among it.
    /// Returns how many wakes were sent.
    pub fn reminder_tick(&mut self) -> usize {
        let now = self.clock.now_ms();
        let faults = self.registry.faults(false);
        self.tell_owner(now, &faults);
        let theirs: Vec<&Fault> = faults
            .iter()
            .filter(|f| !self.registry.is_main(&f.project, &f.thread))
            .collect();
        let notices: Vec<OpenNotice> = self
            .notice_sources
            .iter()
            .flat_map(|source| source.open(now))
            .chain(self.stalls.open_notices())
            .collect();
        let Some(line) = self.resend.due(now, &theirs, &notices) else {
            return 0;
        };
        let wake = WakeInput::about(
            &self.boards.overlord(),
            &self.main_thread(),
            RESEND_REASON,
            "open",
            line,
        );
        self.submit_logged(&[wake], "open resend");
        1
    }

    fn tell_owner(&mut self, now: i64, faults: &[Fault]) {
        let Some(sink) = self.owner.clone() else {
            return;
        };
        for notice in self.owner_alerts.due(now, faults) {
            sink.notify(notice);
        }
    }
}
