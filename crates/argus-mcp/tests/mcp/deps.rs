//! The two dependency tools: the list over the ledger docs, the cancel as the owner, and both
//! hidden and refused while dependencies are off.
// test-tag: critical

use argus_mcp::{DEP_TOOLS, MemoryBackend, RpcHost};
use serde_json::{Value, json};

use crate::support::{Rig, ids};

fn ledger() -> MemoryBackend {
    MemoryBackend::new(json!({
        "alpha": { "meta": {} },
        "_portfolio": { "deps": {
            "D-1": { "id": "D-1", "state": "open",
                "from": { "project": "alpha" }, "to": { "project": "beta" } },
            "D-2": { "id": "D-2", "state": "done",
                "from": { "project": "beta" }, "to": { "project": "gamma" } }
        } }
    }))
}

fn names(rig: &Rig) -> Vec<String> {
    rig.host
        .list_tools()
        .iter()
        .filter_map(|t| t["name"].as_str().map(str::to_owned))
        .collect()
}

#[tokio::test]
async fn list_dependencies_keeps_those_that_start_or_end_on_a_project_and_those_in_a_state() {
    let rig = &Rig::over(ledger());
    assert_eq!(
        ids(&rig.ok("list_dependencies", json!({})).await, "deps"),
        ["D-1", "D-2"]
    );
    let beta = rig
        .ok("list_dependencies", json!({ "project": "beta" }))
        .await;
    assert_eq!(ids(&beta, "deps"), ["D-1", "D-2"]);
    let alpha = rig
        .ok("list_dependencies", json!({ "project": "alpha" }))
        .await;
    assert_eq!(ids(&alpha, "deps"), ["D-1"]);
    let done = rig
        .ok("list_dependencies", json!({ "state": "done" }))
        .await;
    assert_eq!(ids(&done, "deps"), ["D-2"]);
}

#[tokio::test]
async fn cancel_dependency_asks_the_backend_with_the_id_and_the_reason() {
    let rig = &Rig::over(ledger());
    let done = rig
        .ok(
            "cancel_dependency",
            json!({ "id": "D-1", "reason": "no longer needed" }),
        )
        .await;
    assert_eq!(done["dep"]["state"], "cancelled");
    let calls = rig.backend.calls().dep_cancel;
    assert_eq!(
        calls,
        [json!({ "id": "D-1", "reason": "no longer needed" })]
    );
    rig.err("cancel_dependency", json!({ "id": "one" }), "bad_request")
        .await;
}

#[tokio::test]
async fn both_tools_are_hidden_and_refused_while_dependencies_are_off() {
    let on = Rig::over(ledger());
    assert!(DEP_TOOLS.iter().all(|t| names(&on).iter().any(|n| n == t)));
    let off = &Rig::over(ledger().without_deps());
    assert!(DEP_TOOLS.iter().all(|t| !names(off).iter().any(|n| n == t)));
    let error: Value = off.err("list_dependencies", json!({}), "not_found").await;
    assert!(
        error["message"]
            .as_str()
            .is_some_and(|m| m.starts_with("dependencies are off"))
    );
    off.err("cancel_dependency", json!({ "id": "D-1" }), "not_found")
        .await;
    assert!(off.backend.calls().dep_cancel.is_empty());
}
