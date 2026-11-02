//! What the stall watch reads that the wake crate cannot see: running processes, and a small key-value store.

use std::collections::HashMap;
use std::sync::{Arc, Mutex, MutexGuard, PoisonError};

/// Does a project have a running process or run that counts as work?
pub trait BusySource: Send + Sync {
    /// True while the project has work running.
    fn busy(&self, project: &str) -> bool;
}

/// A [`BusySource`] that never sees work.
#[derive(Debug, Clone, Copy, Default)]
pub struct NoBusy;

impl BusySource for NoBusy {
    fn busy(&self, _project: &str) -> bool {
        false
    }
}

/// Where the inbox reads and the acks are kept. The server plugs the database kv in.
pub trait StallKv: Send + Sync {
    /// The value under a key.
    fn get(&self, key: &str) -> Option<String>;

    /// Store a value under a key.
    fn set(&self, key: &str, value: &str);
}

/// A [`StallKv`] in memory. Clones share their data.
#[derive(Clone, Default)]
pub struct MemoryKv {
    inner: Arc<Mutex<HashMap<String, String>>>,
}

impl MemoryKv {
    /// An empty store.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    fn locked(&self) -> MutexGuard<'_, HashMap<String, String>> {
        self.inner.lock().unwrap_or_else(PoisonError::into_inner)
    }
}

impl StallKv for MemoryKv {
    fn get(&self, key: &str) -> Option<String> {
        self.locked().get(key).cloned()
    }

    fn set(&self, key: &str, value: &str) {
        self.locked().insert(key.to_owned(), value.to_owned());
    }
}
