//! The timers of the service, as deadlines: nothing sleeps here, the driver sleeps until the next one.

use tracing::warn;

use super::WakeService;
use crate::activity::ActivityLine;
use crate::dep_wakes::DepEnv;
use crate::item_wakes::ItemEnv;
use crate::registry::Registry;
use crate::store::WakeStore;
use crate::sync_wakes::SyncEnv;

/// When the wakes table last sent a reminder, as the wake kinds read it; a table that cannot be read says never and
/// leaves one warning that names `what`.
pub(super) fn last_sent_of<'a>(
    store: &'a dyn WakeStore,
    what: &'static str,
) -> impl Fn(&str, &str, &str, &str) -> i64 + 'a {
    move |scope: &str, thread: &str, reason: &str, subject: &str| {
        store
            .last_at(scope, thread, reason, Some(subject))
            .unwrap_or_else(|err| {
                warn!(%err, "{what}: last wake not read");
                0
            })
    }
}

impl WakeService {
    /// The earliest moment `tick` has work: a window closing or a periodic job.
    #[must_use]
    pub fn next_due_ms(&self) -> i64 {
        self.engine
            .next_deadline()
            .map_or(self.schedule.earliest(), |d| {
                d.min(self.schedule.earliest())
            })
    }

    /// Milliseconds until `next_due_ms`, never negative.
    #[must_use]
    pub fn until_next_ms(&self) -> i64 {
        (self.next_due_ms() - self.clock.now_ms()).max(0)
    }

    /// Run everything that is due: windows, the registry check, the reminders, the prune.
    pub fn tick(&mut self) {
        self.engine.tick();
        self.pump();
        let now = self.clock.now_ms();
        if self.schedule.registry.due(now) {
            self.registry_tick();
            self.reminder_tick();
            self.relay_tick();
        }
        if self.schedule.agents.due(now) {
            self.agent_tick();
        }
        if self.schedule.items.due(now) {
            self.item_tick();
        }
        if self.schedule.syncs.due(now) {
            self.sync_tick();
        }
        if self.schedule.deps.due(now) {
            self.dep_tick();
        }
        if self.schedule.stalls.due(now) {
            self.stall_tick();
        }
        if self.schedule.prune.due(now) {
            if let Err(err) = self.engine.prune() {
                warn!(%err, "wake prune failed");
            }
        }
    }

    /// Check the timeouts of the pending wakes and the boards, and write the registry docs.
    pub fn registry_tick(&mut self) {
        self.with_env(Registry::tick);
    }

    /// The agent reminders due now; returns how many wakes were sent.
    pub fn agent_tick(&mut self) -> usize {
        let wakes = {
            let last_sent = last_sent_of(self.engine.store(), "agent wakes");
            self.agents.tick(
                self.clock.now_ms(),
                self.boards.as_ref(),
                self.live.as_ref(),
                &last_sent,
            )
        };
        self.submit_logged(&wakes, "agent wakes");
        wakes.len()
    }

    /// The Kanban reminders due now; returns how many wakes were sent.
    pub fn item_tick(&mut self) -> usize {
        let now = self.clock.now_ms();
        let main = Some(self.main_thread());
        let out = {
            let last_sent = last_sent_of(self.engine.store(), "item wakes");
            let env = ItemEnv {
                boards: self.boards.as_ref(),
                main,
                last_sent: &last_sent,
            };
            self.items.tick(now, &env)
        };
        self.forward_activity(out.activity);
        self.tell_owner_of(out.owner);
        self.submit_logged(&out.wakes, "item wakes");
        out.wakes.len() + self.watch_quiet()
    }

    /// The dependency wakes due now; returns how many were sent. The server may call it right after it
    /// reconciled a dependency, so the other side hears at once instead of at the next tick.
    pub fn dep_tick(&mut self) -> usize {
        let now = self.clock.now_ms();
        let wakes = {
            let last_sent = last_sent_of(self.engine.store(), "dependency wakes");
            let env = DepEnv {
                boards: self.boards.as_ref(),
                last_sent: &last_sent,
            };
            self.dep_wakes.tick(now, &env)
        };
        self.submit_logged(&wakes, "dependency wakes");
        wakes.len()
    }

    pub(super) fn forward_activity(&self, lines: Vec<ActivityLine>) {
        if let Some(sink) = &self.activity {
            for line in lines {
                sink.add(line);
            }
        }
    }

    /// The remote sync wakes and activity lines due now; returns how many wakes were sent.
    pub fn sync_tick(&mut self) -> usize {
        let now = self.clock.now_ms();
        let out = {
            let last_sent = last_sent_of(self.engine.store(), "sync wakes");
            let env = SyncEnv {
                boards: self.boards.as_ref(),
                syncs: self.syncs.as_ref(),
                last_sent: &last_sent,
            };
            self.sync_wakes.tick(now, &env)
        };
        self.forward_activity(out.activity);
        self.submit_logged(&out.wakes, "sync wakes");
        out.wakes.len()
    }
}
