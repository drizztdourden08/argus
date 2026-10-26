//! The fixture board builder: a valid schema 2 `board.json` with fixed times, so a seed is the
//! same on both sides of a differential run. The shape is what `store.mjs` writes (key order
//! included), so neither engine migrates or rewrites it just by reading it.

use serde_json::{Map, Value, json};

/// The stamp every seeded document carries.
pub const SEED_TIME: &str = "2026-10-01T12:00:00.000Z";

/// Stands for the work folder of a home in a seeded path; [`FixtureBoard::to_json`] swaps it.
pub const WORK_TOKEN: &str = "@WORK@";

const RANK_STEP: u64 = 1024;

/// One project board under construction.
#[derive(Clone, Debug)]
pub struct FixtureBoard {
    project: String,
    board: Map<String, Value>,
}

impl FixtureBoard {
    /// An empty board for `project`, in the key order of `emptyBoard` in `store.mjs`.
    #[must_use]
    pub fn new(project: &str) -> Self {
        let board = json!({
            "schema": 2,
            "project": project,
            "artifact": { "url": null, "publishedAt": null },
            "createdAt": SEED_TIME,
            "updatedAt": SEED_TIME,
            "counters": {},
            "threads": {}, "items": {}, "tests": {}, "entries": {}, "locks": {},
            "actions": {}, "agents": {}, "log": {}, "inbox": {}, "runDefs": {},
            "builds": {}, "launchers": {}, "dirty": {}, "versions": {},
        });
        let Value::Object(board) = board else {
            unreachable!("a json object")
        };
        FixtureBoard {
            project: project.to_owned(),
            board,
        }
    }

    /// The project key (the folder name of the board).
    #[must_use]
    pub fn project(&self) -> &str {
        &self.project
    }

    /// A registered thread, rooted in the work folder of the home, bound to `session_id`.
    #[must_use]
    pub fn thread(mut self, name: &str, role: &str, session_id: Option<&str>) -> Self {
        let rank = self.next_rank("threads");
        let doc = json!({
            "id": name, "name": name, "role": role,
            "flags": { "needsHuman": false, "blocked": false },
            "note": null, "subject": null, "brief": null,
            "registeredAt": SEED_TIME, "createdBy": name, "updatedAt": SEED_TIME,
            "updatedBy": name, "rev": 1, "rank": rank,
            "root": WORK_TOKEN, "session": null, "sessionId": session_id, "link": null,
            "status": "active", "lastSeen": SEED_TIME,
        });
        self.put("threads", name, doc);
        self
    }

    /// A work item (`I-<n>`, numbered in call order) owned by `thread`, with its criteria.
    #[must_use]
    pub fn item(mut self, title: &str, thread: &str, status: &str, criteria: &[&str]) -> Self {
        let id = self.next_id("I");
        let rank = self.next_rank("items");
        let ac: Vec<Value> = criteria
            .iter()
            .enumerate()
            .map(|(i, text)| json!({ "n": i + 1, "text": text, "done": false }))
            .collect();
        let doc = json!({
            "id": id, "createdAt": SEED_TIME, "createdBy": thread, "updatedAt": SEED_TIME,
            "updatedBy": thread, "rank": rank, "title": title, "description": "",
            "subject": "general", "thread": thread, "status": status, "origin": "thread",
            "ac": ac, "rev": 1,
        });
        self.put("items", &id, doc);
        self
    }

    /// A message waiting in the inbox of `to`, not delivered yet.
    #[must_use]
    pub fn inbox_message(mut self, to: &str, from: &str, text: &str) -> Self {
        let n = self.board["inbox"][to]
            .as_array()
            .map_or(1, |l| l.len() + 1);
        let message = json!({
            "id": format!("m-1790000000000-seed{n}"), "ts": SEED_TIME, "from": from,
            "kind": "note", "text": text, "ref": null, "deliveredAt": null,
        });
        let inbox = self.board["inbox"].as_object_mut().expect("inbox");
        let list = inbox.entry(to).or_insert_with(|| json!([]));
        list.as_array_mut().expect("inbox list").push(message);
        self
    }

    /// The board as `store.mjs` saves it (two-space JSON, a trailing newline), with the work
    /// folder of the home written where [`WORK_TOKEN`] stands.
    #[must_use]
    pub fn to_json(&self, work: &str) -> String {
        let text = serde_json::to_string_pretty(&self.board).expect("serialise");
        let escaped = work.replace('\\', "\\\\");
        format!("{}\n", text.replace(WORK_TOKEN, &escaped))
    }

    fn put(&mut self, collection: &str, id: &str, doc: Value) {
        self.board[collection][id] = doc;
        self.board["dirty"][format!("{collection}/{id}")] = json!(true);
    }

    fn next_id(&mut self, prefix: &str) -> String {
        let n = self.board["counters"][prefix].as_u64().unwrap_or(0) + 1;
        self.board["counters"][prefix] = json!(n);
        format!("{prefix}-{n}")
    }

    fn next_rank(&self, collection: &str) -> u64 {
        let count = self.board[collection].as_object().map_or(0, Map::len) as u64;
        (count + 1) * RANK_STEP
    }
}

/// The standard two-board fixture: `alpha` with a coordinator and a worker thread (each bound
/// to a session id), three items and a waiting message, and `beta` with one coordinator.
#[must_use]
pub fn standard_boards(coordinator_session: &str, worker_session: &str) -> Vec<FixtureBoard> {
    let alpha = FixtureBoard::new("alpha")
        .thread("lead", "coordinator", Some(coordinator_session))
        .thread("worker", "thread", Some(worker_session))
        .item(
            "Plan the release",
            "lead",
            "doing",
            &["Notes written", "Owner agrees"],
        )
        .item("Port the parser", "worker", "todo", &["Cases pass"])
        .item("Write the docs", "worker", "todo", &[])
        .inbox_message("worker", "lead", "Start with the parser.");
    let beta = FixtureBoard::new("beta").thread("lead", "coordinator", None);
    vec![alpha, beta]
}
