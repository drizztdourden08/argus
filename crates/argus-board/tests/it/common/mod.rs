//! A fixture board and a way to post as the owner, shared by the integration tests. Each test
//! crate uses only part of it.
#![allow(dead_code, unused_imports)]

use std::sync::atomic::{AtomicUsize, Ordering};

use argus_board::Board;
use argus_board::ops::{OpResult, Role, Status, Who, apply_op};
use argus_board::store::empty_board;
use argus_board::sync::{ImportResult, export_docs, import_entries};
use argus_core::Coll;
use argus_core::clock::now_iso;
pub use argus_core::fixture::doc;
use argus_core::fixture::{fill_alpha, merge};
use serde_json::{Value, json};

static POSTS: AtomicUsize = AtomicUsize::new(0);

/// The `alpha` board of the dashboard tests: a coordinator and a `ui` thread, items with AC, an
/// auto test (pass) and a manual test (pending, build b1, handed), a question, a notice, a
/// pending action, agents working and done, a lock and a log.
pub fn alpha() -> Board {
    let mut b = empty_board("alpha");
    b["name"] = json!("Alpha");
    fill_alpha(&mut b);
    b
}

/// The page doc `id` of `coll` as the export writes it, `null` when there is none.
pub fn exported(board: &Value, coll: Coll, id: &str) -> Value {
    export_docs(board, &|_| String::new())
        .into_iter()
        .find(|d| d.coll == coll && d.id == id)
        .and_then(|d| d.data)
        .unwrap_or(Value::Null)
}

/// An owner post: `body` over the usual page fields, imported into the board.
pub fn post(board: &mut Board, body: Value) -> (String, ImportResult) {
    let n = POSTS.fetch_add(1, Ordering::SeqCst) + 1;
    let id = format!("O-{n:04}");
    let base = json!({"id": id, "createdAt": now_iso(), "author": "human", "status": "new"});
    let entry = merge(base, body);
    let id = entry["id"].as_str().unwrap_or_default().to_owned();
    let result = import_entries(board, vec![entry], &|_| false);
    (id, result)
}

/// The coordinator, who may do what a lead does.
pub fn lead() -> Who {
    Who {
        thread: Some("coordinator".to_owned()),
        role: Role::Coordinator,
        actor: None,
    }
}

/// Asserts the op was applied.
pub fn applied(res: &OpResult) {
    assert_eq!(res.status, Status::Applied, "{}", res.outcome);
}

/// Asserts the op was refused and answers the reason.
pub fn refused(res: &OpResult) -> String {
    assert_eq!(res.status, Status::Refused, "{}", res.outcome);
    res.outcome.clone()
}

/// A live `app` build named `name` that serves the item `serves` (`None` for none).
pub fn launcher(name: &str, serves: Option<&str>) -> Value {
    json!({
        "name": name, "title": name, "cmd": "node run.js", "cwd": ".", "kind": "app",
        "current": true, "retiredAt": null, "retiredBy": null, "retireNote": null,
        "createdAt": "2026-01-01T00:00:00.000Z", "updatedAt": "2026-01-01T00:00:00.000Z",
        "for": serves,
    })
}

/// A story of the `ui` thread created by `who` under a fresh epic; answers its id.
pub fn story_by(board: &mut Value, who: &Who) -> String {
    let epic = item_by(board, who, json!({"title": "Lobbies", "thread": "ui", "type": "epic"}));
    item_by(
        board,
        who,
        json!({"title": "Lobby", "thread": "ui", "type": "story", "parent": epic}),
    )
}

fn item_by(board: &mut Value, who: &Who, set: Value) -> String {
    let op = json!({"kind": "create", "target": {"c": "items"}, "set": set});
    let res = apply_op(board, &op, who, None);
    applied(&res);
    res.extra["created"]["id"]
        .as_str()
        .unwrap_or_default()
        .to_owned()
}
