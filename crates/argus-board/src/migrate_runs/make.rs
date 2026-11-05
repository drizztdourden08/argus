//! Writing the docs the migration makes: a test run and a test plan, in the shape the board writes.

use argus_core::js;
use serde_json::{Map, Value, json};

use crate::Board;
use crate::error::BoardError;
use crate::registry::registry;
use crate::store::create_doc;

const TITLE_PLAN: usize = 60;

/// A run to write.
pub(super) struct NewRun {
    pub plan: String,
    pub plan_title: String,
    pub thread: String,
    pub n: i64,
    pub stage: &'static str,
    pub assignee: Option<&'static str>,
    pub build: Option<String>,
    pub results: Vec<Value>,
    pub summary: Option<String>,
    pub started: Value,
    pub ended: Value,
    pub by: String,
    pub legacy: Map<String, Value>,
}

fn lane_of(item_type: &str, stage: &str) -> String {
    let found = registry().item_type(item_type).and_then(|t| t.stage(stage));
    found.map_or_else(|| "todo".to_owned(), |s| s.lane.clone())
}

fn base(title: &str, ty: &str, stage: &str, thread: &str) -> Map<String, Value> {
    let made = json!({
        "title": title, "description": "", "ac": [], "parent": null, "type": ty,
        "stage": stage, "status": lane_of(ty, stage), "thread": thread,
    });
    made.as_object().cloned().unwrap_or_default()
}

/// Write one run of a plan. Returns its id.
pub(super) fn run(board: &mut Board, new: NewRun) -> Result<String, BoardError> {
    let title = format!("Run {}: {}", new.n, js::slice(&new.plan_title, TITLE_PLAN));
    let mut item = base(&title, "test-run", new.stage, &new.thread);
    let mut fields = Map::new();
    fields.insert("plan".to_owned(), json!(new.plan));
    if let Some(build) = new.build {
        fields.insert("build".to_owned(), json!(build));
    }
    fields.insert("results".to_owned(), Value::Array(new.results));
    if let Some(summary) = new.summary {
        fields.insert("summary".to_owned(), json!(summary));
    }
    fields.extend(new.legacy);
    item.insert("fields".to_owned(), Value::Object(fields));
    item.insert("n".to_owned(), json!(new.n));
    item.insert("startedAt".to_owned(), new.started);
    item.insert("endedAt".to_owned(), new.ended);
    if let Some(who) = new.assignee {
        item.insert("assignee".to_owned(), json!(who));
    }
    create_doc(board, "items", "I", item, &new.by)
}

/// Write a draft plan holding the given cases. Returns its id.
pub(super) fn plan(
    board: &mut Board,
    (title, thread, by): (&str, &str, &str),
    (tag, cases): (&str, &[String]),
) -> Result<String, BoardError> {
    let mut item = base(title, "test-plan", "draft", thread);
    item.insert(
        "fields".to_owned(),
        json!({"cases": cases, "legacyBundle": tag}),
    );
    create_doc(board, "items", "I", item, by)
}
