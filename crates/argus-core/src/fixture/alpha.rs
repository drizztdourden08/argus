//! The `alpha` board of the dashboard tests: a coordinator and a `ui` thread, items with AC, an
//! auto test that passed and a manual test of the handed build `b1`, a question, a notice, a
//! pending action, agents working and done, a lock and a log.

use serde_json::{Value, json};

use super::{ago, by_id, doc, test_doc, thread};

/// Fills the collections of `board` with the `alpha` project.
pub fn fill_alpha(board: &mut Value) {
    board["counters"] = json!({ "I": 2, "T": 2, "Q": 1, "N": 1, "H": 1, "A": 2, "log:ui": 2 });
    board["threads"] = json!({
        "coordinator": thread("coordinator", "coordinator", "lead", "Alpha", 1024),
        "ui": thread("ui", "thread", "ui", "Alpha - UI", 2048),
    });
    board["items"] = items();
    board["tests"] = tests();
    let (entries, actions, agents) = people();
    board["entries"] = entries;
    board["actions"] = actions;
    board["agents"] = agents;
    board["locks"] = json!({ "dev-server": { "resource": "dev-server", "description": "port 3000", "holder": "ui", "since": ago(10), "queue": [], "updatedAt": ago(10) } });
    board["log"] = json!({ "ui": { "subject": "ui", "entries": [{ "seq": 1, "ts": ago(20), "thread": "ui", "item": "I-1", "text": "form scaffolded" }, { "seq": 2, "ts": ago(10), "thread": "ui", "item": "I-1", "text": "T-1 pass: Form renders" }], "updatedAt": ago(10) } });
    board["handedBuild"] = json!("b1");
    board["handedAt"] = json!(ago(8));
}

fn items() -> Value {
    by_id(vec![
        doc(
            "I-1",
            "ui",
            json!({ "title": "Login screen", "description": "The first screen", "subject": "ui", "thread": "ui", "status": "doing", "origin": "thread", "ac": [{ "n": 1, "text": "shows a form", "done": false }, { "n": 2, "text": "rejects a bad password", "done": false }] }),
            1024,
        ),
        doc(
            "I-2",
            "ui",
            json!({ "title": "Settings page", "description": null, "subject": "ui", "thread": "ui", "status": "todo", "origin": "coordinator", "ac": [] }),
            2048,
        ),
    ])
}

fn tests() -> Value {
    let run = json!({ "status": "pass", "text": "ok", "by": "ui", "at": ago(10) });
    by_id(vec![
        test_doc(
            "T-1",
            "I-1",
            "ui",
            json!({ "title": "Form renders", "mode": "auto", "status": "pass", "ac": [1], "result": run, "runs": [run] }),
            1024,
        ),
        test_doc(
            "T-2",
            "I-1",
            "ui",
            json!({ "title": "Bad password is refused", "mode": "manual", "build": "b1", "steps": ["Open the app", "Type a wrong password"], "pass": ["an error shows"], "fail": ["it logs in"], "ac": [2] }),
            2048,
        ),
    ])
}

fn people() -> (Value, Value, Value) {
    let entries = by_id(vec![
        doc(
            "Q-1",
            "ui",
            json!({ "kind": "question", "item": "I-1", "thread": "ui", "author": "ui", "text": "Should the form remember the user?", "needs": "answer", "options": [], "status": "open", "replyTo": null, "deliveredAt": null }),
            1024,
        ),
        doc(
            "N-1",
            "ui",
            json!({ "kind": "notice", "item": null, "thread": "ui", "author": "ui", "text": "Build b1 is ready", "level": "info", "status": "open", "replyTo": null, "deliveredAt": null }),
            2048,
        ),
    ]);
    let actions = by_id(vec![doc(
        "H-1",
        "ui",
        json!({ "thread": "ui", "item": "I-1", "text": "Install the test certificate", "status": "pending", "doneAt": null, "ackedAt": null }),
        1024,
    )]);
    let agents = by_id(vec![
        doc(
            "A-1",
            "ui",
            json!({ "name": "builder", "thread": "ui", "task": "build the form", "model": null, "kind": "build", "item": "I-1", "branch": null, "worktree": null, "status": "working", "note": null, "result": null, "startedAt": ago(15), "finishedAt": null }),
            1024,
        ),
        doc(
            "A-2",
            "ui",
            json!({ "name": "checker", "thread": "ui", "task": "check the build", "model": null, "kind": "test", "item": null, "branch": null, "worktree": null, "status": "done", "note": null, "result": "all green", "startedAt": ago(40), "finishedAt": ago(35) }),
            2048,
        ),
    ]);
    (entries, actions, agents)
}
