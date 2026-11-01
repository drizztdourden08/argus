//! The timer switches the options carry, and the background tasks the services keep, by the
//! name of the service that started them.

use std::collections::HashMap;
use std::sync::{Mutex, PoisonError};

use tokio::task::JoinHandle;

use crate::ServeOptions;

/// A background timer the options can switch off.
#[derive(Clone, Copy, PartialEq, Eq)]
pub(crate) enum Timer {
    /// The launcher wakes.
    Stale,
    /// The worktree scanner.
    Worktrees,
    /// The wake engine.
    Wake,
    /// The build cache prune.
    Cache,
}

/// Which timers run, and their tasks.
pub(crate) struct Tasks {
    timers: Vec<Timer>,
    handles: Mutex<HashMap<&'static str, Vec<JoinHandle<()>>>>,
}

impl Tasks {
    pub(crate) fn new(opts: &ServeOptions) -> Self {
        let switches = [
            (Timer::Stale, opts.run_service.stale_timer),
            (Timer::Worktrees, opts.worktrees.timers),
            (Timer::Wake, opts.wake.timer),
            (Timer::Cache, opts.cache.timers),
        ];
        Tasks {
            timers: switches
                .into_iter()
                .filter(|(_, on)| *on)
                .map(|(t, _)| t)
                .collect(),
            handles: Mutex::default(),
        }
    }

    /// Whether `timer` runs.
    pub(crate) fn runs(&self, timer: Timer) -> bool {
        self.timers.contains(&timer)
    }

    /// Keeps a task of `service` to end it with [`Tasks::abort`].
    pub(crate) fn keep(&self, service: &'static str, task: JoinHandle<()>) {
        self.handles
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .entry(service)
            .or_default()
            .push(task);
    }

    /// Ends every task of `service`.
    pub(crate) fn abort(&self, service: &'static str) {
        let tasks = self
            .handles
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .remove(service)
            .unwrap_or_default();
        for task in tasks {
            task.abort();
        }
    }
}
