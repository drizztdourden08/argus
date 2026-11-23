//! One transcript stream. It listens before it replays; the live lines that arrive during the
//! replay wait, and one cursor under one lock orders the replay and the live lines, so no line is
//! missed, repeated or sent out of order.

use std::sync::{Arc, Mutex, MutexGuard, PoisonError};

use argus_overlord::transcript::Page;
use serde_json::{Value, json};

use crate::adapters::overlord::feed::FedLine;
use crate::sse::SseSender;

struct State {
    cursor: usize,
    replaying: bool,
    waiting: Vec<FedLine>,
}

pub(super) struct Follow {
    sender: SseSender,
    session: String,
    state: Mutex<State>,
}

fn locked(m: &Mutex<State>) -> MutexGuard<'_, State> {
    m.lock().unwrap_or_else(PoisonError::into_inner)
}

impl Follow {
    pub(super) fn new(sender: SseSender, session: String, cursor: usize) -> Arc<Self> {
        Arc::new(Self {
            sender,
            session,
            state: Mutex::new(State {
                cursor,
                replaying: true,
                waiting: Vec::new(),
            }),
        })
    }

    fn emit(&self, state: &mut State, index: usize, line: &str) {
        if index <= state.cursor {
            return;
        }
        state.cursor = index;
        self.send(index, line, i64::try_from(index).ok());
    }

    fn send(&self, index: usize, line: &str, id: Option<i64>) {
        let event: Value =
            serde_json::from_str(line).unwrap_or_else(|_| Value::String(line.to_owned()));
        let data = json!({ "session": self.session, "index": index, "event": event });
        let _ = self.sender.event("line", &data, id);
    }

    fn emit_fed(&self, state: &mut State, fed: &FedLine) {
        if fed.stored {
            self.emit(state, fed.index, &fed.line);
        } else {
            self.send(state.cursor, &fed.line, None);
        }
    }

    /// A line the feed published: sent now, or kept until the replay is over.
    pub(super) fn live(&self, fed: &FedLine) {
        if fed.session != self.session {
            return;
        }
        let mut state = locked(&self.state);
        if state.replaying {
            state.waiting.push(fed.clone());
        } else {
            self.emit_fed(&mut state, fed);
        }
    }

    /// The lines stored after the cursor, then the live lines that waited.
    pub(super) fn replayed(&self, page: &Page) {
        let mut state = locked(&self.state);
        let first = state.cursor;
        for (i, line) in page.lines.iter().enumerate() {
            self.emit(&mut state, first + i + 1, line);
        }
        for fed in std::mem::take(&mut state.waiting) {
            self.emit_fed(&mut state, &fed);
        }
        state.replaying = false;
    }
}
