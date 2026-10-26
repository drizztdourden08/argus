//! The reads of the tools: the mirrored docs, the project list, the portfolio cards and the run
//! definitions of the live board.

use std::collections::BTreeSet;

use argus_core::Coll;
use argus_core::ids::PORTFOLIO;
use serde_json::Value;

use crate::Services;
use crate::routes::admin::scopes;

/// Scopes with a `meta` doc, the mark of a board in the mirror.
fn meta_scopes(svc: &Services) -> Vec<String> {
    scopes(svc)
        .into_iter()
        .filter(|scope| !docs(svc, scope, "meta").is_empty())
        .collect()
}

/// The keys of the projects that have a board: the sync service's list and the mirror's.
pub fn project_keys(svc: &Services) -> Vec<String> {
    let mut keys: BTreeSet<String> = svc
        .sync
        .iter()
        .flat_map(|s| s.projects())
        .filter(|p| p.has_board)
        .map(|p| p.key)
        .collect();
    keys.extend(meta_scopes(svc).into_iter().filter(|s| s != PORTFOLIO));
    keys.into_iter().collect()
}

/// Every doc of a collection, in store order; none for an unknown collection or scope.
pub fn docs(svc: &Services, scope: &str, coll: &str) -> Vec<Value> {
    let Ok(coll) = coll.parse::<Coll>() else {
        return Vec::new();
    };
    svc.db
        .docs()
        .list(scope, coll)
        .map(|rows| rows.into_iter().map(|r| r.data).collect())
        .unwrap_or_default()
}

/// One doc.
pub fn doc(svc: &Services, scope: &str, coll: &str, id: &str) -> Option<Value> {
    let coll = coll.parse::<Coll>().ok()?;
    svc.db.docs().get(scope, coll, id).ok()?.map(|r| r.data)
}

/// The portfolio cards the sync service holds.
pub fn cards(svc: &Services) -> Vec<Value> {
    svc.sync.as_ref().map(|s| s.cards()).unwrap_or_default()
}

/// The run definitions on the live board; none when it has none, so the `launches` docs answer.
pub fn run_defs(svc: &Services, key: &str) -> Option<Vec<Value>> {
    let defs = svc.sync.as_ref()?.run_defs(key);
    (!defs.is_empty()).then_some(defs)
}
