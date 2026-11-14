//! The hub that tells the transcript streams about a new line. The session supervisor publishes
//! after it stored a line; a stream listens until its [`FeedListening`] is dropped. Like the run
//! log hub, nothing polls.

use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, PoisonError, RwLock};

/// One stored line: `index` is the number of lines the file holds with it (1 for the first).
#[derive(Debug, Clone)]
pub struct FedLine {
    pub session: String,
    pub index: usize,
    pub line: String,
}

type Listener = Arc<dyn Fn(&FedLine) + Send + Sync>;

/// The hub between the supervisor's stored lines and the transcript streams.
#[derive(Default)]
pub struct TranscriptFeed {
    listeners: RwLock<Vec<(u64, Listener)>>,
    next: AtomicU64,
}

/// A listener on the hub; dropping it detaches the listener.
pub struct FeedListening {
    hub: Arc<TranscriptFeed>,
    id: u64,
}

impl Drop for FeedListening {
    fn drop(&mut self) {
        let mut all = self
            .hub
            .listeners
            .write()
            .unwrap_or_else(PoisonError::into_inner);
        all.retain(|(id, _)| *id != self.id);
    }
}

impl TranscriptFeed {
    /// Listens to every published line until the returned value is dropped.
    pub fn listen(self: &Arc<Self>, f: impl Fn(&FedLine) + Send + Sync + 'static) -> FeedListening {
        let id = self.next.fetch_add(1, Ordering::Relaxed);
        let mut all = self
            .listeners
            .write()
            .unwrap_or_else(PoisonError::into_inner);
        all.push((id, Arc::new(f)));
        FeedListening {
            hub: Arc::clone(self),
            id,
        }
    }

    /// Tells every listener one line. The listeners are copied first: one may end its own
    /// stream, which detaches it.
    pub fn publish(&self, line: &FedLine) {
        let all = self
            .listeners
            .read()
            .unwrap_or_else(PoisonError::into_inner);
        let now: Vec<Listener> = all.iter().map(|(_, f)| Arc::clone(f)).collect();
        drop(all);
        for f in now {
            f(line);
        }
    }
}
