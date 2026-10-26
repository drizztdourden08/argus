//! Recording a test result: the owner's run from the page and a thread's from the CLI share it.

use argus_core::clock::now_iso;
use serde_json::{Value, json};

use crate::Board;
use crate::error::BoardError;
use crate::json::{array_mut, display, doc, doc_mut, nullish, set, truthy};
use crate::store::{LogLine, append_log, stamp};
use crate::text::clip_text;

/// What a result says: the status the test gets, the note and who recorded it.
pub(crate) struct Outcome<'a> {
    pub status: &'a Value,
    pub text: &'a str,
    pub by: &'a str,
}

fn tick_covered(board: &mut Board, test: &Value, by: &str) -> Result<(), BoardError> {
    let item_id = display(&test["item"]);
    let covered: Vec<usize> = test["ac"]
        .as_array()
        .map(|a| {
            a.iter()
                .filter_map(Value::as_u64)
                .filter_map(|n| usize::try_from(n).ok())
                .collect()
        })
        .unwrap_or_default();
    let Some(item) = doc_mut(board, "items", &item_id) else {
        return Ok(());
    };
    for n in covered {
        if let Some(criterion) = n
            .checked_sub(1)
            .and_then(|i| array_mut(item, "ac").get_mut(i))
        {
            set(criterion, "done", true);
        }
    }
    stamp(board, "items", &item_id, by)
}

/// Record a result on a test: the status, the result and a run in its history. A pass ticks the
/// item's covered acceptance criteria. One line goes to the work log.
pub(crate) fn record_result(
    board: &mut Board,
    test_id: &str,
    outcome: &Outcome,
) -> Result<(), BoardError> {
    let result = json!({
        "status": outcome.status, "text": outcome.text, "by": outcome.by, "at": now_iso(),
    });
    let Some(test) = doc_mut(board, "tests", test_id) else {
        return Ok(());
    };
    set(test, "status", outcome.status.clone());
    set(test, "result", result.clone());
    array_mut(test, "runs").push(result);
    stamp(board, "tests", test_id, outcome.by)?;
    let test = doc(board, "tests", test_id).cloned().unwrap_or(Value::Null);
    if outcome.status == "pass" {
        tick_covered(board, &test, outcome.by)?;
    }
    let subject = if truthy(&test, "subject") {
        display(&test["subject"])
    } else {
        "general".to_owned()
    };
    let title = nullish(&test, "title").unwrap_or_default();
    let line = format!(
        "{test_id} {}: {}",
        display(outcome.status),
        clip_text(&title, 120)
    );
    let item = nullish(&test, "item");
    let log = LogLine {
        thread: outcome.by,
        item: item.as_deref(),
        text: &line,
    };
    append_log(board, &subject, &log);
    Ok(())
}
