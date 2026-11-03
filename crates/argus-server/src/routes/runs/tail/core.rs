//! The run tail itself. It listens before it replays, and one cursor under one lock orders the
//! replay and the live lines, so no line is missed or sent twice. Everything it says goes to a
//! [`TailSink`].

use std::any::Any;
use std::sync::{Arc, Mutex, MutexGuard, PoisonError};

use argus_core::Coll;
use argus_launchers::{LogEvent, LogOptions, RunService, RunStatus};

use super::sink::TailSink;
use crate::Services;
use crate::adapters::runs::RunHost;

const REPLAY_PAGE: usize = 2000;

/// The listeners a tail holds; clearing the vector drops them.
pub(crate) type Watching = Mutex<Vec<Box<dyn Any + Send>>>;

pub(crate) struct Tail {
    sink: Box<dyn TailSink>,
    cursor: Mutex<u64>,
    runs: Arc<RunService<RunHost>>,
    key: String,
    rid: String,
    watching: Arc<Watching>,
}

pub(crate) fn locked<T>(m: &Mutex<T>) -> MutexGuard<'_, T> {
    m.lock().unwrap_or_else(PoisonError::into_inner)
}

impl Tail {
    pub(crate) fn new(
        runs: Arc<RunService<RunHost>>,
        (key, rid): (String, String),
        cursor: u64,
        sink: Box<dyn TailSink>,
        watching: Arc<Watching>,
    ) -> Arc<Self> {
        Arc::new(Self {
            sink,
            cursor: Mutex::new(cursor),
            runs,
            key,
            rid,
            watching,
        })
    }

    /// Listens, replays what is stored after the cursor, then reports the run's status.
    pub(crate) fn start(self: &Arc<Self>, svc: &Services) {
        *locked(&self.watching) = self.watch(svc);
        self.replay();
        self.status();
    }

    fn line(&self, event: &LogEvent) {
        let mut cursor = locked(&self.cursor);
        if event.seq <= *cursor {
            return;
        }
        *cursor = event.seq;
        self.sink.line(event);
    }

    fn status(&self) {
        let Some(run) = self.runs.get(&self.key, &self.rid) else {
            self.finish();
            return;
        };
        self.sink.status(&run);
        if run.status != RunStatus::Queued && run.status != RunStatus::Running {
            self.finish();
        }
    }

    fn finish(&self) {
        self.sink.end();
        locked(&self.watching).clear();
    }

    fn replay(&self) {
        loop {
            let after = i64::try_from(*locked(&self.cursor)).ok();
            let options = LogOptions {
                after,
                limit: i64::try_from(REPLAY_PAGE).ok(),
                ..LogOptions::default()
            };
            let Ok(page) = self.runs.logs(&self.key, &self.rid, &options) else {
                return;
            };
            for l in &page.lines {
                self.line(&LogEvent {
                    scope: self.key.clone(),
                    rid: self.rid.clone(),
                    seq: l.seq,
                    stream: l.stream,
                    at: l.at.clone(),
                    text: l.text.clone(),
                });
            }
            if page.lines.len() < REPLAY_PAGE {
                return;
            }
        }
    }

    fn watch(self: &Arc<Self>, svc: &Services) -> Vec<Box<dyn Any + Send>> {
        let lines = Arc::clone(self);
        let docs = Arc::clone(self);
        let line_watch = svc.run_logs.listen(move |e| {
            if e.scope == lines.key && e.rid == lines.rid {
                lines.line(e);
            }
        });
        let doc_watch = svc.bus.on_docs(move |e| {
            let mine = e.scope == docs.key
                && e.changes
                    .iter()
                    .any(|c| c.change.coll == Coll::Runs && c.change.id == docs.rid);
            if mine {
                docs.status();
            }
        });
        vec![Box::new(line_watch), Box::new(doc_watch)]
    }
}
