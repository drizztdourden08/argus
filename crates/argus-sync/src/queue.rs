//! One queue per scope: a mutation starts when the previous one of that scope has finished, so
//! owner posts, proofs and resyncs apply in arrival order; scopes run side by side.

use std::collections::HashMap;
use std::future::Future;
use std::sync::{Arc, Mutex, PoisonError};

use tokio::sync::Mutex as AsyncMutex;

/// The per-scope queues. Cloning shares them.
#[derive(Clone, Default)]
pub struct ScopeQueue {
    gates: Arc<Mutex<HashMap<String, Arc<AsyncMutex<()>>>>>,
}

impl ScopeQueue {
    /// A new set of queues.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    fn gate(&self, scope: &str) -> Arc<AsyncMutex<()>> {
        let mut gates = self.gates.lock().unwrap_or_else(PoisonError::into_inner);
        Arc::clone(gates.entry(scope.to_owned()).or_default())
    }

    /// Runs `job` after every earlier job of `scope`, in arrival order.
    pub async fn run<T, F>(&self, scope: &str, job: F) -> T
    where
        F: Future<Output = T>,
    {
        let gate = self.gate(scope);
        let _turn = gate.lock().await;
        job.await
    }
}
