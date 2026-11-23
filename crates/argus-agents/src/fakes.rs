//! In-memory and temp-folder stand-ins for the ports, for the tests of this crate and of the
//! server that wires it. None of them touches anything outside the folder it is given.

use crate::board::{Board, put_back, records};
use crate::error::AgentsError;
use crate::iso::format_iso;
use crate::ports::{ActivityLine, ActivitySink, BoardStore, LiveDocs, WakeEvent, WakeSink};
use crate::record::Record;
use serde_json::{Map, Value};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::{Mutex, MutexGuard, PoisonError};
use std::time::{Duration, UNIX_EPOCH};

fn locked<T>(mutex: &Mutex<T>) -> MutexGuard<'_, T> {
    mutex.lock().unwrap_or_else(PoisonError::into_inner)
}

/// A wake engine that keeps what it is given.
#[derive(Debug, Default)]
pub struct RecordingWakes(Mutex<Vec<WakeEvent>>);

impl RecordingWakes {
    /// Every wake submitted so far.
    #[must_use]
    pub fn events(&self) -> Vec<WakeEvent> {
        locked(&self.0).clone()
    }
}

impl WakeSink for RecordingWakes {
    fn submit(&self, events: &[WakeEvent]) -> Result<(), AgentsError> {
        locked(&self.0).extend_from_slice(events);
        Ok(())
    }
}

/// An activity log that keeps its lines.
#[derive(Debug, Default)]
pub struct RecordingActivity(Mutex<Vec<ActivityLine>>);

impl RecordingActivity {
    /// Every line added so far.
    #[must_use]
    pub fn lines(&self) -> Vec<ActivityLine> {
        locked(&self.0).clone()
    }
}

impl ActivitySink for RecordingActivity {
    fn add(&self, line: ActivityLine) {
        locked(&self.0).push(line);
    }
}

/// The agents docs in memory, with a log of every set.
#[derive(Debug, Default)]
pub struct MemoryDocs {
    docs: Mutex<HashMap<(String, String), Record>>,
    applied: Mutex<Vec<Record>>,
}

impl MemoryDocs {
    /// Puts a doc in place without counting it as a write.
    pub fn insert(&self, project: &str, id: &str, data: Record) {
        locked(&self.docs).insert((project.to_owned(), id.to_owned()), data);
    }

    /// The data of a doc.
    #[must_use]
    pub fn doc(&self, project: &str, id: &str) -> Option<Record> {
        LiveDocs::get(self, project, id)
    }

    /// The data of every set so far, in order.
    #[must_use]
    pub fn applied(&self) -> Vec<Record> {
        locked(&self.applied).clone()
    }
}

impl LiveDocs for MemoryDocs {
    fn get(&self, project: &str, id: &str) -> Option<Record> {
        locked(&self.docs)
            .get(&(project.to_owned(), id.to_owned()))
            .cloned()
    }

    fn set(&self, project: &str, id: &str, data: Record) -> Result<(), AgentsError> {
        locked(&self.applied).push(data.clone());
        self.insert(project, id, data);
        Ok(())
    }
}

/// Boards kept as `<home>/<key>/board.json`, written under a lock the way the coord tool does,
/// with an optional stale mirror per project for the cheap first look.
#[derive(Debug)]
pub struct JsonBoards {
    home: PathBuf,
    projects: Vec<String>,
    mirrors: Mutex<HashMap<String, Board>>,
}

impl JsonBoards {
    /// Boards under `home` for these project keys.
    #[must_use]
    pub fn new(home: &Path, projects: &[&str]) -> Self {
        Self {
            home: home.to_path_buf(),
            projects: projects.iter().map(|p| (*p).to_owned()).collect(),
            mirrors: Mutex::new(HashMap::new()),
        }
    }

    /// The board file of a project.
    #[must_use]
    pub fn path(&self, project: &str) -> PathBuf {
        self.home.join(project).join("board.json")
    }

    /// Writes a board file.
    ///
    /// # Panics
    /// The folder or the file cannot be written.
    pub fn save(&self, project: &str, board: &Value) {
        let path = self.path(project);
        fs::create_dir_all(path.parent().expect("board folder")).expect("create board folder");
        let text = serde_json::to_string_pretty(board).expect("board json");
        fs::write(path, text).expect("write board file");
    }

    /// Reads a board file.
    ///
    /// # Panics
    /// The file is missing or not JSON.
    #[must_use]
    pub fn load(&self, project: &str) -> Value {
        let text = fs::read_to_string(self.path(project)).expect("read board file");
        serde_json::from_str(&text).expect("board json")
    }

    /// Makes `snapshot` answer with this board instead of the file.
    pub fn set_mirror(&self, project: &str, board: Board) {
        locked(&self.mirrors).insert(project.to_owned(), board);
    }

    fn read_board(&self, project: &str) -> Option<(Value, Board)> {
        let text = fs::read_to_string(self.path(project)).ok()?;
        let root: Value = serde_json::from_str(&text).ok()?;
        let board = Board {
            agents: records(&root, "agents"),
            threads: records(&root, "threads"),
            items: records(&root, "items"),
        };
        Some((root, board))
    }
}

