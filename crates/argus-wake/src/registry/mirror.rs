//! The page mirror of the registry: collection `registry`, doc id = thread on a project scope; the
//! portfolio scope has the lead rows of every project (id `<project>/<thread>`) and every faulted row.

use tracing::warn;

use super::rows::RegistryRow;
use super::{Env, Registry};
use crate::docs::Doc;
use crate::reasons::PORTFOLIO;

const REGISTRY_COLLECTION: &str = "registry";

impl Registry {
    /// Write the docs of a scope when they differ from what was written last.
    fn write(&mut self, scope: &str, rows: &[RegistryRow]) {
        let signature = serde_json::to_string(rows).unwrap_or_default();
        if self.last_mirror.get(scope) == Some(&signature) {
            return;
        }
        let docs = rows
            .iter()
            .map(|r| Doc {
                id: r.id.clone(),
                data: serde_json::to_value(r).unwrap_or_default(),
            })
            .collect();
        match self.docs.replace(scope, REGISTRY_COLLECTION, docs) {
            Ok(()) => {
                self.last_mirror.insert(scope.to_owned(), signature);
            }
            Err(err) => {
                self.last_mirror.remove(scope);
                warn!(scope, %err, "registry mirror failed");
            }
        }
    }

    /// Mirror one project scope and the portfolio scope (or all of them).
    pub(super) fn refresh(&mut self, env: &Env<'_>, scope: Option<&str>) {
        let faults = self.open_faults();
        let all = self.scopes();
        let targets = scope.map_or_else(|| all.clone(), |s| vec![s.to_owned()]);
        for target in &targets {
            let rows = self.rows_with(env, target, &faults);
            self.write(target, &rows);
        }
        let mut lead: Vec<RegistryRow> = Vec::new();
        for project in &all {
            for row in self.rows_with(env, project, &faults) {
                if row.role == "coordinator" || row.fault.is_some() {
                    lead.push(RegistryRow {
                        id: format!("{project}/{}", row.thread),
                        ..row
                    });
                }
            }
        }
        self.write(PORTFOLIO, &lead);
    }
}
