//! The live tail of run logs: the lines the run service flushed, told to the streams that
//! watch a run. A stream listens until its [`LogListening`] is dropped.

use std::sync::Arc;

use argus_launchers::LogEvent;

use super::hub::Hub;

/// The hub between the run service's flushed lines and the log streams.
#[derive(Default)]
pub struct RunLogs {
    hub: Hub<LogEvent>,
}

/// A listener on the hub; dropping it detaches the listener.
pub struct LogListening {
    logs: Arc<RunLogs>,
    id: u64,
}

impl Drop for LogListening {
    fn drop(&mut self) {
        self.logs.hub.detach(self.id);
    }
}

impl RunLogs {
    /// Listens to every flushed line until the returned value is dropped.
    pub fn listen(self: &Arc<Self>, f: impl Fn(&LogEvent) + Send + Sync + 'static) -> LogListening {
        LogListening {
            logs: Arc::clone(self),
            id: self.hub.attach(f),
        }
    }

    /// Tells every listener one line.
    pub fn publish(&self, event: &LogEvent) {
        self.hub.publish(event);
    }

    /// How many listeners are attached.
    #[must_use]
    pub fn listener_count(&self) -> usize {
        self.hub.count()
    }
}
