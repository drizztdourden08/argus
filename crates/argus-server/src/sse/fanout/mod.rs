//! The doc fan-out: bus events to the streams of their scope. A client follows a set of
//! collections of one scope. On connect, and whenever collections are added, it gets the missed
//! changes when the log still reaches back to `since`, else one snapshot per added collection;
//! then `ready`. Every doc apply becomes at most one `changes` event per client, filtered to its
//! collections; a hard resync sends `reset`.
//!
//! Adding collections never pauses the others. The added collections are marked pending under
//! the lock, the rows are read outside it, and under the lock again the first frames go out, then
//! the events buffered for those collections above the read seq, then the collections go live.
//! The old route (`attach`) is one client of the legacy kind built on the same calls.

mod client;
mod first_frames;
mod frames;
mod registry;
#[cfg(test)]
mod tests;

use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, AtomicUsize};
use std::sync::{Arc, Mutex};
use std::time::Duration;

use argus_core::{ApiError, Bus, Coll, DocsEvent, ResetEvent, Subscription};
use argus_store::Db;
use axum::response::Response;

use self::client::Client;
use self::first_frames::first_frames;
pub use self::frames::Kind;
use super::{SseSender, open_stream};

/// Streams one scope may hold.
pub const MAX_CLIENTS_PER_SCOPE: usize = 32;

/// What a new client asked for.
pub struct StreamRequest<'a> {
    /// The scope.
    pub scope: &'a str,
    /// Its collections.
    pub colls: Vec<Coll>,
    /// The resume point.
    pub since: Option<i64>,
}

/// One client of the fan-out: a scope of a stream. Cloning names the same client.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Handle {
    scope: String,
    id: u64,
}

impl Handle {
    /// The scope the client follows.
    #[must_use]
    pub fn scope(&self) -> &str {
        &self.scope
    }
}

/// Where a subscription goes.
#[derive(Clone, Copy)]
pub enum Target<'a> {
    /// A new client of this kind that writes to this sender. Counts toward the scope's cap.
    New(Kind, &'a SseSender),
    /// More collections for an existing client.
    Add(&'a Handle),
}

/// The fan-out hub: bus events to the streams of their scope.
#[derive(Default)]
pub struct Fanout {
    clients: Mutex<HashMap<String, Vec<Client>>>,
    next: AtomicU64,
}

impl Fanout {
    /// Follows the bus: doc applies and resets reach the streams until the links are dropped.
    #[must_use]
    pub fn link(self: &Arc<Self>, bus: &Bus) -> Vec<Subscription> {
        let (f1, f2) = (Arc::clone(self), Arc::clone(self));
        vec![
            bus.on_docs(move |e| f1.on_docs(e)),
            bus.on_reset(move |e| f2.on_reset(e)),
        ]
    }

    /// Delivers one doc apply.
    pub fn on_docs(&self, e: &DocsEvent) {
        let mut all = self.lock();
        if let Some(list) = all.get_mut(&e.scope) {
            list.retain_mut(|c| c.deliver(&e.scope, e));
        }
    }

    /// Delivers a reset.
    pub fn on_reset(&self, e: &ResetEvent) {
        let mut all = self.lock();
        if let Some(list) = all.get_mut(&e.scope) {
            list.retain(|c| c.reset(&e.scope, &e.reason));
        }
    }

    /// Ends every stream (shutdown).
    pub fn close_all(&self) {
        let mut all = self.lock();
        all.values().flatten().for_each(Client::close);
        all.clear();
    }

    fn subscribe_to(
        &self,
        db: &Db,
        h: &Handle,
        target: Target<'_>,
        colls: &[Coll],
        since: Option<i64>,
    ) -> Result<(), ApiError> {
        if since.is_none() && matches!(target, Target::Add(_)) {
            self.forget_live(h, colls);
        }
        let (token, read) = self.begin(h, target, colls)?;
        if read.is_empty() {
            let seq = db.docs().seq()?;
            self.edit(h, |c| c.ack(&h.scope, seq));
            return Ok(());
        }
        match first_frames(db, &h.scope, &read, since) {
            Ok(first) => {
                self.finish(h, token, &first);
                Ok(())
            }
            Err(e) => {
                self.abort(h, token);
                Err(e)
            }
        }
    }

    /// Follows `colls` of `scope`: the missed changes when `since` is servable, else one snapshot
    /// per collection, then `ready`. Without `since`, collections the client already follows get
    /// a fresh snapshot again; with `since`, those it follows send nothing, and when nothing is
    /// new a mux client still gets `ready` with no collections.
    /// The other collections of the client keep streaming meanwhile.
    ///
    /// # Errors
    /// 400 for no collection or a handle of another scope; 503 past 32 clients for the scope; 404
    /// when the handle's client left; a store failure.
    pub fn subscribe(
        &self,
        db: &Db,
        scope: &str,
        target: Target<'_>,
        colls: &[Coll],
        since: Option<i64>,
    ) -> Result<Handle, ApiError> {
        let h = match target {
            Target::New(..) => self.new_handle(scope),
            Target::Add(h) if h.scope == scope => h.clone(),
            Target::Add(h) => {
                return Err(ApiError::bad_request(format!(
                    "the client follows {}, not {scope}",
                    h.scope
                )));
            }
        };
        self.subscribe_to(db, &h, target, colls, since)?;
        Ok(h)
    }

    /// Opens a doc stream: the first frames, then live changes, in the legacy frame shapes.
    ///
    /// # Errors
    /// 503 past 32 streams for the scope; a store failure.
    pub fn attach(
        self: &Arc<Self>,
        db: &Db,
        req: &StreamRequest,
        heartbeat: Duration,
        open: &Arc<AtomicUsize>,
    ) -> Result<Response, ApiError> {
        let h = self.new_handle(req.scope);
        let (weak, gone) = (Arc::downgrade(self), h.clone());
        let on_drop = Box::new(move || {
            if let Some(f) = weak.upgrade() {
                f.drop_client(&gone);
            }
        });
        let (sender, res) = open_stream(heartbeat, open, Some(on_drop));
        let target = Target::New(Kind::Legacy, &sender);
        self.subscribe_to(db, &h, target, &req.colls, req.since)?;
        Ok(res)
    }
}
