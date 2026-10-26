//! The live tail of run logs: the lines the run service flushed, told to the streams that
//! watch a run. A stream listens until its [`LogListening`] is dropped.

use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex, PoisonError};

use argus_launchers::LogEvent;

type Listener = Arc<dyn Fn(&LogEvent) + Send + Sync>;

/// The hub between the run service's flushed lines and the log streams.
#[derive(Default)]
pub struct RunLogs {
    listeners: Mutex<Vec<(u64, Listener)>>,
    next: AtomicU64,
}

/// A listener on the hub; dropping it detaches the listener.
pub struct LogListening {
    hub: Arc<RunLogs>,
    id: u64,
}

impl Drop for LogListening {
    fn drop(&mut self) {
        self.hub
            .listeners
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .retain(|(id, _)| *id != self.id);
    }
}

impl RunLogs {
    /// Listens to every flushed line until the returned value is dropped.
    pub fn listen(
        self: &Arc<Self>,
        f: impl Fn(&LogEvent) + Send + Sync + 'static,
    ) -> LogListening {
        let id = self.next.fetch_add(1, Ordering::Relaxed);
        self.listeners
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .push((id, Arc::new(f)));
        LogListening {
            hub: Arc::clone(self),
            id,
        }
    }

    /// Tells every listener one line.
    pub fn publish(&self, event: &LogEvent) {
        let now: Vec<Listener> = self
            .listeners
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .iter()
            .map(|(_, f)| Arc::clone(f))
            .collect();
        for f in now {
            f(event);
        }
    }

    /// How many listeners are attached.
    #[must_use]
    pub fn listener_count(&self) -> usize {
        self.listeners
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .len()
    }
}
