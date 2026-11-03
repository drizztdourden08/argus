//! A to-do that holds an item back and was given to the lead of another project, quiet for a day: that
//! lead hears of it once, as a notice (`todo-quiet`).

use serde_json::Value;

use argus_board::json::{doc, section, text};

use crate::board::BoardView;
use crate::policy::coordinator_in;
use crate::source::BoardSource;
use crate::text::clip;
use crate::time::ms_or_zero;
use crate::wake::WakeInput;

/// The reason of the notice.
const REASON: &str = "todo-quiet";

/// A to-do untouched for this long is quiet.
const QUIET_MS: i64 = 24 * 3_600_000;

const TITLE_MAX: usize = 60;

/// The lead a to-do names, as a project key.
fn lead_of(todo: &Value) -> Option<&str> {
    text(todo, "assignee")?.strip_prefix("lead:")
}

/// The ids of the to-dos an item is linked to.
fn todo_links(item: &Value) -> Vec<&str> {
    item["blockedBy"]
        .as_array()
        .into_iter()
        .flatten()
        .filter_map(Value::as_str)
        .filter(|id| id.starts_with("H-"))
        .collect()
}

fn is_quiet(todo: &Value, now: i64) -> bool {
    let at = ms_or_zero(text(todo, "updatedAt")).max(ms_or_zero(text(todo, "createdAt")));
    text(todo, "status") == Some("pending") && at > 0 && now - at >= QUIET_MS
}

/// The wake for a quiet to-do of the item, when its lead has a board to hear it on.
fn wake_for(
    boards: &dyn BoardSource,
    scope: &str,
    item: &Value,
    (id, todo): (&str, &Value),
) -> Option<WakeInput> {
    let key = lead_of(todo)?;
    let lead = coordinator_in(boards.board(key).as_deref())?;
    let subject = format!("{id}:{}", todo["rev"].as_i64().unwrap_or(0));
    let words = format!(
        "to-do {id} ({}) has been quiet for a day and holds {} on {scope}",
        clip(text(todo, "text").unwrap_or_default(), TITLE_MAX),
        text(item, "id").unwrap_or_default()
    );
    Some(WakeInput::about(key, &lead, REASON, &subject, words))
}

/// The quiet to-dos that hold an item of this board back, each as a wake for the lead it names.
pub(super) fn quiet_todos(
    boards: &dyn BoardSource,
    scope: &str,
    board: &BoardView,
    now: i64,
) -> Vec<WakeInput> {
    let json = board.json();
    let mut out = Vec::new();
    for item in section(json, "items").values() {
        if matches!(text(item, "status"), Some("done" | "dropped")) {
            continue;
        }
        for id in todo_links(item) {
            let todo = doc(json, "actions", id).filter(|t| is_quiet(t, now));
            out.extend(todo.and_then(|todo| wake_for(boards, scope, item, (id, todo))));
        }
    }
    out
}
