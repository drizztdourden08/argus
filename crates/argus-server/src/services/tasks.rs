//! The timer switches the options carry, and the background tasks the services keep, by the
//! name of the service that started them.

use std::collections::HashMap;
use std::sync::{Mutex, PoisonError};

use tokio::task::JoinHandle;

use crate::ServeOptions;

/// Whether the timers of the launcher wakes, the worktree scanner and the build cache prune run, and their tasks.
pub(crate) struct Tasks {
    pub stale_timer: bool,
    pub worktree_timers: bool,
    pub cache_timer: bool,
    handles: Mutex<HashMap<&'static str, Vec<JoinHandle<()>>>>,
}

impl Tasks {
    pub(crate) fn new(opts: &ServeOptions) -> Self {
        Tasks {
            stale_timer: opts.run_service.stale_timer,
            worktree_timers: opts.worktrees.timers,
            cache_timer: opts.cache.timers,
            handles: Mutex::default(),
        }
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
