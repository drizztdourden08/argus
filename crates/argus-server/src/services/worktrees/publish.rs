//! The worktree docs of a project: one `worktrees` doc per worktree on disk and the summary
//! `meta/worktrees`, rebuilt from the service's last scan, never read back from storage.

use argus_core::{Coll, DocChange};
use argus_worktrees::{Derived, Service as WorktreeService};
use serde_json::{Value, json, to_value};

use crate::Services;

fn changes_of(derived: &Derived) -> Vec<DocChange> {
    derived
        .docs
        .iter()
        .filter_map(|doc| {
            let data = to_value(doc).ok()?;
            Some(DocChange::set(Coll::Worktrees, doc.id.clone(), data))
        })
        .collect()
}

/// Writes the docs of an already derived picture.
pub(crate) fn publish_derived(svc: &Services, key: &str, derived: &Derived) {
    let meta = to_value(&derived.meta).unwrap_or(Value::Null);
    let docs = changes_of(derived);
    let written = svc.db.tx(|tx| {
        tx.replace(key, &[Coll::Worktrees], docs)?;
        tx.apply(key, &[DocChange::set(Coll::Meta, "worktrees", meta)])?;
        Ok(())
    });
    if let Err(e) = written {
        svc.log.warn(
            "worktree docs not written",
            json!({ "key": key, "err": e.to_string() }),
        );
    }
}

/// Judges the project's last scan now and writes the docs.
pub(crate) fn publish(svc: &Services, service: &WorktreeService, key: &str) {
    if let Some(derived) = service.derive(key) {
        publish_derived(svc, key, &derived);
    }
}
