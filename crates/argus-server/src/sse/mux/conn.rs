//! One mux connection: the sender of its stream and one fan-out client per scope it follows.

use std::sync::{Mutex, MutexGuard, PoisonError};

use argus_core::ApiError;
use argus_wake::service::driver::SharedWake;
use serde_json::Value;

use super::ops::{self, Env, Handles};
use super::presence::{self, Tokens};
use super::tails::{self, Tails};
use crate::Services;
use crate::sse::{Fanout, SseSender};

/// What a connection holds, behind the lock that serialises its POSTs and its end.
#[derive(Default)]
pub(super) struct State {
    pub(super) handles: Handles,
    pub(super) presence: Tokens,
    pub(super) tails: Tails,
    closed: bool,
}

/// A connection of the mux stream.
pub struct Conn {
    id: String,
    sender: SseSender,
    wake: Option<SharedWake>,
    state: Mutex<State>,
}

impl Conn {
    pub(super) fn new(id: String, sender: SseSender, wake: Option<SharedWake>) -> Self {
        Conn {
            id,
            sender,
            wake,
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
    pub fn apply(&self, svc: &Services, list: &[Value]) -> Result<Vec<Value>, ApiError> {
        let mut state = self.lock();
        if state.closed {
            return Err(ApiError::not_found("the stream is closed"));
        }
        let env = Env {
            svc,
            sender: &self.sender,
            wake: self.wake.as_ref(),
        };
        Ok(ops::apply(&env, &mut state, list))
    }

    /// Ends the connection: removes every client it held, gives back every presence claim and drops every tail.
    /// Safe to call twice.
    pub(super) fn teardown(&self, fanout: &Fanout) {
        let mut state = self.lock();
        state.closed = true;
        for (_, h) in state.handles.drain() {
            fanout.drop_client(&h);
        }
        presence::release_all(self.wake.as_ref(), &mut state.presence);
        tails::release_all(&mut state.tails);
    }
}
