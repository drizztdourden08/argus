//! The live tail of run logs: the lines the run service flushed, told to the streams that
//! watch a run. A stream listens until its [`LogListening`] is dropped.

use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, PoisonError, RwLock};

use argus_launchers::LogEvent;

type Listener = Arc<dyn Fn(&LogEvent) + Send + Sync>;

/// The hub between the run service's flushed lines and the log streams.
#[derive(Default)]
pub struct RunLogs {
    listeners: RwLock<Vec<(u64, Listener)>>,
    next: AtomicU64,
}

/// A listener on the hub; dropping it detaches the listener.
pub struct LogListening {
    hub: Arc<RunLogs>,
    id: u64,
}

impl Drop for LogListening {
    fn drop(&mut self) {
        let mut all = self
            .hub
            .listeners
            .write()
            .unwrap_or_else(PoisonError::into_inner);
        all.retain(|(id, _)| *id != self.id);
    }
}

impl RunLogs {
    /// Listens to every flushed line until the returned value is dropped.
    pub fn listen(self: &Arc<Self>, f: impl Fn(&LogEvent) + Send + Sync + 'static) -> LogListening {
        let id = self.next.fetch_add(1, Ordering::Relaxed);
        let mut all = self
            .listeners
            .write()
            .unwrap_or_else(PoisonError::into_inner);
        all.push((id, Arc::new(f)));
        LogListening {
            hub: Arc::clone(self),
            id,
        }
    }

    /// Tells every listener one line. The listeners are copied first: one may end its own
    /// stream, which detaches it.
    pub fn publish(&self, event: &LogEvent) {
        let all = self
            .listeners
            .read()
            .unwrap_or_else(PoisonError::into_inner);
        let now: Vec<Listener> = all.iter().map(|(_, f)| Arc::clone(f)).collect();
        drop(all);
        for f in now {
            f(event);
        }
    }

    /// How many listeners are attached.
    #[must_use]
    pub fn listener_count(&self) -> usize {
        self.listeners
            .read()
            .unwrap_or_else(PoisonError::into_inner)
            .len()
    }
}
