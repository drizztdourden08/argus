//! One mux connection: the sender of its stream and one fan-out client per scope it follows.

use std::sync::{Mutex, MutexGuard, PoisonError};

use argus_core::ApiError;
use argus_store::Db;
use serde_json::Value;

use super::ops::{self, Env, Handles};
use crate::sse::{Fanout, SseSender};

#[derive(Default)]
struct State {
    handles: Handles,
    closed: bool,
}

/// A connection of the mux stream.
pub struct Conn {
    id: String,
    sender: SseSender,
    state: Mutex<State>,
}

impl Conn {
    pub(super) fn new(id: String, sender: SseSender) -> Self {
        Conn {
            id,
            sender,
            state: Mutex::new(State::default()),
        }
    }

    /// The connection id, the capability the POST route takes.
    #[must_use]
    pub fn id(&self) -> &str {
        &self.id
    }

    pub(super) fn sender(&self) -> &SseSender {
        &self.sender
    }

    fn lock(&self) -> MutexGuard<'_, State> {
        self.state.lock().unwrap_or_else(PoisonError::into_inner)
    }

    /// Applies the ops in order and answers one result per op. The whole list runs under the
    /// connection's lock, so two POSTs never interleave their ops and the frames of an op are
    /// queued before the next op starts.
    ///
    /// # Errors
    /// 404 when the stream is gone.
    pub fn apply(&self, db: &Db, fanout: &Fanout, list: &[Value]) -> Result<Vec<Value>, ApiError> {
        let mut state = self.lock();
        if state.closed {
            return Err(ApiError::not_found("the stream is closed"));
        }
        let env = Env {
            db,
            fanout,
            sender: &self.sender,
        };
        Ok(ops::apply(&env, &mut state.handles, list))
    }

    /// Ends the connection: removes every client it held. Safe to call twice.
    pub(super) fn teardown(&self, fanout: &Fanout) {
        let mut state = self.lock();
        state.closed = true;
        for (_, h) in state.handles.drain() {
            fanout.drop_client(&h);
        }
    }
}
