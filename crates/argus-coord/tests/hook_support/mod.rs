//! Shared by the hook tests: the `alpha` board of the Node fixture (`makeHome`), a checkout
//! wired to it, and the hooks run in-process with their JSON on stdin, as Claude Code sends it.
#![allow(dead_code)]

use std::fs;
use std::path::{Path, PathBuf};

use argus_core::clock::{iso_from_ms, now_ms};
use serde_json::{Value, json};

use crate::common::{Home, Ran};
use argus_coord::Invocation;

mod stand_in;

#[allow(unused_imports)]
pub use stand_in::{StandIn, fixture};

/// The session ids of the fixture.
pub const LEAD: &str = "sid-lead-0001";
pub const UI: &str = "sid-ui-0002";
pub const STRANGER: &str = "sid-stranger-0003";
pub const FRESH: &str = "sid-fresh-0004";

/// A time `min` minutes ago.
pub fn ago(min: i64) -> String {
    iso_from_ms(now_ms() - min * 60_000)
}

fn thread(id: &str, role: &str, subject: &str, session: &str, sid: &str, rank: i64) -> Value {
    json!({
        "id": id, "name": id, "role": role, "status": "active", "subject": subject,
        "brief": null, "note": "working", "flags": { "needsHuman": false, "blocked": false },
        "root": null, "session": session, "lastSeen": ago(1), "activity": "working",
        "registeredAt": ago(60), "createdBy": id, "updatedAt": ago(5), "updatedBy": id,
        "rev": 1, "rank": rank, "sessionId": sid,
    })
}

fn agent(id: &str, name: &str, status: &str, started: i64, rank: i64) -> Value {
    json!({
        "id": id, "createdAt": ago(30), "createdBy": "ui", "updatedAt": ago(30),
        "updatedBy": "ui", "rank": rank, "name": name, "thread": "ui", "task": name,
        "model": null, "kind": "build", "item": null, "branch": null, "worktree": null,
        "status": status, "note": null, "result": null, "startedAt": ago(started),
        "finishedAt": null, "rev": 1,
    })
}

/// The `alpha` board: a coordinator and a `ui` thread bound to their sessions, and two agents
/// of `ui` (one working, one done) too old to be bound by a hook.
pub fn alpha_board() -> Value {
    json!({
        "schema": 2, "project": "alpha", "name": "Alpha",
        "artifact": { "url": null, "publishedAt": null },
        "createdAt": ago(120), "updatedAt": ago(5),
        "counters": { "A": 2 },
        "threads": {
            "coordinator": thread("coordinator", "coordinator", "lead", "Alpha", LEAD, 1024),
            "ui": thread("ui", "thread", "ui", "Alpha - UI", UI, 2048),
        },
        "items": {}, "tests": {}, "entries": {}, "locks": {}, "actions": {},
        "agents": {
            "A-1": agent("A-1", "builder", "working", 15, 1024),
            "A-2": agent("A-2", "checker", "done", 40, 2048),
        },
        "log": {}, "inbox": {}, "runDefs": {}, "dirty": {}, "versions": {},
    })
}

/// A temp home with the `alpha` board and a checkout wired to it.
pub struct Fixture {
    pub home: Home,
    pub cwd: PathBuf,
}

