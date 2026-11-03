//! The client registry of the fan-out: finding a client by its handle, adding and removing
//! collections under the lock, and the three steps of an add (begin, read, finish).

use std::collections::HashMap;
use std::sync::atomic::Ordering;
use std::sync::{MutexGuard, PoisonError};

use argus_core::{ApiError, Coll};

use super::client::Client;
use super::first_frames::First;
use super::{Fanout, Handle, MAX_CLIENTS_PER_SCOPE, Target};

fn closed() -> ApiError {
    ApiError::not_found("the stream is closed")
}

impl Fanout {
    pub(super) fn lock(&self) -> MutexGuard<'_, HashMap<String, Vec<Client>>> {
        self.clients.lock().unwrap_or_else(PoisonError::into_inner)
    }

    pub(super) fn new_handle(&self, scope: &str) -> Handle {
        Handle {
            scope: scope.to_owned(),
            id: self.next.fetch_add(1, Ordering::SeqCst),
        }
    }

    /// Runs `f` on the client; the client leaves when `f` answers `false` or it follows nothing.
    /// `true` when the client is gone afterwards.
    pub(super) fn edit(&self, h: &Handle, f: impl FnOnce(&mut Client) -> bool) -> bool {
        let mut all = self.lock();
        let Some(list) = all.get_mut(&h.scope) else {
            return true;
        };
        if let Some(i) = list.iter().position(|c| c.id == h.id) {
            let keep = f(&mut list[i]);
            if !keep || list[i].is_idle() {
                list.remove(i);
            }
        }
        let gone = !list.iter().any(|c| c.id == h.id);
        if list.is_empty() {
            all.remove(&h.scope);
        }
        gone
    }

    /// Removes the client and everything it follows. For the end of a stream; it never touches
    /// the sender, so it is safe when other clients share it.
    pub fn drop_client(&self, h: &Handle) {
        self.edit(h, |_| false);
    }

    /// Stops following `colls` (pending ones too). `true` when the client followed nothing more
    /// and left the fan-out; the handle is dead then.
    pub fn unsubscribe(&self, h: &Handle, colls: &[Coll]) -> bool {
        self.edit(h, |c| {
            c.remove(colls);
            true
        })
    }

    /// Starts an add under the lock: registers a new client or finds the existing one, and marks
    /// the collections new to it pending. Answers the add's token and those collections.
    pub(super) fn begin(
        &self,
        h: &Handle,
        target: Target<'_>,
        colls: &[Coll],
    ) -> Result<(u64, Vec<Coll>), ApiError> {
        if colls.is_empty() {
            return Err(ApiError::bad_request("no collection to follow"));
        }
        let token = self.next.fetch_add(1, Ordering::SeqCst);
        let mut all = self.lock();
        let list = match target {
            Target::New(..) => all.entry(h.scope.clone()).or_default(),
            Target::Add(_) => all.get_mut(&h.scope).ok_or_else(closed)?,
        };
        if let Target::New(kind, sender) = target {
            if list.len() >= MAX_CLIENTS_PER_SCOPE {
                return Err(ApiError::unavailable(format!(
                    "too many streams open for {} (max {MAX_CLIENTS_PER_SCOPE})",
                    h.scope
                )));
            }
            list.push(Client::new(h.id, kind, sender.clone()));
        }
        let client = list.iter_mut().find(|c| c.id == h.id).ok_or_else(closed)?;
        Ok((token, client.begin_add(token, colls)))
    }

    /// Ends an add under the lock: the first frames, the buffered events, `ready`, live.
    pub(super) fn finish(&self, h: &Handle, token: u64, first: &First) {
        self.edit(h, |c| c.finish_add(&h.scope, token, first));
    }

    /// Drops an add whose rows could not be read; a client that followed nothing leaves.
    pub(super) fn abort(&self, h: &Handle, token: u64) {
        self.edit(h, |c| {
            c.cancel_add(token);
            true
        });
    }
}
