//! The fixture home of the Node tests, written as plain JSON:
//! `alpha` (coordinator and `ui`, items with AC, an auto test that passed and a manual test of
//! the handed build `b1`, a question, a notice, a pending action, agents working and done, a
//! lock, a log, and the run defs `hello`, `fail`, `slow`, `app`), `beta` (coordinator and `api`,
//! one item, a manual test, an open decision request) and `_portfolio` listing both.

use std::path::Path;

pub use argus_core::fixture::ago;
use argus_core::fixture::{by_id, doc, fill_alpha, merge, test_doc, thread};
use serde_json::{Value, json};

const COLLECTIONS: [&str; 13] = [
    "counters", "threads", "items", "tests", "entries", "locks", "actions", "agents", "log",
    "inbox", "runDefs", "dirty", "versions",
];

fn run_def(name: &str, cmd: &str, cwd: &str, extra: Value) -> Value {
    let base = json!({ "name": name, "cmd": cmd, "cwd": cwd, "kind": "task", "env": {}, "hand": false, "visible": false, "timeoutSec": 0, "parallel": 1, "definedBy": "coordinator", "definedAt": ago(20), "rev": 1 });
    merge(base, extra)
}

fn base(project: &str, name: &str) -> Value {
    let mut board = json!({
        "schema": 2, "project": project, "name": name, "artifact": { "url": null, "publishedAt": null },
        "createdAt": ago(120), "updatedAt": ago(5),
    });
    for collection in COLLECTIONS {
        board[collection] = json!({});
    }
    board
}

fn alpha_defs(home: &str) -> Value {
    let slow = "node -e \"setInterval(()=>console.log(Date.now()),200)\"";
    json!({
        "hello": run_def("hello", "node -e \"console.log('hi')\"", home, json!({})),
        "fail": run_def("fail", "node -e \"process.exit(3)\"", home, json!({})),
        "slow": run_def("slow", slow, home, json!({})),
        "app": run_def("app", slow, home, json!({ "kind": "launch", "hand": true, "visible": true })),
    })
}

/// The `alpha` board.
#[must_use]
pub fn alpha_board(home: &Path) -> Value {
    let mut b = base("alpha", "Alpha");
    fill_alpha(&mut b);
    b["runDefs"] = alpha_defs(&home.display().to_string());
    b
}

/// The `beta` board.
#[must_use]
pub fn beta_board() -> Value {
    let mut b = base("beta", "Beta");
    b["counters"] = json!({ "I": 1, "T": 1, "Q": 1 });
    b["threads"] = json!({ "coordinator": thread("coordinator", "coordinator", "lead", "Beta", 1024), "api": thread("api", "thread", "api", "Beta - API", 2048) });
    b["items"] = by_id(vec![doc(
        "I-1",
        "api",
        json!({ "title": "Health endpoint", "description": null, "subject": "api", "thread": "api", "status": "review", "origin": "thread", "ac": [{ "n": 1, "text": "answers 200", "done": true }] }),
        1024,
    )]);
    b["tests"] = by_id(vec![test_doc(
        "T-1",
        "I-1",
        "api",
        json!({ "title": "Health answers", "mode": "manual", "ac": [1] }),
        1024,
    )]);
    b["entries"] = by_id(vec![doc(
        "Q-1",
        "api",
        json!({ "kind": "question", "item": "I-1", "thread": "api", "author": "api", "text": "JSON or plain text?", "needs": "decision", "options": ["JSON", "plain text"], "status": "open", "replyTo": null, "deliveredAt": null }),
        1024,
    )]);
    b
}

/// The `_portfolio` board listing both projects.
#[must_use]
pub fn portfolio_board() -> Value {
    let mut p = base("_portfolio", "Argus");
    if let Some(o) = p.as_object_mut() {
        o.remove("schema");
    }
    p["kind"] = json!("portfolio");
    p["projects"] = json!({ "alpha": { "key": "alpha", "name": "Alpha", "root": null }, "beta": { "key": "beta", "name": "Beta", "root": null } });
    p["pushedHash"] = json!({});
    p
}

/// Writes `<home>/<key>/board.json` as two-space JSON plus a newline.
///
/// # Panics
/// When the file cannot be written (a test fixture).
pub fn write_board(home: &Path, key: &str, board: &Value) {
    let dir = home.join(key);
    std::fs::create_dir_all(&dir).expect("board folder");
    let text = serde_json::to_string_pretty(board).expect("board json") + "\n";
    std::fs::write(dir.join("board.json"), text).expect("board file");
}

/// Writes the three fixture boards under `home`.
pub fn make_home(home: &Path) {
    write_board(home, "alpha", &alpha_board(home));
    write_board(home, "beta", &beta_board());
    write_board(home, "_portfolio", &portfolio_board());
}
