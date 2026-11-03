//! Fixtures for the served-build tests: a small board, items, builds with their Play definitions,
//! and the owner's ops.

use argus_core::fixture::thread;
use serde_json::{Value, json};

use crate::ops::{OpResult, Status, Who, apply_op};
use crate::store::empty_board;

mod close;
mod link;
mod master;

const AT: &str = "2026-01-01T00:00:00.000Z";

/// An empty board with one worker thread.
pub(super) fn board() -> Value {
    let mut b = empty_board("alpha");
    b["threads"]["ui"] = thread("ui", "worker", "interface", "s-ui", 2);
    b
}

/// A doing item of a type.
pub(super) fn item(b: &mut Value, id: &str, kind: &str) {
    b["items"][id] = json!({
        "id": id, "title": format!("Item {id}"), "type": kind, "thread": "ui",
        "status": "doing", "rev": 1, "rank": 1, "createdAt": AT, "createdBy": "human",
        "updatedAt": AT, "updatedBy": "human",
    });
}

/// A live build with a one-line command and its Play definition; `serves` is its `for`.
pub(super) fn build(b: &mut Value, name: &str, serves: Option<&str>) {
    let mut rec = json!({
        "name": name, "title": name, "cmd": "node run.js", "cwd": ".", "kind": "app",
        "current": true, "retiredAt": null, "retiredBy": null, "retireNote": null,
        "createdAt": AT, "updatedAt": AT,
    });
    if let Some(item) = serves {
        rec["for"] = json!(item);
    }
    b["launchers"][name] = rec;
    let def = format!("launcher-{name}");
    b["runDefs"][def.as_str()] = json!({
        "name": def, "cmd": "node run.js", "cwd": "/p", "kind": "launch",
        "launcher": name, "rev": 1,
    });
}

/// Retires a build the way `coord launcher retire` does, and drops its Play.
pub(super) fn retire_by_hand(b: &mut Value, name: &str) {
    b["launchers"][name]["retiredAt"] = json!(AT);
    b["launchers"][name]["retiredBy"] = json!("ui");
    if let Some(defs) = b["runDefs"].as_object_mut() {
        defs.shift_remove(&format!("launcher-{name}"));
    }
}

/// An op posted as the owner.
pub(super) fn owner(b: &mut Value, op: &Value) -> OpResult {
    apply_op(b, op, &Who::human(), None)
}

/// The owner moves an item to a lane.
pub(super) fn to_lane(b: &mut Value, id: &str, lane: &str) -> OpResult {
    let op = json!({"kind": "move", "target": {"c": "items", "id": id}, "to": {"status": lane}});
    owner(b, &op)
}

/// The owner sends an item to the trash (`delete`) or brings it back (`restore`).
pub(super) fn trash(b: &mut Value, kind: &str, id: &str) -> OpResult {
    owner(
        b,
        &json!({"kind": kind, "target": {"c": "items", "id": id}}),
    )
}

/// Asserts the op was applied.
pub(super) fn applied(res: &OpResult) {
    assert_eq!(res.status, Status::Applied, "{}", res.outcome);
}

/// Whether the Play definition of a build is on the board.
pub(super) fn has_play(b: &Value, name: &str) -> bool {
    b["runDefs"][format!("launcher-{name}").as_str()]["launcher"] == name
}

/// Whether a build is retired.
pub(super) fn is_closed(b: &Value, name: &str) -> bool {
    !b["launchers"][name]["retiredAt"].is_null()
}
