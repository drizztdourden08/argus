//! The `agents` docs of the doc store, where the watcher keeps the `live` field of a record,
//! and the wake engine as the sub-agent services take it.

use std::sync::Arc;

use argus_agents::record::Record;
use argus_agents::{AgentsError, LiveDocs, WakeEvent, WakeSink};
use argus_core::{Coll, DocChange};
use argus_store::Db;
use serde_json::Value;

use crate::services::{WakeHook, WakeRequest};

/// The agents docs of a database.
pub struct AgentDocs {
    db: Arc<Db>,
}

impl AgentDocs {
    /// The docs of a database.
    #[must_use]
    pub fn new(db: Arc<Db>) -> Self {
        AgentDocs { db }
    }
}

impl LiveDocs for AgentDocs {
    fn get(&self, key: &str, id: &str) -> Option<Record> {
        let row = self.db.docs().get(key, Coll::Agents, id).ok()??;
        row.data.as_object().cloned()
    }

    fn set(&self, key: &str, id: &str, data: Record) -> Result<(), AgentsError> {
        let change = DocChange::set(Coll::Agents, id, Value::Object(data));
        self.db
            .docs()
            .apply(key, &[change])
            .map(|_| ())
            .map_err(|e| AgentsError::Docs(format!("{key}/{id}"), e.to_string()))
    }
}

/// The wake service for the reaper and the reconciler.
pub struct AgentWakes {
    hook: WakeHook,
}

impl AgentWakes {
    /// Wakes through `hook`.
    #[must_use]
    pub fn new(hook: WakeHook) -> Self {
        AgentWakes { hook }
    }
}

impl WakeSink for AgentWakes {
    fn submit(&self, events: &[WakeEvent]) -> Result<(), AgentsError> {
        let wakes: Vec<WakeRequest> = events
            .iter()
            .map(|e| WakeRequest::about(&e.scope, &e.thread, &e.reason, &e.subject, &e.text))
            .collect();
        self.hook.submit(&wakes).map_err(AgentsError::Wake)
    }
}
