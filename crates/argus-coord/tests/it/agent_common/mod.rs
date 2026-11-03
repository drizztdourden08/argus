//! A temp home with the board `alpha` and its three threads (`coordinator`, `worker`, `other`),
//! and calls of `coord` against it with `--project alpha` and `--as <thread>`: what the Node
//! suites of the agent, build and launcher verbs set up with `board init` and `thread register`.
#![allow(dead_code)]

use std::fs;
use std::path::PathBuf;

use argus_board::store::empty_board;
use argus_core::clock::{iso_from_ms, now_iso, now_ms};
use serde_json::{Value, json};

use crate::common::{Home, Ran, coord};

/// The home and the board.
pub struct Rig {
    pub home: Home,
}

/// Which thread a call acts as.
#[derive(Clone, Copy)]
pub enum As {
    Lead,
    Worker,
    Other,
    Nobody,
}

impl As {
    fn args(self) -> &'static [&'static str] {
        match self {
            As::Lead => &["--as", "coordinator"],
            As::Worker => &["--as", "worker"],
            As::Other => &["--as", "other"],
            As::Nobody => &[],
        }
    }
}

fn thread(id: &str, role: &str, root: &str, rank: u32) -> Value {
    json!({
        "id": id, "name": id, "role": role,
        "flags": { "needsHuman": false, "blocked": false },
        "note": null, "subject": null, "brief": null,
        "registeredAt": "2026-10-01T12:00:00.000Z", "createdBy": id,
        "updatedAt": "2026-10-01T12:00:00.000Z", "updatedBy": id, "rev": 1, "rank": rank,
        "root": root, "session": null, "sessionId": null, "link": null,
        "status": "active", "lastSeen": "2026-10-01T12:00:00.000Z",
    })
}

/// The agent record in the board named `name`, or null.
pub fn agent(rig: &Rig, name: &str) -> Value {
    rig.board()["agents"]
        .as_object()
        .and_then(|m| m.values().find(|a| a["name"] == name).cloned())
        .unwrap_or(Value::Null)
}

/// A record the agent hook made: working, with no item, branch or worktree.
pub fn hook_agent(name: &str, model: &str) -> Value {
    let at = now_iso();
    json!({
        "id": "A-90", "createdAt": at, "createdBy": "coordinator", "updatedAt": at,
        "updatedBy": "coordinator", "rank": 90000, "name": name,
        "thread": "coordinator", "task": "Rebuild the pad overlay screen", "model": model,
        "modelWhy": null, "kind": "build", "item": null, "branch": null, "worktree": null,
        "status": "working", "note": null, "result": null, "startedAt": at,
        "finishedAt": null, "hookId": "hk-90", "source": "hook", "agentType": "general-purpose",
        "modelUsed": null, "transcriptFile": null, "rev": 1,
    })
}

fn item_doc(id: &str, status: &str) -> Value {
    let at = "2026-10-26T12:00:00.000Z";
    json!({
        "id": id, "title": "Login", "thread": "worker", "status": status, "rev": 1,
        "rank": 1024, "createdAt": at, "createdBy": "worker", "updatedAt": at, "updatedBy": "worker",
    })
}

impl Rig {
    pub fn new() -> Rig {
        let home = Home::new();
        let root = home.path().to_string_lossy().into_owned();
        let mut board = empty_board("alpha");
        let threads = [
            ("coordinator", "coordinator"),
            ("worker", "thread"),
            ("other", "thread"),
        ];
        for (rank, (id, role)) in (1_u32..).zip(threads) {
            board["threads"][id] = thread(id, role, &root, 1024 * rank);
        }
        let rig = Rig { home };
        rig.save(&board);
        rig
    }

    /// The project root every thread is registered from.
    pub fn root(&self) -> String {
        self.home.path().to_string_lossy().into_owned()
    }

    pub fn file(&self) -> PathBuf {
        self.home.coord_home().join("alpha").join("board.json")
    }

    pub fn board(&self) -> Value {
        serde_json::from_str(&fs::read_to_string(self.file()).expect("board file")).expect("json")
    }

    pub fn save(&self, board: &Value) {
        self.home.write_board("alpha", board);
    }

