//! A fixture board and a way to post as the owner, shared by the integration tests. Each test
//! crate uses only part of it.
#![allow(dead_code)]

use std::sync::atomic::{AtomicUsize, Ordering};

use argus_board::Board;
use argus_board::store::empty_board;
use argus_board::sync::{ImportResult, import_entries};
use argus_core::clock::{iso_from_ms, now_iso, now_ms};
use serde_json::{Map, Value, json};

static POSTS: AtomicUsize = AtomicUsize::new(0);

/// An ISO time `min` minutes ago.
pub fn ago(min: i64) -> String {
    iso_from_ms(now_ms() - min * 60_000)
}

fn merge(base: &Value, extra: Value) -> Value {
    let mut out: Map<String, Value> = base.as_object().cloned().unwrap_or_default();
    if let Value::Object(more) = extra {
        out.extend(more);
    }
    Value::Object(out)
}

/// A board doc with its bookkeeping, then `fields`, then revision 1.
pub fn doc(id: &str, by: &str, fields: Value, rank: i64) -> Value {
    let at = ago(30);
    let base = json!({
        "id": id, "createdAt": at, "createdBy": by, "updatedAt": at, "updatedBy": by, "rank": rank,
    });
    merge(&merge(&base, fields), json!({"rev": 1}))
}

fn thread(id: &str, role: &str, subject: &str, session: &str, rank: i64) -> Value {
    json!({
        "id": id, "name": id, "role": role, "status": "active", "subject": subject, "brief": null,
        "note": "working", "flags": {"needsHuman": false, "blocked": false}, "root": null,
        "session": session, "lastSeen": ago(1), "activity": "working", "registeredAt": ago(60),
        "createdBy": id, "updatedAt": ago(5), "updatedBy": id, "rev": 1, "rank": rank,
    })
}

fn test_doc(id: &str, item: &str, owner: &str, fields: Value, rank: i64) -> Value {
    let base = json!({
        "item": item, "thread": owner, "subject": owner, "title": null, "covers": null,
        "build": null, "mode": "auto", "run": null, "context": null, "changed": [], "setup": [],
        "steps": [], "pass": [], "fail": [], "notes": [], "ac": [], "status": "pending",
        "result": null, "runs": [], "proofs": [],
    });
    doc(id, owner, merge(&base, fields), rank)
}

/// The `alpha` board of the dashboard tests: a coordinator and a `ui` thread, items with AC, an
/// auto test (pass) and a manual test (pending, build b1, handed), a question, a notice, a
/// pending action, agents working and done, a lock and a log.
pub fn alpha() -> Board {
    let mut b = empty_board("alpha");
    b["name"] = json!("Alpha");
    b["counters"] = json!({"I": 2, "T": 2, "Q": 1, "N": 1, "H": 1, "A": 2, "log:ui": 2});
    b["threads"] = json!({
        "coordinator": thread("coordinator", "coordinator", "lead", "Alpha", 1024),
        "ui": thread("ui", "thread", "ui", "Alpha - UI", 2048),
    });
    items_and_tests(&mut b);
    entries_and_actions(&mut b);
    agents_locks_log(&mut b);
    b
}

fn items_and_tests(b: &mut Board) {
    b["items"] = json!({
        "I-1": doc("I-1", "ui", json!({
            "title": "Login screen", "description": "The first screen", "subject": "ui",
            "thread": "ui", "status": "doing", "origin": "thread",
            "ac": [
                {"n": 1, "text": "shows a form", "done": false},
                {"n": 2, "text": "rejects a bad password", "done": false},
            ],
        }), 1024),
        "I-2": doc("I-2", "ui", json!({
            "title": "Settings page", "description": null, "subject": "ui", "thread": "ui",
            "status": "todo", "origin": "coordinator", "ac": [],
        }), 2048),
    });
    b["tests"] = json!({
        "T-1": test_doc("T-1", "I-1", "ui", json!({
            "title": "Form renders", "mode": "auto", "status": "pass", "ac": [1],
            "result": {"status": "pass", "text": "ok", "by": "ui", "at": ago(10)},
            "runs": [{"status": "pass", "text": "ok", "by": "ui", "at": ago(10)}],
        }), 1024),
        "T-2": test_doc("T-2", "I-1", "ui", json!({
            "title": "Bad password is refused", "mode": "manual", "build": "b1",
            "steps": ["Open the app", "Type a wrong password"], "pass": ["an error shows"],
            "fail": ["it logs in"], "ac": [2],
        }), 2048),
    });
}

fn entries_and_actions(b: &mut Board) {
    b["entries"] = json!({
        "Q-1": doc("Q-1", "ui", json!({
            "kind": "question", "item": "I-1", "thread": "ui", "author": "ui",
            "text": "Should the form remember the user?", "needs": "answer", "options": [],
            "status": "open", "replyTo": null, "deliveredAt": null,
        }), 1024),
        "N-1": doc("N-1", "ui", json!({
            "kind": "notice", "item": null, "thread": "ui", "author": "ui",
            "text": "Build b1 is ready", "level": "info", "status": "open", "replyTo": null,
            "deliveredAt": null,
        }), 2048),
    });
    b["actions"] = json!({
        "H-1": doc("H-1", "ui", json!({
            "thread": "ui", "item": "I-1", "text": "Install the test certificate",
            "status": "pending", "doneAt": null, "ackedAt": null,
        }), 1024),
    });
}

fn agents_locks_log(b: &mut Board) {
    b["agents"] = json!({
        "A-1": doc("A-1", "ui", json!({
            "name": "builder", "thread": "ui", "task": "build the form", "model": null,
            "kind": "build", "item": "I-1", "branch": null, "worktree": null, "status": "working",
            "note": null, "result": null, "startedAt": ago(15), "finishedAt": null,
        }), 1024),
        "A-2": doc("A-2", "ui", json!({
            "name": "checker", "thread": "ui", "task": "check the build", "model": null,
            "kind": "test", "item": null, "branch": null, "worktree": null, "status": "done",
            "note": null, "result": "all green", "startedAt": ago(40), "finishedAt": ago(35),
        }), 2048),
    });
    b["locks"] = json!({
        "dev-server": {
            "resource": "dev-server", "description": "port 3000", "holder": "ui", "since": ago(10),
            "queue": [], "updatedAt": ago(10),
        },
    });
    b["log"] = json!({"ui": {"subject": "ui", "updatedAt": ago(10), "entries": [
        {"seq": 1, "ts": ago(20), "thread": "ui", "item": "I-1", "text": "form scaffolded"},
        {"seq": 2, "ts": ago(10), "thread": "ui", "item": "I-1", "text": "T-1 pass: Form renders"},
    ]}});
    b["handedBuild"] = json!("b1");
    b["handedAt"] = json!(ago(8));
}

/// An owner post: `body` over the usual page fields, imported into the board.
pub fn post(board: &mut Board, body: Value) -> (String, ImportResult) {
    let n = POSTS.fetch_add(1, Ordering::SeqCst) + 1;
    let id = format!("O-{n:04}");
    let base = json!({"id": id, "createdAt": now_iso(), "author": "human", "status": "new"});
    let entry = merge(&base, body);
    let id = entry["id"].as_str().unwrap_or_default().to_owned();
    let result = import_entries(board, vec![entry], &|_| false);
    (id, result)
}
