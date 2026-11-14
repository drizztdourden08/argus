//! The list of listeners the log and transcript hubs share: attach one, detach it by id, tell all.

use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, PoisonError, RwLock};

type Listener<E> = Arc<dyn Fn(&E) + Send + Sync>;

/// The listeners of one kind of event.
pub struct Hub<E> {
    listeners: RwLock<Vec<(u64, Listener<E>)>>,
    next: AtomicU64,
}

impl<E> Default for Hub<E> {
    fn default() -> Self {
        Hub {
            listeners: RwLock::new(Vec::new()),
            next: AtomicU64::new(0),
        }
    }
}

impl<E> Hub<E> {
    /// Attaches a listener and gives its id.
    pub fn attach(&self, f: impl Fn(&E) + Send + Sync + 'static) -> u64 {
        let id = self.next.fetch_add(1, Ordering::Relaxed);
        let mut all = self
            .listeners
            .write()
            .unwrap_or_else(PoisonError::into_inner);
        all.push((id, Arc::new(f)));
        id
    }

    /// Detaches the listener of `id`.
    pub fn detach(&self, id: u64) {
        let mut all = self
            .listeners
            .write()
            .unwrap_or_else(PoisonError::into_inner);
        all.retain(|(held, _)| *held != id);
    }

    /// Tells every listener one event. The listeners are copied first: one may end its own
    /// stream, which detaches it.
    pub fn publish(&self, event: &E) {
        let all = self
            .listeners
            .read()
            .unwrap_or_else(PoisonError::into_inner);
        let now: Vec<Listener<E>> = all.iter().map(|(_, f)| Arc::clone(f)).collect();
        drop(all);
        for f in now {
            f(event);
        }
    }

    /// How many listeners are attached.
    #[must_use]
    pub fn count(&self) -> usize {
        self.listeners
            .read()
            .unwrap_or_else(PoisonError::into_inner)
            .len()
    }
}