    /// Edits the board file: what a test seeds that the verbs do not make.
    pub fn edit(&self, change: impl FnOnce(&mut Value)) {
        let mut board = self.board();
        change(&mut board);
        self.save(&board);
    }

    /// A call as `who`, on the project `alpha`.
    pub fn call(&self, args: &[&str], who: As) -> Ran {
        let mut line: Vec<&str> = args.to_vec();
        line.extend(["--project", "alpha"]);
        line.extend(who.args());
        coord(&line, self.home.env(), self.home.path())
    }

    /// A call that must succeed.
    pub fn ok(&self, args: &[&str], who: As) -> Ran {
        let ran = self.call(args, who);
        assert_eq!(ran.code, 0, "{args:?}\n{}\n{}", ran.out, ran.err);
        ran
    }

    /// A call that must be refused, with a message holding `needle`.
    pub fn refused(&self, args: &[&str], who: As, needle: &str) -> Ran {
        let ran = self.call(args, who);
        assert_ne!(ran.code, 0, "{args:?} should be refused\n{}", ran.out);
        assert!(
            ran.err.contains(needle),
            "{args:?}: {:?} lacks {needle:?}",
            ran.err
        );
        ran
    }

    /// A call that must be refused, whatever the message.
    pub fn fails(&self, args: &[&str], who: As) -> Ran {
        let ran = self.call(args, who);
        assert_ne!(ran.code, 0, "{args:?} should be refused\n{}", ran.out);
        ran
    }

    /// The master launcher every project must have, made as the coordinator.
    pub fn with_master(&self) {
        self.ok(
            &["launcher", "set", "master", "--cmd", "game run"],
            As::Lead,
        );
    }

    /// A task `I-1`, unless the board has one: the item the builds of a test serve.
    pub fn with_item(&self) {
        self.edit(|b| {
            if b["items"].get("I-1").is_none() {
                b["items"]["I-1"] = item_doc("I-1", "todo");
            }
        });
    }

    /// A task with the id and status, for a build to serve (or not, when it is closed).
    pub fn with_item_status(&self, id: &str, status: &str) {
        self.edit(|b| b["items"][id] = item_doc(id, status));
    }

    /// A test plan `id` that names the bundle `tag`: the item a build of that bundle serves.
    pub fn with_plan(&self, id: &str, tag: &str) {
        self.edit(|b| {
            let at = "2026-10-26T12:00:00.000Z";
            b["items"][id] = json!({
                "id": id, "type": "test-plan", "title": "Plan", "thread": "worker", "status": "todo",
                "rev": 1, "rank": 2048, "fields": { "bundle": tag },
                "createdAt": at, "createdBy": "worker", "updatedAt": at, "updatedBy": "worker",
            });
        });
    }

    /// A launcher written before builds served items: no `for`, a usage window when given.
    pub fn legacy_launcher(&self, name: &str, use_every: Option<&str>) {
        self.edit(|b| {
            let now = iso_from_ms(now_ms());
            let mut rec = json!({
                "name": name, "title": null, "cmd": "node tool.js", "cwd": null, "kind": "app",
                "branch": null, "commit": null, "note": null, "current": false,
                "retiredAt": null, "retiredBy": null, "retireNote": null,
                "createdAt": now, "createdBy": "worker", "updatedAt": now, "by": "worker",
            });
            if let Some(every) = use_every {
                rec["useEvery"] = json!(every);
            }
            b["launchers"][name] = rec;
        });
    }

    /// An item `I-1` and a manual test `T-1` of it carrying the build tag `bundle-4`.
    pub fn with_test(&self) {
        self.edit(|b| {
            let at = "2026-10-26T12:00:00.000Z";
            b["items"]["I-1"] = json!({
                "id": "I-1", "title": "Login", "thread": "worker", "status": "todo", "rev": 1,
                "rank": 1024, "createdAt": at, "createdBy": "worker", "updatedAt": at, "updatedBy": "worker",
            });
            b["tests"]["T-1"] = json!({
                "id": "T-1", "item": "I-1", "title": "Form renders", "mode": "manual",
                "status": "pending", "build": "bundle-4", "thread": "worker", "rev": 1, "rank": 1024,
                "createdAt": at, "createdBy": "worker", "updatedAt": at, "updatedBy": "worker",
            });
        });
    }
}
