//! Boards, wakes and times the tests of the wake crate share.

use std::collections::HashSet;
use std::sync::{Arc, Mutex, PoisonError};

use serde_json::{Map, Value, json};

use crate::board::BoardView;
use crate::board::EntryView;
use crate::config::WakeConfig;
use crate::policy::RunView;
use crate::service::AppliedPost;
use crate::stalls::BusySource;
use crate::time::iso;
use crate::wake::WakeInput;
use crate::wake_policy::WakePolicy;

/// `2026-10-22T10:00:00.000Z`, where the fake clocks start.
pub const START: i64 = 1_792_663_200_000;
/// One second in milliseconds.
pub const SEC: i64 = 1000;
/// One minute in milliseconds.
pub const MIN: i64 = 60 * SEC;
/// One hour in milliseconds.
pub const HOUR: i64 = 60 * MIN;
/// One day in milliseconds.
pub const DAY: i64 = 24 * HOUR;

/// A board from its JSON.
#[must_use]
pub fn board(value: &Value) -> BoardView {
    BoardView::from_json(value.clone())
}

/// A wake for project `alpha` with reason `test-fail` about `T-1`.
#[must_use]
pub fn wake(thread: &str, text: &str) -> WakeInput {
    WakeInput::about("alpha", thread, "test-fail", "T-1", text)
}

/// A wake with a project and a reason of its own.
#[must_use]
pub fn wake_in(scope: &str, thread: &str, text: &str, reason: &str) -> WakeInput {
    WakeInput::about(scope, thread, reason, "T-1", text)
}

/// A thread record the way the registry tests have them: active, with a session, last seen ten minutes before `START`.
#[must_use]
pub fn thread(id: &str, role: &str, extra: &Value) -> Value {
    let mut record = json!({
        "id": id, "role": role, "status": "active", "session": format!("{id} session"),
        "lastSeen": iso(START - 10 * MIN), "activity": "idle",
    });
    merge(&mut record, extra);
    record
}

/// What blocks a work item, as pairs of item fields and board entries: a blocker text, a dependency wait, a blocked
/// request and an open ask, for the item `item_id`.
#[must_use]
pub fn blocking_causes(item_id: &str) -> Vec<(Value, Value)> {
    let ask = |kind: &str, status: &str| json!({"E-1": {"id": "E-1", "item": item_id, "kind": kind, "status": status}});
    vec![
        (json!({"blocker": "needs the schema"}), json!({})),
        (json!({"waitsOn": {"state": "open"}}), json!({})),
        (json!({"requestedBy": {"state": "blocked"}}), json!({})),
        (json!({}), ask("question", "open")),
        (json!({}), ask("decision", "pending")),
    ]
}

/// Board records keyed by their `id` field.
#[must_use]
pub fn by_id(records: &[Value]) -> Map<String, Value> {
    records
        .iter()
        .map(|record| {
            (
                record["id"].as_str().unwrap_or_default().to_owned(),
                record.clone(),
            )
        })
        .collect()
}

/// A last-wake lookup a test can swap for another one.
pub type SentLookup = Box<dyn Fn(&str, &str, &str, &str) -> i64>;

/// A wakes table that never sent anything.
pub fn never_sent() -> SentLookup {
    Box::new(|_, _, _, _| 0)
}

/// Put the fields of `extra` over `record`.
pub fn merge(record: &mut Value, extra: &Value) {
    if let (Some(into), Some(fields)) = (record.as_object_mut(), extra.as_object()) {
        for (key, value) in fields {
            into.insert(key.clone(), value.clone());
        }
    }
}

/// The two boards of the registry tests: project `alpha` (a lead, `ui`, `api`, a done thread, a placeholder) and
/// `portfolio`, every live thread last seen `seen_ago_ms` before `START`.
#[must_use]
pub fn registry_boards(seen_ago_ms: i64) -> [(&'static str, BoardView); 2] {
    let seen = json!({"lastSeen": iso(START - seen_ago_ms)});
    [
        (
            "alpha",
            board(&json!({
                "threads": {
                    "lead": thread("lead", "coordinator", &seen),
                    "ui": thread("ui", "thread", &seen),
                    "api": thread("api", "thread", &seen),
                    "old": thread("old", "thread", &json!({"status": "done"})),
                    "ghost": thread("ghost", "thread", &json!({"placeholder": true})),
                },
                "inbox": {},
            })),
        ),
        (
            "portfolio",
            board(
                &json!({"threads": {"coordinator": thread("coordinator", "coordinator", &seen)}, "inbox": {}}),
            ),
        ),
    ]
}

/// The registry tests' settings: windows of 0, no debounce, a delivered wake faults after 120 s, a
/// queued one after 60 s.
#[must_use]
pub fn registry_config() -> WakeConfig {
    let mut config = WakeConfig {
        window_ms: 0,
        policy: WakePolicy::off(),
        ..WakeConfig::default()
    };
    config.registry.reaction_ms = 120 * SEC;
    config.registry.no_listener_ms = 60 * SEC;
    config.registry.restart_grace_ms = 0;
    config
}

/// The presence tests' settings: the registry's, plus a board grace of 30 s and a deaf grace of 60 s.
#[must_use]
pub fn presence_config() -> WakeConfig {
    let mut config = registry_config();
    config.registry.board_grace_ms = 30 * SEC;
    config.registry.alive_ms = 10 * MIN;
    config.registry.deaf_grace_ms = 60 * SEC;
    config
}

/// An owner post the importer applied to project `scope`, with the board after it and the board just before.
///
/// # Panics
/// When `entry` is not an entry.
#[must_use]
pub fn applied(scope: &str, entry: &Value, after: &Value, before: &Value) -> AppliedPost {
    AppliedPost {
        scope: scope.to_owned(),
        entry: Some(serde_json::from_value::<EntryView>(entry.clone()).expect("entry")),
        status: Some("applied".to_owned()),
        board: Some(Arc::new(board(after))),
        before: Some(Arc::new(board(before))),
    }
}

/// A run of project `alpha` that failed with exit code 1 and a coloured last line.
#[must_use]
pub fn failed_run() -> RunView {
    RunView {
        id: "R-6".to_owned(),
        scope: "alpha".to_owned(),
        name: "launcher-master".to_owned(),
        status: "failed".to_owned(),
        exit_code: Some(1),
        error: None,
        last_line: Some("\u{1b}[31mNode.js v24.8.0\u{1b}[0m".to_owned()),
        orphan: false,
    }
}

/// The projects that have a running process or run, in memory. Clones share their data.
#[derive(Clone, Default)]
pub struct MemoryBusy {
    inner: Arc<Mutex<HashSet<String>>>,
}

impl MemoryBusy {
    /// No project is busy.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Say whether a project has work running.
    pub fn set(&self, project: &str, busy: bool) {
        let mut all = self.inner.lock().unwrap_or_else(PoisonError::into_inner);
        if busy {
            all.insert(project.to_owned());
        } else {
            all.remove(project);
        }
    }
}

impl BusySource for MemoryBusy {
    fn busy(&self, project: &str) -> bool {
        self.inner
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .contains(project)
    }
}
