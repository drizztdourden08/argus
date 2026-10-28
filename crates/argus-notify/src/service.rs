//! The notification service: its parts, its start and stop, and the owner's settings.

use std::sync::atomic::{AtomicBool, AtomicU16, Ordering};
use std::sync::{Mutex, MutexGuard, PoisonError};

use crate::configure::{SettingsPatch, apply, describe};
use crate::limits::Limiter;
use crate::links::Links;
use crate::ports::{ActivityLevel, ActivityLine, Ports};
use crate::record::{Filter, KEEP_ROWS, Notification};
use crate::seen::SeenMap;
use crate::settings::{Settings, settings_of};
use argus_core::StorageError;
use argus_core::names::Door;

pub(crate) fn locked<T>(mutex: &Mutex<T>) -> MutexGuard<'_, T> {
    mutex.lock().unwrap_or_else(PoisonError::into_inner)
}

/// Notifications to the owner: records them, shows their toasts and pings by itself when a board
/// says the owner is needed. It never fails its callers.
pub struct Notify {
    pub(crate) ports: Ports,
    port: AtomicU16,
    door: Mutex<Option<Door>>,
    pub(crate) limiter: Mutex<Limiter>,
    pub(crate) seen: Mutex<Option<SeenMap>>,
    running: AtomicBool,
}

impl Notify {
    /// A stopped service; `port` is the dashboard's, for the addresses a toast opens.
    #[must_use]
    pub fn new(ports: Ports, port: u16) -> Self {
        Self {
            ports,
            port: AtomicU16::new(port),
            door: Mutex::new(None),
            limiter: Mutex::new(Limiter::default()),
            seen: Mutex::new(None),
            running: AtomicBool::new(false),
        }
    }

    /// Tells the service the port the dashboard really listens on, for the addresses a toast
    /// opens; the server knows it only once the listener is bound.
    pub fn set_port(&self, port: u16) {
        self.port.store(port, Ordering::SeqCst);
    }

    /// Tells the service the front door of the local names, once they answer, so the addresses a
    /// toast opens use them; `None` returns to `.localhost:<port>`.
    pub fn set_door(&self, door: Option<Door>) {
        *locked(&self.door) = door;
    }

    pub(crate) fn links(&self) -> Links {
        Links::new(self.port.load(Ordering::SeqCst), locked(&self.door).clone())
    }

    /// Starts the service and writes the docs of the stored notifications again.
    pub fn start(&self) {
        if !self.running.swap(true, Ordering::SeqCst) {
            self.mirror();
        }
    }

    /// Stops the automatic pings; `send` keeps working.
    pub fn stop(&self) {
        self.running.store(false, Ordering::SeqCst);
    }

    /// True between `start` and `stop`.
    #[must_use]
    pub fn is_running(&self) -> bool {
        self.running.load(Ordering::SeqCst)
    }

    /// Writes the docs again, as after a hard resync of the portfolio scope.
    pub fn mirror(&self) {
        if self.ports.store.mirror().is_err() {
            self.warn("notification mirror failed");
        }
    }

    /// The stored notifications that match, newest first, at most `KEEP_ROWS`.
    ///
    /// # Errors
    ///
    /// When the database cannot be read.
    pub fn list(&self, filter: &Filter) -> Result<Vec<Notification>, StorageError> {
        let clamped = Filter {
            limit: filter.limit.clamp(1, KEEP_ROWS),
            ..filter.clone()
        };
        self.ports.store.list(&clamped)
    }

    /// The settings in effect.
    #[must_use]
    pub fn settings(&self) -> Settings {
        settings_of(&self.ports.config.load())
    }

    /// Changes the toast and the sound of some levels, writes the config and answers the settings
    /// now in effect.
    ///
    /// # Errors
    ///
    /// When `config.json` cannot be written; nothing changed then.
    pub fn configure(&self, patch: SettingsPatch) -> Result<Settings, StorageError> {
        let next = apply(&self.ports.config.load(), patch);
        self.ports.config.save(&next)?;
        let settings = settings_of(&next);
        self.activity(ActivityLevel::Info, describe(settings));
        Ok(settings)
    }

    pub(crate) fn warn(&self, text: &str) {
        self.activity(ActivityLevel::Warn, text.to_owned());
    }

    fn activity(&self, level: ActivityLevel, text: String) {
        self.ports.activity.note(ActivityLine {
            level,
            text,
            project: None,
            thread: None,
        });
    }

    pub(crate) fn name_of(&self, key: &str) -> String {
        self.ports.names.name(key).unwrap_or_else(|| key.to_owned())
    }
}
