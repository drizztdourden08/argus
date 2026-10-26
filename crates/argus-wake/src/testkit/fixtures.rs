//! Boards, wakes and times the tests of the wake crate share.

use std::sync::Arc;

use serde_json::{Value, json};

use crate::board::BoardView;
use crate::board::EntryView;
use crate::config::WakeConfig;
use crate::policy::RunView;
use crate::service::AppliedPost;
use crate::time::iso;
use crate::wake::WakeInput;

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

/// The registry tests' settings: windows of 0, a delivered wake faults after 120 s, a queued one after 60 s.
#[must_use]
pub fn registry_config() -> WakeConfig {
    let mut config = WakeConfig {
        window_ms: 0,
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
