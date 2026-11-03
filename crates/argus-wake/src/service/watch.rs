//! The item events of the exported boards: each export is compared with the one before, and the quiet
//! to-dos are looked for with the Kanban reminders.

use super::WakeService;
use super::periodic::last_sent_of;
use crate::item_watch::WatchEnv;

impl WakeService {
    /// A project board was exported again: the wakes, the owner's notices and the activity lines for what
    /// changed on its items since the last export. Returns how many wakes were sent.
    pub fn watch_board(&mut self, scope: &str) -> usize {
        let now = self.clock.now_ms();
        let out = {
            let last_sent = last_sent_of(self.engine.store(), "item events");
            let env = WatchEnv {
                boards: self.boards.as_ref(),
                last_sent: &last_sent,
                now,
            };
            self.watch.on_board(scope, &env)
        };
        self.forward_activity(out.activity);
        if let Some(sink) = &self.owner {
            for notice in out.owner {
                sink.notify(notice);
            }
        }
        self.submit_logged(&out.wakes, "item events");
        out.wakes.len()
    }

    /// The to-dos of other leads that hold an item back and went quiet; returns how many wakes were sent.
    pub(super) fn watch_quiet(&mut self) -> usize {
        let now = self.clock.now_ms();
        let wakes = {
            let last_sent = last_sent_of(self.engine.store(), "quiet to-dos");
            let env = WatchEnv {
                boards: self.boards.as_ref(),
                last_sent: &last_sent,
                now,
            };
            self.watch.quiet(&env)
        };
        self.submit_logged(&wakes, "quiet to-dos");
        wakes.len()
    }
}
