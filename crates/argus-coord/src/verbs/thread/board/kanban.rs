//! `board kanban show` and `board kanban set --file <json>`: the board's Kanban config.

use std::fs;

use argus_board::kanban_config::export;
use argus_board::ops::{Status, apply_op};
use serde_json::{Value, json};

use crate::cli::exit::{CoordError, Outcome};
use crate::verbs::thread::frame::{Frame, who_of};

/// The verb `board kanban show`: the config the board runs, as JSON; the default when it sets none.
///
/// # Errors
/// Refused when the project has no board.
pub fn show(frame: &mut Frame<'_>) -> Outcome {
    let board = frame.read_board()?;
    let custom = board.get("kanban").is_some_and(|k| !k.is_null());
    let config = export(&board);
    frame.json(&json!({ "custom": custom, "kanban": config }));
    Ok(())
}

fn read_file(path: &str) -> Result<Value, CoordError> {
    let text = fs::read_to_string(path)
        .map_err(|e| CoordError::refuse(format!("cannot read {path}: {e}.")))?;
    serde_json::from_str(&text)
        .map_err(|e| CoordError::refuse(format!("{path} is not valid JSON: {e}.")))
}

/// The verb `board kanban set --file <json>`: replaces the config (the owner or the board's lead).
///
/// # Errors
/// Refused when the file cannot be read, the caller may not change it or the config is bad.
pub fn set(frame: &mut Frame<'_>) -> Outcome {
    let Some(path) = frame.options.text("file").map(str::to_owned) else {
        return Err(CoordError::refuse(
            "pass the config as --file <json>; coord board kanban show prints the one in use.",
        ));
    };
    let config = read_file(&path)?;
    frame.mutate(false, |board, fr| {
        let who = fr.identify(board)?;
        let op = json!({ "kind": "kanban-config", "kanban": config });
        let result = apply_op(board, &op, &who_of(&who), None);
        match result.status {
            Status::Applied => {
                fr.say(&[&result.outcome]);
                Ok(())
            }
            _ => Err(CoordError::refuse(result.outcome)),
        }
    })
}

/// The verb `board kanban <show|set>`.
///
/// # Errors
/// Refused as the verb it runs refuses, or for an unknown word.
pub fn run(frame: &mut Frame<'_>) -> Outcome {
    let sub = frame.arg(0);
    frame.args = &frame.args[usize::from(sub.is_some())..];
    match sub {
        Some("show") => show(frame),
        Some("set") => set(frame),
        _ => Err(CoordError::refuse(
            "board kanban verb must be one of show, set.",
        )),
    }
}