impl BoardStore for JsonBoards {
    fn projects(&self) -> Vec<String> {
        self.projects.clone()
    }

    fn snapshot(&self, project: &str) -> Option<Board> {
        if let Some(mirror) = locked(&self.mirrors).get(project) {
            return Some(mirror.clone());
        }
        self.read_board(project).map(|(_, board)| board)
    }

    fn write(&self, project: &str, change: &mut dyn FnMut(&mut Board)) -> Result<(), AgentsError> {
        let fail = |why: &str| AgentsError::Board(project.to_owned(), why.to_owned());
        let (mut root, mut board) = self.read_board(project).ok_or_else(|| fail("unreadable"))?;
        change(&mut board);
        put_back(&mut root, "agents", &board.agents);
        put_back(&mut root, "threads", &board.threads);
        put_back(&mut root, "items", &board.items);
        self.save(project, &root);
        Ok(())
    }
}

/// An agent record as the board keeps it, started an hour before `now_ms`; `fields` win over the defaults.
///
/// # Panics
/// `fields` is not a JSON object.
#[must_use]
pub fn agent_record(id: &str, now_ms: i64, fields: &Value) -> Value {
    let hour_ago = format_iso(now_ms - 60 * 60_000);
    let number: i64 = id.get(2..).and_then(|n| n.parse().ok()).unwrap_or(0);
    let mut record = serde_json::json!({
        "id": id, "createdAt": hour_ago, "createdBy": "coordinator", "updatedAt": hour_ago,
        "updatedBy": "coordinator", "rank": 9000 + number, "name": format!("agent-{id}"),
        "thread": "coordinator", "task": "work", "model": "sonnet", "modelWhy": null,
        "kind": "build", "item": null, "branch": null, "worktree": null, "status": "working",
        "note": null, "result": null, "startedAt": hour_ago, "finishedAt": null, "rev": 1
    });
    overlay(&mut record, fields);
    record
}

/// A thread record; `fields` win over the defaults.
///
/// # Panics
/// `fields` is not a JSON object.
#[must_use]
pub fn thread_record(id: &str, role: &str, fields: &Value) -> Value {
    let mut record = serde_json::json!({ "id": id, "name": id, "role": role, "status": "active" });
    overlay(&mut record, fields);
    record
}

/// One assistant line of a transcript, with its newline. `extra` is merged into the message.
#[must_use]
pub fn assistant_line(at_ms: i64, content: &Value, extra: &Value) -> String {
    let mut message = serde_json::json!({
        "role": "assistant", "model": "claude-sonnet-5-5", "content": content,
        "usage": { "input_tokens": 10, "output_tokens": 5, "cache_read_input_tokens": 100 }
    });
    overlay(&mut message, extra);
    let line = serde_json::json!({ "type": "assistant", "timestamp": format_iso(at_ms), "message": message });
    format!("{line}\n")
}

/// One user line of a transcript, with its newline.
#[must_use]
pub fn user_line(at_ms: i64, content: &Value) -> String {
    let message = serde_json::json!({ "role": "user", "content": content });
    let line =
        serde_json::json!({ "type": "user", "timestamp": format_iso(at_ms), "message": message });
    format!("{line}\n")
}

/// A text content block.
#[must_use]
pub fn text_block(text: &str) -> Value {
    serde_json::json!({ "type": "text", "text": text })
}

/// A tool call content block.
#[must_use]
pub fn tool_use(id: &str, name: &str, input: &Value) -> Value {
    serde_json::json!({ "type": "tool_use", "id": id, "name": name, "input": input })
}

/// A tool result content block.
#[must_use]
pub fn tool_result(id: &str) -> Value {
    serde_json::json!({ "type": "tool_result", "tool_use_id": id, "content": "ok" })
}

fn overlay(record: &mut Value, fields: &Value) {
    let target = record.as_object_mut().expect("record is an object");
    for (key, value) in fields.as_object().expect("fields are an object") {
        target.insert(key.clone(), value.clone());
    }
}

/// A board file with these threads and agents, each keyed by its id.
#[must_use]
pub fn board_json(threads: &[Value], agents: &[Value]) -> Value {
    let keyed = |docs: &[Value]| -> Value {
        let map: Map<String, Value> = docs
            .iter()
            .map(|d| (d["id"].as_str().unwrap_or("").to_owned(), d.clone()))
            .collect();
        Value::Object(map)
    };
    serde_json::json!({ "threads": keyed(threads), "agents": keyed(agents) })
}

/// Sets the last write time of a file, in epoch milliseconds.
///
/// # Panics
/// The file cannot be opened or its time cannot be set.
pub fn touch(path: &Path, ms: i64) {
    let file = fs::OpenOptions::new()
        .write(true)
        .open(path)
        .expect("open file to touch");
    let when = UNIX_EPOCH + Duration::from_millis(u64::try_from(ms).expect("time after 1970"));
    file.set_modified(when).expect("set modified time");
}
