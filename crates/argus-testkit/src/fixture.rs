//! The fixture home of the Node tests, written as plain JSON:
//! `alpha` (coordinator and `ui`, items with AC, an auto test that passed and a manual test of
//! the handed build `b1`, a question, a notice, a pending action, agents working and done, a
//! lock, a log, and the run defs `hello`, `fail`, `slow`, `app`), `beta` (coordinator and `api`,
//! one item, a manual test, an open decision request) and `_portfolio` listing both.

use std::path::Path;

use argus_core::clock::{iso_from_ms, now_ms};
use serde_json::{Map, Value, json};

/// The ISO time `min` minutes ago.
#[must_use]
pub fn ago(min: i64) -> String {
    iso_from_ms(now_ms() - min * 60_000)
}

fn merge(mut base: Value, fields: Value) -> Value {
    if let (Some(b), Value::Object(f)) = (base.as_object_mut(), fields) {
        b.extend(f);
    }
    base
}

fn doc(id: &str, by: &str, fields: Value, rank: i64) -> Value {
    let at = ago(30);
    let mut d = merge(
        json!({ "id": id, "createdAt": at, "createdBy": by, "updatedAt": at, "updatedBy": by, "rank": rank }),
        fields,
    );
    d["rev"] = json!(1);
    d
}

fn thread(id: &str, role: &str, subject: &str, session: &str, rank: i64) -> Value {
    let note = if role == "coordinator" {
        "leading".to_owned()
    } else {
        format!("working on {subject}")
    };
    json!({
        "id": id, "name": id, "role": role, "status": "active", "subject": subject, "brief": null, "note": note,
        "flags": { "needsHuman": false, "blocked": false }, "root": null, "session": session, "lastSeen": ago(1),
        "activity": "working", "registeredAt": ago(60), "createdBy": id, "updatedAt": ago(5), "updatedBy": id, "rev": 1, "rank": rank,
    })
}

fn test_doc(id: &str, item: &str, thread: &str, fields: Value, rank: i64) -> Value {
    let base = json!({
        "item": item, "thread": thread, "subject": thread, "title": null, "covers": null, "build": null, "mode": "auto", "run": null,
        "context": null, "changed": [], "setup": [], "steps": [], "pass": [], "fail": [], "notes": [], "ac": [], "status": "pending",
        "result": null, "runs": [], "proofs": [],
    });
    doc(id, thread, merge(base, fields), rank)
}

fn run_def(name: &str, cmd: &str, cwd: &str, extra: Value) -> Value {
    let base = json!({ "name": name, "cmd": cmd, "cwd": cwd, "kind": "task", "env": {}, "hand": false, "visible": false, "timeoutSec": 0, "parallel": 1, "definedBy": "coordinator", "definedAt": ago(20), "rev": 1 });
    merge(base, extra)
}

fn base(project: &str, name: &str) -> Value {
    json!({
        "schema": 2, "project": project, "name": name, "artifact": { "url": null, "publishedAt": null }, "createdAt": ago(120), "updatedAt": ago(5),
        "counters": {}, "threads": {}, "items": {}, "tests": {}, "entries": {}, "locks": {}, "actions": {}, "agents": {}, "log": {},
        "inbox": {}, "runDefs": {}, "dirty": {}, "versions": {},
    })
}

fn by_id(docs: Vec<Value>) -> Value {
    Value::Object(
        docs.into_iter()
            .map(|d| (d["id"].as_str().unwrap_or_default().to_owned(), d))
            .collect::<Map<_, _>>(),
    )
}

fn alpha_items() -> Value {
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

fn alpha_tests() -> Value {
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

fn alpha_people() -> (Value, Value, Value) {
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
    let (entries, actions, agents) = alpha_people();
    b["counters"] = json!({ "I": 2, "T": 2, "Q": 1, "N": 1, "H": 1, "A": 2, "log:ui": 2 });
    b["threads"] = json!({ "coordinator": thread("coordinator", "coordinator", "lead", "Alpha", 1024), "ui": thread("ui", "thread", "ui", "Alpha - UI", 2048) });
    b["items"] = alpha_items();
    b["tests"] = alpha_tests();
    b["entries"] = entries;
    b["actions"] = actions;
    b["agents"] = agents;
    b["locks"] = json!({ "dev-server": { "resource": "dev-server", "description": "port 3000", "holder": "ui", "since": ago(10), "queue": [], "updatedAt": ago(10) } });
    b["log"] = json!({ "ui": { "subject": "ui", "entries": [{ "seq": 1, "ts": ago(20), "thread": "ui", "item": "I-1", "text": "form scaffolded" }, { "seq": 2, "ts": ago(10), "thread": "ui", "item": "I-1", "text": "T-1 pass: Form renders" }], "updatedAt": ago(10) } });
    b["handedBuild"] = json!("b1");
    b["handedAt"] = json!(ago(8));
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
