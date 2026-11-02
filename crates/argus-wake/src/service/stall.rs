//! The stall watch inside the service: the tick, the inbox reads and the acks.

use std::sync::Arc;

use super::WakeService;
use crate::reminders::OwnerNotice;
use crate::stalls::{BusySource, StallEnv, StallInfo, StallKv};

impl WakeService {
    /// Plug in the running processes of the projects and the kv of the inbox reads and the acks.
    pub fn set_stall_sources(&mut self, busy: Arc<dyn BusySource>, kv: Arc<dyn StallKv>) {
        self.stalls.set_sources(busy, kv);
    }

    /// The stall watch due now: wakes a thread that did not read, tells the Overlord of every open stall, tells the
    /// owner, writes the page documents. Returns how many wakes were sent.
    pub fn stall_tick(&mut self) -> usize {
        let now = self.clock.now_ms();
        let main = self.main_thread();
        let out = {
            let env = StallEnv {
                boards: self.boards.as_ref(),
                store: self.engine.store(),
                main: &main,
            };
            self.stalls.tick(now, &env)
        };
        self.forward_activity(out.activity);
        self.tell_owner_of(out.owner);
        self.submit_logged(&out.wakes, "stall wakes");
        out.wakes.len()
    }

    /// A thread read its own inbox: the wakes it was sent before are acknowledged. The route has checked that the
    /// thread exists.
    pub fn inbox_read(&self, scope: &str, thread: &str) {
        self.stalls.inbox_read(scope, thread, self.clock.now_ms());
    }

    /// The open stalls, of one project or of all.
    #[must_use]
    pub fn stalls(&self, project: Option<&str>) -> Vec<StallInfo> {
        self.stalls.open(project)
    }

    /// The Overlord acknowledges every open stall of a project; the route has checked who asks. Returns the stalls
    /// acknowledged, none when the project has no open stall.
    pub fn ack_stalls(&mut self, project: &str, by: &str, note: &str) -> Vec<StallInfo> {
        self.stalls.ack(project, by, note, self.clock.now_ms())
    }

    fn tell_owner_of(&self, notices: Vec<OwnerNotice>) {
        if let Some(sink) = &self.owner {
            for notice in notices {
                sink.notify(notice);
            }
        }
    }
}
