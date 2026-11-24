//! The wake listener a hosted child holds while it runs: attached when the child starts, detached and
//! the approval gate closed when the child ends. The live and the shadow observer both hold one.

use std::path::PathBuf;
use std::sync::{Arc, Mutex, PoisonError};

use argus_wake::engine::{AttachOptions, ListenerId};
use argus_wake::hosted::HostedSink;
use argus_wake::service::driver::{SharedWake, lock};

use crate::adapters::overlord::gate::ApprovalGate;
use argus_board::roles::overlord_key;

/// One child's hold on the wake engine and on the approval gate.
pub(super) struct Attachment {
    wake: SharedWake,
    gate: Arc<ApprovalGate>,
    coord_home: PathBuf,
    id: Mutex<Option<ListenerId>>,
}

impl Attachment {
    pub(super) fn new(wake: SharedWake, gate: Arc<ApprovalGate>, coord_home: PathBuf) -> Self {
        Self {
            wake,
            gate,
            coord_home,
            id: Mutex::new(None),
        }
    }

    /// Attaches `sink` to `thread` of the Overlord board and keeps its id for [`Self::end`].
    pub(super) fn attach(&self, thread: &str, sink: HostedSink, options: AttachOptions) {
        let scope = overlord_key(&self.coord_home);
        let attached = lock(&self.wake).attach(&scope, thread, Box::new(sink), options);
        *self.id.lock().unwrap_or_else(PoisonError::into_inner) = Some(attached.id);
    }

    /// Closes the approval gate and detaches the listener, if one is attached.
    pub(super) fn end(&self) {
        self.gate.end();
        let id = self
            .id
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .take();
        if let Some(id) = id {
            lock(&self.wake).detach(&id);
        }
    }
}
