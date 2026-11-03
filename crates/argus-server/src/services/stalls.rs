//! What the stall watch of the wake engine reads from the server, plugged in once every service is
//! built: the database kv for the inbox reads and the acks, and the processes and runs of a project
//! for "is it busy". The wake engine writes the `_portfolio/stalls` docs itself, through its doc sink.

use std::sync::Arc;

use argus_core::ids::PORTFOLIO;
use argus_core::{Coll, Config};
use argus_store::Db;
use argus_wake::service::driver::lock;
use argus_wake::stalls::{BusySource, StallConfig, StallKv};
use serde_json::Value;

use crate::Services;

const RUNNING: &str = "running";
const ORPHAN: &str = "orphan";

/// The kv of the database. A shadow server reads it and writes nothing.
struct KvStall {
    db: Arc<Db>,
    shadow: bool,
}

impl StallKv for KvStall {
    fn get(&self, key: &str) -> Option<String> {
        self.db.kv_get(key).ok().flatten()
    }

    fn set(&self, key: &str, value: &str) {
        if !self.shadow {
            let _ = self.db.kv_set(key, value);
        }
    }
}

/// A project is busy while one of its runs is `running`, or a process listed for it is alive, not
/// protected, not an orphan and not quieter than `idle_min` minutes.
struct ProcBusy {
    db: Arc<Db>,
    idle_min: i64,
}

impl ProcBusy {
    fn docs(&self, scope: &str, coll: Coll) -> Vec<Value> {
        let rows = self.db.docs().list(scope, coll).unwrap_or_default();
        rows.into_iter().map(|row| row.data).collect()
    }

    fn running(&self, project: &str) -> bool {
        self.docs(project, Coll::Runs)
            .iter()
            .any(|run| run["status"] == RUNNING)
    }

    fn working(&self, doc: &Value) -> bool {
        let orphan = doc["flags"]
            .as_array()
            .is_some_and(|flags| flags.iter().any(|flag| flag == ORPHAN));
        doc["protected"] != true && !orphan && doc["idleMin"].as_i64().unwrap_or(0) < self.idle_min
    }

    fn process(&self, project: &str) -> bool {
        self.docs(PORTFOLIO, Coll::Procs)
            .iter()
            .any(|doc| doc["project"] == project && self.working(doc))
    }
}

impl BusySource for ProcBusy {
    fn busy(&self, project: &str) -> bool {
        self.running(project) || self.process(project)
    }
}

fn settings(config: &Config) -> StallConfig {
    StallConfig::from_json(config.extra.get("stall").unwrap_or(&Value::Null))
}

pub(crate) fn wire(svc: &Services) {
    let Some(wake) = &svc.wake else {
        return;
    };
    let busy = ProcBusy {
        db: Arc::clone(&svc.db),
        idle_min: settings(&svc.config()).proc_idle_min,
    };
    let kv = KvStall {
        db: Arc::clone(&svc.db),
        shadow: svc.shadow,
    };
    lock(wake).set_stall_sources(Arc::new(busy), Arc::new(kv));
}
