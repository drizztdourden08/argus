//! When each periodic job of the service runs next.

use crate::config::WakeConfig;

const PRUNE_EVERY_MS: i64 = 3_600_000;

/// One periodic job: when it runs next, and how often.
pub(super) struct Timer {
    next: i64,
    every: i64,
}

impl Timer {
    fn new(now: i64, every: i64) -> Self {
        Self {
            next: now + every,
            every,
        }
    }

    /// Is the job due at `now`? When it is, the next run is one period from now.
    pub(super) fn due(&mut self, now: i64) -> bool {
        if now < self.next {
            return false;
        }
        self.next = now + self.every;
        true
    }
}

/// The timers of every periodic job.
pub(super) struct Schedule {
    pub(super) registry: Timer,
    pub(super) agents: Timer,
    pub(super) items: Timer,
    pub(super) syncs: Timer,
    pub(super) deps: Timer,
    pub(super) stalls: Timer,
    pub(super) prune: Timer,
}

impl Schedule {
    pub(super) fn starting(now: i64, config: &WakeConfig) -> Self {
        Self {
            registry: Timer::new(now, config.registry.tick_ms),
            agents: Timer::new(now, config.agents.tick_ms),
            items: Timer::new(now, config.items.tick_ms),
            syncs: Timer::new(now, config.syncs.tick_ms),
            deps: Timer::new(now, config.deps.tick_ms),
            stalls: Timer::new(now, config.stall.tick_ms),
            prune: Timer::new(now, PRUNE_EVERY_MS),
        }
    }

    pub(super) fn earliest(&self) -> i64 {
        [
            self.registry.next,
            self.agents.next,
            self.items.next,
            self.syncs.next,
            self.deps.next,
            self.stalls.next,
            self.prune.next,
        ]
        .into_iter()
        .min()
        .unwrap_or(i64::MAX)
    }
}
