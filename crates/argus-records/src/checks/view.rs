//! A run as the API shows it, and where the log of one of its steps is.

use std::path::{Path, PathBuf};

use serde_json::{Map, Value};

use super::store::{CheckRun, CheckStore};
use crate::fields::{list, opt_text};
use crate::safe_path::is_inside;

/// The record with `id` set to `C-n` and `runId` and `project` from the file.
#[must_use]
pub fn run_json(run: &CheckRun) -> Value {
    let mut out = Map::new();
    out.insert("id".into(), Value::String(run.id()));
    out.insert("runId".into(), Value::String(run.run_id.clone()));
    if let Value::Object(fields) = &run.record {
        for (key, value) in fields
            .iter()
            .filter(|(k, _)| !["id", "runId"].contains(&k.as_str()))
        {
            out.insert(key.clone(), value.clone());
        }
    }
    out.insert("project".into(), Value::String(run.project.clone()));
    Value::Object(out)
}

fn resolved(base: &Path, log: &str) -> PathBuf {
    let path = PathBuf::from(log);
    if path.is_absolute() {
        path
    } else {
        base.join(path)
    }
}

impl CheckStore {
    /// The log file of step `step` of a run: the `log` the step names, found inside the store
    /// (`<root>/<project>/`) or inside the run's own `logDir`. Anything else is `None`.
    #[must_use]
    pub fn step_log(&self, run: &CheckRun, step: usize) -> Option<PathBuf> {
        let log = opt_text(list(&run.record, "steps").get(step)?, "log")?;
        let project_dir = self.root().join(&run.project);
        let file = resolved(&project_dir, &log);
        let own = opt_text(&run.record, "logDir").map(|d| resolved(&project_dir, &d));
        let allowed = is_inside(self.root(), &file) || own.is_some_and(|d| is_inside(&d, &file));
        (allowed && file.is_file()).then_some(file)
    }
}
