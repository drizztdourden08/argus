//! In-process events between the stores and the streams. An emit calls every listener in the
//! emitting thread, as the Node bus does, so a stream sees a change before the writer returns.

use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex, PoisonError};

use crate::{DocsEvent, ResetEvent};

type Listener<E> = Arc<dyn Fn(&E) + Send + Sync>;

struct Topic<E> {
    listeners: Mutex<Vec<(u64, Listener<E>)>>,
}

impl<E> Topic<E> {
    fn new() -> Self {
        Topic {
            listeners: Mutex::new(Vec::new()),
        }
    }

    fn add(&self, id: u64, f: Listener<E>) {
        self.listeners
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .push((id, f));
    }

    fn remove(&self, id: u64) {
        self.listeners
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .retain(|(i, _)| *i != id);
    }

    fn emit(&self, e: &E) {
        let now: Vec<Listener<E>> = self
            .listeners
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .iter()
            .map(|(_, f)| Arc::clone(f))
            .collect();
        for f in now {
            f(e);
        }
    }

    fn len(&self) -> usize {
        self.listeners
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .len()
    }
}

struct Inner {
    next: AtomicU64,
    docs: Topic<DocsEvent>,
    reset: Topic<ResetEvent>,
}

/// The event bus. Cloning shares it.
#[derive(Clone)]
pub struct Bus(Arc<Inner>);

impl Default for Bus {
    fn default() -> Self {
        Self::new()
    }
}

impl Bus {
    /// A new, empty bus.
    #[must_use]
    pub fn new() -> Self {
        Bus(Arc::new(Inner {
            next: AtomicU64::new(1),
            docs: Topic::new(),
            reset: Topic::new(),
        }))
    }

    /// Listen to doc applies until the subscription is dropped.
    pub fn on_docs(&self, f: impl Fn(&DocsEvent) + Send + Sync + 'static) -> Subscription {
        let id = self.0.next.fetch_add(1, Ordering::Relaxed);
        self.0.docs.add(id, Arc::new(f));
        Subscription {
            bus: self.clone(),
            id,
        }
    }

    /// Listen to resets until the subscription is dropped.
    pub fn on_reset(&self, f: impl Fn(&ResetEvent) + Send + Sync + 'static) -> Subscription {
        let id = self.0.next.fetch_add(1, Ordering::Relaxed);
        self.0.reset.add(id, Arc::new(f));
        Subscription {
            bus: self.clone(),
            id,
        }
    }

    /// Tell every doc listener.
    pub fn emit_docs(&self, e: &DocsEvent) {
        self.0.docs.emit(e);
    }

    /// Tell every reset listener.
    pub fn emit_reset(&self, e: &ResetEvent) {
        self.0.reset.emit(e);
    }

    /// How many listeners are attached.
    #[must_use]
    pub fn listener_count(&self) -> usize {
        self.0.docs.len() + self.0.reset.len()
    }

    fn remove(&self, id: u64) {
        self.0.docs.remove(id);
        self.0.reset.remove(id);
    }
}

/// A listener on the bus; dropping it detaches the listener.
pub struct Subscription {
    bus: Bus,
    id: u64,
}

impl Drop for Subscription {
    fn drop(&mut self) {
        self.bus.remove(self.id);
    }
}
