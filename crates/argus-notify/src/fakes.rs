//! In-memory stand-ins for the ports, for the tests of this crate and of the server.

use std::collections::HashMap;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::{Mutex, MutexGuard, PoisonError};

use argus_core::StorageError;

use crate::config::NotifyConfig;
use crate::ports::{
    ActivityLine, ActivitySink, ConfigStore, NotifyStore, PageEvents, ProjectNames,
};
use crate::record::{Filter, KEEP_ROWS, NewNotification, Notification};

fn locked<T>(mutex: &Mutex<T>) -> MutexGuard<'_, T> {
    mutex.lock().unwrap_or_else(PoisonError::into_inner)
}

#[derive(Debug, Default)]
struct Rows {
    kept: Vec<Notification>,
    last_n: u64,
    seen: Option<String>,
}

/// A store that keeps the newest `KEEP_ROWS` notifications in memory, like the table does.
#[derive(Debug, Default)]
pub struct MemoryStore {
    rows: Mutex<Rows>,
    broken: AtomicBool,
    mirrors: AtomicUsize,
}

impl MemoryStore {
    /// An empty store.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Makes every call fail, like a closed database.
    pub fn set_broken(&self, broken: bool) {
        self.broken.store(broken, Ordering::SeqCst);
    }

    /// The stored notifications, oldest first.
    #[must_use]
    pub fn rows(&self) -> Vec<Notification> {
        locked(&self.rows).kept.clone()
    }

    /// How many times the docs were written again.
    #[must_use]
    pub fn mirrors(&self) -> usize {
        self.mirrors.load(Ordering::SeqCst)
    }

    fn check(&self) -> Result<(), StorageError> {
        if self.broken.load(Ordering::SeqCst) {
            return Err(StorageError("the database is closed".to_owned()));
        }
        Ok(())
    }
}

impl NotifyStore for MemoryStore {
    fn insert(&self, new: NewNotification) -> Result<Notification, StorageError> {
        self.check()?;
        let mut rows = locked(&self.rows);
        rows.last_n += 1;
        let note = Notification::from_new(rows.last_n, new);
        rows.kept.push(note.clone());
        let extra = rows.kept.len().saturating_sub(KEEP_ROWS);
        rows.kept.drain(..extra);
        Ok(note)
    }

    fn list(&self, filter: &Filter) -> Result<Vec<Notification>, StorageError> {
        self.check()?;
        let rows = locked(&self.rows);
        Ok(rows
            .kept
            .iter()
            .rev()
            .filter(|note| filter.level.is_none_or(|level| note.level == level))
            .filter(|note| {
                filter
                    .project
                    .as_ref()
                    .is_none_or(|project| note.project.as_ref() == Some(project))
            })
            .take(filter.limit)
            .cloned()
            .collect())
    }

    fn mirror(&self) -> Result<(), StorageError> {
        self.check()?;
        self.mirrors.fetch_add(1, Ordering::SeqCst);
        Ok(())
    }

    fn seen_load(&self) -> Option<String> {
        locked(&self.rows).seen.clone()
    }

    fn seen_save(&self, json: &str) -> Result<(), StorageError> {
        self.check()?;
        locked(&self.rows).seen = Some(json.to_owned());
        Ok(())
    }
}

/// A config held in memory that counts its saves and can refuse them.
#[derive(Debug, Default)]
pub struct MemoryConfig {
    config: Mutex<NotifyConfig>,
    saves: AtomicUsize,
    refuse: AtomicBool,
}

impl MemoryConfig {
    /// A config with these values.
    #[must_use]
    pub fn new(config: NotifyConfig) -> Self {
        Self {
            config: Mutex::new(config),
            ..Self::default()
        }
    }

    /// The config now.
    #[must_use]
    pub fn get(&self) -> NotifyConfig {
        locked(&self.config).clone()
    }

    /// Replaces the config, as an edit of `config.json` would.
    pub fn set(&self, config: NotifyConfig) {
        *locked(&self.config) = config;
    }

    /// Makes every save fail, like a full disk.
    pub fn refuse_saves(&self, refuse: bool) {
        self.refuse.store(refuse, Ordering::SeqCst);
    }

    /// How many saves went through.
    #[must_use]
    pub fn saves(&self) -> usize {
        self.saves.load(Ordering::SeqCst)
    }
}

impl ConfigStore for MemoryConfig {
    fn load(&self) -> NotifyConfig {
        self.get()
    }

    fn save(&self, config: &NotifyConfig) -> Result<(), StorageError> {
        if self.refuse.load(Ordering::SeqCst) {
            return Err(StorageError("disk full".to_owned()));
        }
        self.set(config.clone());
        self.saves.fetch_add(1, Ordering::SeqCst);
        Ok(())
    }
}

/// Project names from a fixed list.
#[derive(Debug, Default)]
pub struct StaticNames(HashMap<String, String>);

impl StaticNames {
    /// The names of `(key, name)` pairs.
    #[must_use]
    pub fn of(pairs: &[(&str, &str)]) -> Self {
        Self(
            pairs
                .iter()
                .map(|(key, name)| ((*key).to_owned(), (*name).to_owned()))
                .collect(),
        )
    }
}

impl ProjectNames for StaticNames {
    fn name(&self, key: &str) -> Option<String> {
        self.0.get(key).cloned()
    }
}

/// An activity log that keeps its lines.
#[derive(Debug, Default)]
pub struct RecordingActivity(Mutex<Vec<ActivityLine>>);

impl RecordingActivity {
    /// The lines added so far, oldest first.
    #[must_use]
    pub fn lines(&self) -> Vec<ActivityLine> {
        locked(&self.0).clone()
    }
}

impl ActivitySink for RecordingActivity {
    fn note(&self, line: ActivityLine) {
        locked(&self.0).push(line);
    }
}

/// A page fan-out that keeps the notifications it was told about.
#[derive(Debug, Default)]
pub struct RecordingEvents(Mutex<Vec<Notification>>);

impl RecordingEvents {
    /// The notifications pushed so far, oldest first.
    #[must_use]
    pub fn pushed(&self) -> Vec<Notification> {
        locked(&self.0).clone()
    }
}

impl PageEvents for RecordingEvents {
    fn recorded(&self, notification: &Notification) {
        locked(&self.0).push(notification.clone());
    }
}