impl Fixture {
    pub fn new() -> Self {
        let home = Home::new();
        home.write_board("alpha", &alpha_board());
        let config = home.dashboard_home().join("config.json");
        fs::write(config, r#"{"port":51235}"#).expect("config");
        let cwd = home.folder("checkout").join("alpha");
        wire(&cwd, "alpha");
        Fixture { home, cwd }
    }

    pub fn board_file(&self) -> PathBuf {
        self.home.coord_home().join("alpha").join("board.json")
    }

    pub fn board(&self) -> Value {
        serde_json::from_str(&self.raw()).expect("board json")
    }

    pub fn raw(&self) -> String {
        fs::read_to_string(self.board_file()).expect("board file")
    }

    pub fn save(&self, board: &Value) {
        self.home.write_board("alpha", board);
    }

    /// Runs `coord hook <event>` with `input` on stdin.
    pub fn hook(&self, event: &str, input: &Value) -> Ran {
        hook_in(&self.home, event, input, &self.cwd)
    }

    /// The agents whose `hookId` is `id`.
    pub fn by_hook(&self, id: &str) -> Option<Value> {
        let board = self.board();
        let agents = board["agents"].as_object().expect("agents").clone();
        agents.into_values().find(|a| a["hookId"] == id)
    }

    /// The agents the hooks made.
    pub fn hook_agents(&self) -> Vec<Value> {
        let board = self.board();
        let agents = board["agents"].as_object().expect("agents").clone();
        agents
            .into_values()
            .filter(|a| a["source"] == "hook")
            .collect()
    }

    /// The `cwd` the hook input carries.
    pub fn cwd_text(&self) -> String {
        self.cwd.to_string_lossy().into_owned()
    }

    /// `PreToolUse` on a tool with this input.
    pub fn dispatch_tool(&self, sid: &str, tool: &str, input: Value, tool_use_id: &str) -> Ran {
        let mut body = json!({
            "session_id": sid, "cwd": self.cwd_text(), "hook_event_name": "PreToolUse",
            "tool_name": tool, "tool_use_id": tool_use_id,
        });
        body["tool_input"] = input;
        self.hook("agent-dispatch", &body)
    }

    /// `PreToolUse` on the Agent tool.
    pub fn dispatch(&self, sid: &str, input: Value, tool_use_id: &str) -> Ran {
        self.dispatch_tool(sid, "Agent", input, tool_use_id)
    }

    /// `SubagentStart`.
    pub fn started(&self, sid: &str, agent_id: &str, agent_type: &str) -> Ran {
        let body = json!({
            "session_id": sid, "cwd": self.cwd_text(), "hook_event_name": "SubagentStart",
            "agent_id": agent_id, "agent_type": agent_type,
        });
        self.hook("subagent-start", &body)
    }

    /// `SubagentStop`, with `extra` fields (`last_assistant_message`, `agent_transcript_path`).
    pub fn stopped(&self, sid: &str, agent_id: &str, extra: &Value) -> Ran {
        let mut body = json!({
            "session_id": sid, "cwd": self.cwd_text(), "hook_event_name": "SubagentStop",
            "stop_hook_active": false, "agent_id": agent_id, "agent_type": "general-purpose",
        });
        for (k, v) in extra.as_object().into_iter().flatten() {
            body[k] = v.clone();
        }
        self.hook("subagent-stop", &body)
    }

    /// `PostToolUse` of an Agent call with this tool response.
    pub fn launched(&self, sid: &str, tool_use_id: &str, response: Value) -> Ran {
        let mut body = json!({
            "session_id": sid, "cwd": self.cwd_text(), "hook_event_name": "PostToolUse",
            "tool_name": "Agent", "tool_use_id": tool_use_id,
        });
        body["tool_response"] = response;
        self.hook("agent-result", &body)
    }

    /// Puts a record on the board as `coord agent start` makes it: working, started now.
    pub fn manual(&self, id: &str, name: &str, task: &str, extra: &Value) -> String {
        let mut board = self.board();
        let mut record = json!({
            "id": id, "createdAt": ago(0), "createdBy": "coordinator", "updatedAt": ago(0),
            "updatedBy": "coordinator", "rank": 9000, "name": name, "thread": "coordinator",
            "task": task, "model": "sonnet", "modelWhy": "sonnet: most coding, one module",
            "kind": "build", "item": null, "branch": null, "worktree": null,
            "status": "working", "note": null, "result": null, "startedAt": ago(0),
            "finishedAt": null, "rev": 1,
        });
        for (k, v) in extra.as_object().into_iter().flatten() {
            record[k] = v.clone();
        }
        board["agents"][id] = record;
        self.save(&board);
        id.to_owned()
    }
}

/// Writes `.ai-config.json` naming `project` in `dir`.
pub fn wire(dir: &Path, project: &str) {
    fs::create_dir_all(dir).expect("checkout");
    let text = format!(r#"{{"projectKey":"{project}"}}"#);
    fs::write(dir.join(".ai-config.json"), text).expect("wire");
}

/// Runs `coord hook <event>` in-process in `cwd` with `input` on stdin.
pub fn hook_in(home: &Home, event: &str, input: &Value, cwd: &Path) -> Ran {
    hook_with(home.env(), event, &input.to_string(), cwd)
}

/// Runs `coord hook <event>` in-process with a raw stdin and a given environment.
pub fn hook_with(env: argus_coord::Env, event: &str, stdin: &str, cwd: &Path) -> Ran {
    let (mut inv, out, err) = Invocation::captured(&["hook", event], env, cwd.to_path_buf(), stdin);
    let code = argus_coord::run(&mut inv);
    Ran {
        code,
        out: out.text(),
        err: err.text(),
    }
}

/// A transcript with a user line and one assistant line per model.
pub fn transcript(dir: &Path, name: &str, models: &[&str]) -> PathBuf {
    let mut lines = vec![json!({ "type": "user", "message": { "role": "user", "content": "go" } })];
    lines.extend(models.iter().map(|m| {
        json!({ "type": "assistant", "message": { "role": "assistant", "model": m, "content": [{ "type": "text", "text": "ok" }] } })
    }));
    let text: Vec<String> = lines.iter().map(Value::to_string).collect();
    let file = dir.join(name);
    fs::write(&file, format!("{}\n", text.join("\n"))).expect("transcript");
    file
}
