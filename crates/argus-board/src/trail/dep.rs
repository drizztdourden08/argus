//! The far end of a request between boards: the ledger of the portfolio board says where it
//! goes and whether it is still live; the mirror on the item is the fallback when the ledger has
//! no such request.

use std::str::FromStr;

use argus_core::ids::PORTFOLIO;
use serde_json::Value;

use super::Boards;
use super::follow::Spot;
use crate::blocked::{Cause, CauseKind};
use crate::deps::DepState;
use crate::json::doc;

/// The other end of a request.
pub(super) struct Far {
    /// The item at the other end; `None` while the request has none.
    pub spot: Option<Spot>,
    /// The state of the request.
    pub state: Option<String>,
    /// When the request began.
    pub since: Option<String>,
}

/// The far end of the request a cause names; `None` when the ledger says it is settled.
pub(super) fn far_end(boards: &Boards, from: &Spot, cause: &Cause) -> Option<Far> {
    let waits = cause.kind == CauseKind::DepWaits;
    let ledger = boards
        .get(PORTFOLIO)
        .and_then(|b| doc(b, "deps", &cause.id));
    if ledger.is_some_and(settled) {
        return None;
    }
    let mirror = mirror_of(boards, from, (cause.id.as_str(), waits));
    let side = ledger.map(|d| &d[if waits { "to" } else { "from" }]);
    Some(Far {
        spot: spot_of(side).or_else(|| spot_of(mirror)),
        state: text(ledger, "state").or_else(|| text(mirror, "state")),
        since: text(ledger, "openedAt")
            .or_else(|| text(ledger, "createdAt"))
            .or_else(|| text(mirror, "since")),
    })
}

/// The request is over: done, dropped, gone or cancelled.
fn settled(dep: &Value) -> bool {
    dep["state"]
        .as_str()
        .and_then(|s| DepState::from_str(s).ok())
        .is_some_and(|state| !state.is_live())
}

fn text(record: Option<&Value>, key: &str) -> Option<String> {
    record?[key]
        .as_str()
        .filter(|t| !t.is_empty())
        .map(str::to_owned)
}

fn spot_of(end: Option<&Value>) -> Option<Spot> {
    let project = text(end, "project")?;
    let item = text(end, "item")?;
    Some(Spot { project, item })
}

/// The entry of a request in the mirror field of the item: `waitsOn` when the item is the
/// requester, `requestedBy` when it is the assignee.
fn mirror_of<'a>(boards: &'a Boards, from: &Spot, (dep, waits): (&str, bool)) -> Option<&'a Value> {
    let item = doc(boards.get(&from.project)?, "items", &from.item)?;
    let field = &item[if waits { "waitsOn" } else { "requestedBy" }];
    let entries: Vec<&Value> = match field {
        Value::Array(list) => list.iter().collect(),
        Value::Object(_) => vec![field],
        _ => Vec::new(),
    };
    entries.into_iter().find(|e| e["dep"].as_str() == Some(dep))
}
