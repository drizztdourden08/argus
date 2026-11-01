//! The timers of the service, as deadlines: nothing sleeps here, the driver sleeps until the next one.

use tracing::warn;

use super::WakeService;
use crate::config::WakeConfig;
use crate::item_wakes::ItemEnv;
use crate::registry::Registry;
use crate::sync_wakes::SyncEnv;

const PRUNE_EVERY_MS: i64 = 3_600_000;

/// When each periodic job runs next.
pub(super) struct Schedule {
    registry: i64,
    agents: i64,
    items: i64,
    syncs: i64,
    prune: i64,
    registry_every: i64,
    agents_every: i64,
    items_every: i64,
    syncs_every: i64,
}

impl Schedule {
    pub(super) fn starting(now: i64, config: &WakeConfig) -> Self {
        Self {
            registry: now + config.registry.tick_ms,
            agents: now + config.agents.tick_ms,
            items: now + config.items.tick_ms,
            syncs: now + config.syncs.tick_ms,
            prune: now + PRUNE_EVERY_MS,
            registry_every: config.registry.tick_ms,
            agents_every: config.agents.tick_ms,
            items_every: config.items.tick_ms,
            syncs_every: config.syncs.tick_ms,
        }
    }

    fn earliest(&self) -> i64 {
        self.registry
            .min(self.agents)
            .min(self.items)
            .min(self.syncs)
            .min(self.prune)
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
        if now >= self.schedule.registry {
            self.schedule.registry = now + self.schedule.registry_every;
            self.registry_tick();
        }
        if now >= self.schedule.agents {
            self.schedule.agents = now + self.schedule.agents_every;
            self.agent_tick();
        }
        if now >= self.schedule.items {
            self.schedule.items = now + self.schedule.items_every;
            self.item_tick();
        }
        if now >= self.schedule.syncs {
            self.schedule.syncs = now + self.schedule.syncs_every;
            self.sync_tick();
        }
        if now >= self.schedule.prune {
            self.schedule.prune = now + PRUNE_EVERY_MS;
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
        let wakes = self.agents.tick(
            self.clock.now_ms(),
            self.boards.as_ref(),
            self.live.as_ref(),
        );
        self.submit_logged(&wakes, "agent wakes");
        wakes.len()
    }

    /// The Kanban reminders due now; returns how many wakes were sent.
    pub fn item_tick(&mut self) -> usize {
        let now = self.clock.now_ms();
        let main = Some(self.main_thread());
        let store = self.engine.store();
        let last_sent = |scope: &str, thread: &str, reason: &str, subject: &str| {
            store
                .last_at(scope, thread, reason, Some(subject))
                .unwrap_or_else(|err| {
                    warn!(%err, "item wakes: last wake not read");
                    0
                })
        };
        let env = ItemEnv {
            boards: self.boards.as_ref(),
            main,
            last_sent: &last_sent,
        };
        let out = self.items.tick(now, &env);
        if let Some(sink) = &self.activity {
            for line in out.activity {
                sink.add(line);
            }
        }
        self.submit_logged(&out.wakes, "item wakes");
        out.wakes.len()
    }

    /// The remote sync wakes and activity lines due now; returns how many wakes were sent.
    pub fn sync_tick(&mut self) -> usize {
        let now = self.clock.now_ms();
        let store = self.engine.store();
        let last_sent = |scope: &str, thread: &str, reason: &str, subject: &str| {
            store
                .last_at(scope, thread, reason, Some(subject))
                .unwrap_or_else(|err| {
                    warn!(%err, "sync wakes: last wake not read");
                    0
                })
        };
        let env = SyncEnv {
            boards: self.boards.as_ref(),
            syncs: self.syncs.as_ref(),
            last_sent: &last_sent,
        };
        let out = self.sync_wakes.tick(now, &env);
        if let Some(sink) = &self.activity {
            for line in out.activity {
                sink.add(line);
            }
        }
        self.submit_logged(&out.wakes, "sync wakes");
        out.wakes.len()
    }
}
