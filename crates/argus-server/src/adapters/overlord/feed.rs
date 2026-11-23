//! The hub that tells the transcript streams about a new line. The session supervisor publishes
//! after it stored a line; a stream listens until its [`FeedListening`] is dropped. Like the run
//! log hub, nothing polls.

use std::sync::Arc;

use crate::sse::Hub;

/// One line for the streams: `index` is the number of lines the file holds with it (1 for the
/// first). A line that is not `stored` is only for the pages open now (a partial message): its
/// `index` is the stored count at that moment and it never moves a stream's cursor.
#[derive(Debug, Clone)]
pub struct FedLine {
    pub session: String,
    pub index: usize,
    pub line: String,
    pub stored: bool,
}

/// The hub between the supervisor's stored lines and the transcript streams.
#[derive(Default)]
pub struct TranscriptFeed {
    hub: Hub<FedLine>,
}

/// A listener on the hub; dropping it detaches the listener.
pub struct FeedListening {
    feed: Arc<TranscriptFeed>,
    id: u64,
}

impl Drop for FeedListening {
    fn drop(&mut self) {
        self.feed.hub.detach(self.id);
    }
}

impl TranscriptFeed {
    /// Listens to every published line until the returned value is dropped.
    pub fn listen(self: &Arc<Self>, f: impl Fn(&FedLine) + Send + Sync + 'static) -> FeedListening {
        FeedListening {
            feed: Arc::clone(self),
            id: self.hub.attach(f),
        }
    }

    /// Tells every listener one line.
    pub fn publish(&self, line: &FedLine) {
        self.hub.publish(line);
    }
}
