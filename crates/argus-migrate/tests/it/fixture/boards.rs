//! The JSON of the fixture home.

use std::fs;
use std::path::Path;

use serde_json::{Map, Value, json};

use super::KEYS;

fn write_json(file: &Path, value: &Value) {
    fs::create_dir_all(file.parent().unwrap()).unwrap();
    fs::write(file, format!("{}\n", serde_json::to_string_pretty(value).unwrap())).unwrap();
}

fn other(key: &str) -> &'static str {
    match key {
        "alpha" => "beta-two",
        "beta-two" => "gamma",
        _ => "alpha",
    }
}

fn board(key: &str) -> Value {
    let peer = other(key);
    let proof = format!("see C:\\Users\\x\\.coord\\{key}\\proofs\\p1.png and /home/x/.coord/{key}/proofs/p1.png");
    let mut board = json!({
        "version": 1,
        "project": key,
        "items": {
            "I-1": {
                "id": "I-1", "createdBy": "portfolio", "assignee": format!("lead:{key}"),
                "waitsOn": [{"project": peer, "item": "I-2", "dep": "D-1"}],
                "requestedBy": {"project": peer, "dep": "D-1"},
                "text": proof
            },
            "I-2": {"id": "I-2", "waitsOn": [], "requestedBy": [{"project": peer, "dep": "D-2"}]}
        },
        "tests": {"T-1": {"id": "T-1"}},
        "entries": {"E-1": {"author": "portfolio"}},
        "threads": {"portfolio": {"id": "portfolio"}, "coordinator": {"id": "coordinator"}},
        "log": {"portfolio": ["a", "b"]},
        "counters": {"log:portfolio": 2}
    });
    if key == "alpha" {
        board["name"] = json!("Alpha Name");
    }
    board
}

fn per_key(make: impl Fn(&str) -> Value) -> Value {
    let map: Map<String, Value> = KEYS.iter().map(|k| ((*k).to_owned(), make(k))).collect();
    Value::Object(map)
}

fn portfolio() -> Value {
    json!({
        "version": 1, "project": "_portfolio", "kind": "portfolio",
        "projects": per_key(|k| {
            let mut p = json!({"key": k, "root": "X:\\r"});
            if k == "beta-two" {
                p["name"] = json!("Beta");
            }
            p
        }),
        "cards": per_key(|k| json!({"key": k, "state": "running"})),
        "pushedHash": per_key(|_| json!("abc")),
        "boards": {"BD-1": {"id": "BD-1", "key": "alpha"}, "BD-2": {"id": "BD-2", "key": "gamma"}},
        "deps": {"D-1": {"id": "D-1", "from": {"project": "alpha", "item": "I-1"},
                         "to": {"project": "beta-two", "item": "I-2"}}},
        "entries": {
            "S-1": {"author": "portfolio", "project": "alpha", "deliveredTo": ["alpha", "gamma"],
                    "text": "see C:\\Users\\x\\.coord\\gamma\\proofs\\p1.png"},
            "S-2": {"project": "all"}
        }
    })
}

/// Writes every board, the portfolio, the stamps and the two home files.
pub fn write_all(home: &Path) {
    for key in KEYS {
        let dir = home.join(key);
        fs::create_dir_all(dir.join("proofs")).unwrap();
        fs::write(dir.join("proofs").join("p1.png"), format!("png of {key}")).unwrap();
        write_json(&dir.join("board.json"), &board(key));
    }
    write_json(&home.join("_portfolio").join("board.json"), &portfolio());
    let syncs = json!({"alpha": {"at": "t", "by": "alpha"}, "gamma": {"at": "t", "by": "gamma"}});
    write_json(&home.join("remote.json"), &json!({"mode": "local", "syncs": syncs}));
    let roots = json!({"alpha": "X:\\a", "gamma": "X:\\g"});
    write_json(&home.join("dashboard").join("config.json"), &json!({"port": 1, "projectRoots": roots}));
    for name in [
        "listen-alpha-coordinator.json",
        "listen-beta-two-coordinator.json",
        "listen-beta-two-lead.json",
        "listen-portfolio-coordinator.json",
    ] {
        write_json(&home.join("dashboard").join(name), &json!({"pid": 1}));
    }
}
