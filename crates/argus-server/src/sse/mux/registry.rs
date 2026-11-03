//! The open mux connections, by id, and the end of one: when the stream body drops, every client
//! of the connection leaves the fan-out and the connection is forgotten.

use std::collections::HashMap;
use std::sync::atomic::AtomicUsize;
use std::sync::{Arc, Mutex, MutexGuard, PoisonError};
use std::time::Duration;

use argus_core::ApiError;
use axum::response::Response;
use serde_json::json;

use super::{Conn, VERSION};
use crate::sse::{Fanout, open_stream};

/// Connections the server holds at once.
pub const MAX_CONNS: usize = 64;

/// The open connections.
#[derive(Default)]
pub struct MuxRegistry {
    conns: Mutex<HashMap<String, Arc<Conn>>>,
}

fn new_id() -> String {
    let mut bytes = [0u8; 16];
    getrandom::fill(&mut bytes).unwrap_or_default();
    argus_core::config::hex(&bytes)
}

impl MuxRegistry {
    fn lock(&self) -> MutexGuard<'_, HashMap<String, Arc<Conn>>> {
        self.conns.lock().unwrap_or_else(PoisonError::into_inner)
    }

    /// The connection with this id.
    #[must_use]
    pub fn get(&self, id: &str) -> Option<Arc<Conn>> {
        self.lock().get(id).cloned()
    }

    /// How many connections are open.
    #[must_use]
    pub fn len(&self) -> usize {
        self.lock().len()
    }

    /// Whether no connection is open.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.lock().is_empty()
    }

    /// Opens a connection: the stream, whose first frame is `hello {conn, v}`.
    ///
    /// # Errors
    /// 503 past [`MAX_CONNS`] connections.
    pub fn open(
        self: &Arc<Self>,
        fanout: &Arc<Fanout>,
        heartbeat: Duration,
        open: &Arc<AtomicUsize>,
    ) -> Result<Response, ApiError> {
        let mut conns = self.lock();
        if conns.len() >= MAX_CONNS {
            return Err(ApiError::unavailable(format!(
                "too many mux connections open (max {MAX_CONNS})"
            )));
        }
        let id = new_id();
        let (registry, fanout, gone) = (Arc::downgrade(self), Arc::downgrade(fanout), id.clone());
        let on_drop = Box::new(move || {
            if let (Some(r), Some(f)) = (registry.upgrade(), fanout.upgrade()) {
                r.forget(&gone, &f);
            }
        });
        let (sender, res) = open_stream(heartbeat, open, Some(on_drop));
        let hello = json!({ "conn": id, "v": VERSION });
        let _sent = sender.mux_opening_event("hello", &hello);
        conns.insert(id.clone(), Arc::new(Conn::new(id, sender)));
        Ok(res)
    }

    /// Removes the connection after its clients, so a connection that is gone from the registry
    /// has left no client in the fan-out.
    fn forget(&self, id: &str, fanout: &Fanout) {
        let conn = self.get(id);
        if let Some(c) = &conn {
            c.teardown(fanout);
        }
        self.lock().remove(id);
    }

    /// Ends every stream (shutdown).
    pub fn close_all(&self) {
        self.lock().values().for_each(|c| c.sender().close());
    }
}
