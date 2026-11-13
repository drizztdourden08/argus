//! The worktree docs of a project: one `worktrees` doc per worktree on disk and the summary
//! `meta/worktrees`, rebuilt from the service's last scan, never read back from storage.

use argus_core::ids::PORTFOLIO;
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

/// The counter of the project for the overview cards: the `counts` of the portfolio doc `meta/worktrees`, by project.
fn portfolio_counter(
    tx: &argus_store::Tx,
    key: &str,
    derived: &Derived,
) -> Result<DocChange, argus_store::StoreError> {
    let mut all = tx
        .get(PORTFOLIO, Coll::Meta, "worktrees")?
        .map_or_else(|| json!({}), |row| row.data);
    all["counts"][key] = json!({
        "listed": derived.meta.listed,
        "running": derived.meta.running_agents,
    });
    Ok(DocChange::set(Coll::Meta, "worktrees", all))
}

/// Writes the docs of an already derived picture.
pub(crate) fn publish_derived(svc: &Services, key: &str, derived: &Derived) {
    let meta = to_value(&derived.meta).unwrap_or(Value::Null);
    let docs = changes_of(derived);
    let written = svc.db.tx(|tx| {
        tx.replace(key, &[Coll::Worktrees], docs)?;
        tx.apply(key, &[DocChange::set(Coll::Meta, "worktrees", meta)])?;
        let counter = portfolio_counter(tx, key, derived)?;
        tx.apply(PORTFOLIO, &[counter])?;
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

/// The folders the scratch step would remove, read-only, in the portfolio doc `meta/worktrees` under `scratch`.
pub(crate) fn publish_scratch(svc: &Services, service: &WorktreeService) {
    let plan = to_value(service.scratch_plan()).unwrap_or(Value::Null);
    let written = svc.db.tx(|tx| {
        let mut all = tx
            .get(PORTFOLIO, Coll::Meta, "worktrees")?
            .map_or_else(|| json!({}), |row| row.data);
        all["scratch"] = plan;
        tx.apply(PORTFOLIO, &[DocChange::set(Coll::Meta, "worktrees", all)])?;
        Ok(())
    });
    if let Err(e) = written {
        svc.log
            .warn("scratch list not written", json!({ "err": e.to_string() }));
    }
}
