//! The activity log as every service crate sees it. [`ActivityAdapter`] is the one local type
//! that implements each crate's activity trait, one file per crate; a crate whose activity
//! port is part of a larger trait (the process monitor's host, the worktree notifier, the
//! launcher wakes) gets a method its own adapter calls.

use std::sync::Arc;

use argus_core::log::Logger;
use argus_store::{ActivityLevel, ActivityLog, LogNote};
use serde_json::json;

use crate::Services;

/// `level_of`, from a crate's three-level enum (`Info`, `Warn`, `Error`) to the log's.
macro_rules! level_of {
    ($level:ty) => {
        fn level_of(level: $level) -> argus_store::ActivityLevel {
            match level {
                <$level>::Info => argus_store::ActivityLevel::Info,
                <$level>::Warn => argus_store::ActivityLevel::Warn,
                <$level>::Error => argus_store::ActivityLevel::Error,
            }
        }
    };
}

mod agents;
mod launchers;
mod notify;
mod procs;
mod wake;
mod worktrees;

/// Writes the lines of the service crates to the activity log.
#[derive(Clone)]
pub struct ActivityAdapter {
    log: Arc<ActivityLog>,
    logger: Arc<Logger>,
}

impl ActivityAdapter {
    /// An adapter over the activity log of the services.
    #[must_use]
    pub fn of(svc: &Services) -> Self {
        ActivityAdapter {
            log: Arc::clone(&svc.activity),
            logger: Arc::clone(&svc.log),
        }
    }

    /// A warning of a service: to the server log, and to the activity log as `kind` (one line
    /// per message within a minute).
    pub fn warn(&self, kind: &str, message: &str, detail: &str) {
        self.logger.warn(message, json!({ "detail": detail }));
        self.log.note(&LogNote {
            kind,
            level: ActivityLevel::Warn,
            message,
            cause: Some(detail),
            project: None,
        });
    }
}
