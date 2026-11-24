//! `thread set`: a thread changes its own subject, note and status; the lead any thread,
//! plus its brief and its place in the manual order (`--before` / `--after` another thread).

use argus_board::Board;
use argus_board::json::{as_integer, display, set, shown};
use argus_board::limits::require_text;
use argus_board::ops::{Role, Status, apply_op, check_authority};
use argus_board::store::stamp;
use serde_json::{Map, Value, json};

use super::edit::status_error;
use super::{STATUSES, target};
use crate::cli::exit::{CoordError, Outcome};
use crate::cli::{Opt, Val};
use crate::verbs::thread::docs::thread_mut;
use crate::verbs::thread::frame::{Frame, who_of};

/// What the typed options ask for.
struct Input {
    subject: Option<String>,
    brief: Option<String>,
    note: Option<String>,
    status: Option<String>,
    anchor: Option<(&'static str, String)>,
    if_rev: Option<i64>,
}

fn given(frame: &Frame<'_>, name: &str) -> Result<Option<String>, CoordError> {
    if frame.options.get(name).is_none() {
        return Ok(None);
    }
    Ok(Some(require_text(name, &frame.value(name))?))
}

fn status_of(frame: &Frame<'_>) -> Result<Option<String>, CoordError> {
    match frame.options.get("status") {
        None => Ok(None),
        Some(Opt::One(Val::Text(text))) if STATUSES.contains(&text.as_str()) => {
            Ok(Some(text.clone()))
        }
        Some(_) => Err(status_error()),
    }
}

fn anchor_of(frame: &Frame<'_>) -> Option<(&'static str, String)> {
    frame
        .options
        .text("before")
        .map(|t| ("before", t.to_owned()))
        .or_else(|| frame.options.text("after").map(|t| ("after", t.to_owned())))
}

fn revision_of(frame: &Frame<'_>) -> Result<Option<i64>, CoordError> {
    let Some(value) = frame.options.json("if-rev") else {
        return Ok(None);
    };
    as_integer(Some(&value))
        .map(Some)
        .ok_or_else(|| CoordError::refuse("--if-rev takes the revision number you last saw."))
}

fn read(frame: &Frame<'_>) -> Result<Input, CoordError> {
    let input = Input {
        subject: given(frame, "subject")?,
        brief: given(frame, "brief")?,
        note: given(frame, "note")?,
        status: status_of(frame)?,
        anchor: anchor_of(frame),
        if_rev: revision_of(frame)?,
    };
    let nothing = input.subject.is_none()
        && input.brief.is_none()
        && input.note.is_none()
        && input.status.is_none()
        && input.anchor.is_none();
    if nothing {
        return Err(CoordError::refuse(
            "nothing to set: pass --subject, --brief, --note, --status, --before or --after.",
        ));
    }
    Ok(input)
}

/// The edit and move ops this call needs, with the revision check the CLI asks for: last writer
/// wins unless `--if-rev` names the revision it saw.
fn ops_of(id: &str, input: &Input, edit: &Map<String, Value>) -> Vec<Value> {
    let check = match input.if_rev {
        None => ("force", json!(true)),
        Some(rev) => ("baseRev", json!(rev)),
    };
    let target = json!({"c": "threads", "id": id});
    let mut ops = Vec::new();
    if !edit.is_empty() {
        let mut op = json!({"kind": "edit", "target": target.clone(), "set": edit});
        op[check.0] = check.1.clone();
        ops.push(op);
    }
    if let Some((place, other)) = &input.anchor {
        let to = Map::from_iter([((*place).to_owned(), json!(other))]);
        let mut op = json!({"kind": "move", "target": target, "to": to});
        op[check.0] = check.1;
        ops.push(op);
    }
    ops
}

fn run_ops(
    board: &mut Board,
    who: &argus_board::ops::Who,
    ops: &[Value],
    (id, if_rev): (&str, Option<i64>),
) -> Result<(), CoordError> {
    for op in ops {
        let result = apply_op(board, op, who, None);
        match result.status {
            Status::Applied => {}
            Status::Conflict => {
                let by = result
                    .by
                    .unwrap_or_else(|| shown(board["threads"][id].get("updatedBy")));
                let current = serde_json::to_string(&result.current.unwrap_or_else(|| json!({})))
                    .unwrap_or_default();
                return Err(CoordError::refuse(format!(
                    "{id} changed since rev {} (by {by}): {current}. Look again, then retry.",
                    if_rev.map_or_else(String::new, |r| r.to_string())
                )));
            }
            Status::Refused => return Err(CoordError::refuse(result.outcome)),
        }
    }
    Ok(())
}

fn apply(board: &mut Board, frame: &mut Frame<'_>, input: &Input) -> Outcome {
    let who = frame.identify(board)?;
    let id = target(board, frame, &who)?;
    let acting = who_of(&who);
    check_authority(&acting, "set", "threads", board["threads"].get(&id))?;
    if who.role != Role::Lead && (input.brief.is_some() || input.anchor.is_some()) {
        return Err(CoordError::refuse(
            "only the lead can change a thread brief or the thread order.",
        ));
    }
    if let Some((_, other)) = &input.anchor {
        if board["threads"][other.as_str()].is_null() {
            return Err(CoordError::refuse(format!("no thread \"{other}\".")));
        }
    }
    let mut changed: Vec<&str> = Vec::new();
    if input.note.is_some() || input.status.is_some() {
        let thread = thread_mut(board, &id)?;
        for (key, text) in [("note", &input.note), ("status", &input.status)] {
            if let Some(text) = text {
                set(thread, key, text.as_str());
                changed.push(key);
            }
        }
        stamp(board, "threads", &id, &who.actor)?;
    }
    let mut edit = Map::new();
    for (key, text) in [("subject", &input.subject), ("brief", &input.brief)] {
        if let Some(text) = text {
            edit.insert(key.to_owned(), json!(text));
            changed.push(key);
        }
    }
    run_ops(
        board,
        &acting,
        &ops_of(&id, input, &edit),
        (&id, input.if_rev),
    )?;
    if input.anchor.is_some() {
        changed.push("order");
    }
    let rev = display(&board["threads"][id.as_str()]["rev"]);
    frame.say(&[&format!("{id}: set {} (rev {rev})", changed.join(", "))]);
    Ok(())
}

/// The verb `thread set [name] ...`.
///
/// # Errors
/// Refused for a text over its limit, a status that does not exist, a bad `--if-rev`, nothing to
/// set, a thread that is not the caller's to change, a changed revision and no board.
pub fn run(frame: &mut Frame<'_>) -> Outcome {
    let input = read(frame)?;
    frame.mutate(false, |board, fr| apply(board, fr, &input))
}
