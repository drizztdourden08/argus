//! `coord worktree keep | done`: snooze the stale signal of one worktree, and answer a request
//! of the owner (the Delete or Check button of the Worktrees tab).

use argus_board::Board;
use argus_board::json::map_mut;
use argus_board::limits::require_text_max;
use argus_core::clock::{iso_from_ms, now_iso, parse_ms};
use argus_core::js;
use serde_json::{Value, json};

use super::opts::text_opt;
use super::path::target_of;
use super::rule_add::audit;
use crate::cli::exit::{CoordError, Outcome};
use crate::cli::route::Call;
use crate::verbs::remote::support::scope::as_caller;
use crate::verbs::remote::support::text::{cell, int_within, number_of};
use crate::who::Caller;

const DAY_MS: i64 = 86_400_000;
const DEFAULT_KEEP_DAYS: i64 = 7;
const MAX_KEEP_DAYS: f64 = 365.0;
const NOTE_MAX: usize = 300;

/// What `keep` was asked for.
struct Keep {
    target: String,
    note: Option<String>,
    days: Option<String>,
    clear: bool,
}

/// Forgets the keeps that ended more than 30 days ago.
fn forget_old(board: &mut Board, now_ms: i64) {
    let keeps = map_mut(board, "worktreeKeeps");
    let old: Vec<String> = keeps
        .iter()
        .filter(|(_, k)| {
            parse_ms(&cell(k.get("until"))).is_some_and(|until| until < now_ms - 30 * DAY_MS)
        })
        .map(|(id, _)| id.clone())
        .collect();
    for id in old {
        keeps.shift_remove(&id);
    }
}

fn days_of(text: Option<&str>) -> Result<i64, CoordError> {
    let Some(text) = text else {
        return Ok(DEFAULT_KEEP_DAYS);
    };
    int_within(number_of(text), 1.0, MAX_KEEP_DAYS)
        .ok_or_else(|| CoordError::refuse("--days takes a whole number of days from 1 to 365."))
}

/// Snoozes (or with `clear` ends the snooze of) one worktree. Returns the record, `None` when it
/// was cleared.
fn keep_worktree(
    board: &mut Board,
    who: &Caller,
    keep: &Keep,
) -> Result<Option<Value>, CoordError> {
    let (id, path) = target_of(&keep.target)?;
    let at = now_iso();
    let now_ms = parse_ms(&at).unwrap_or_default();
    forget_old(board, now_ms);
    if keep.clear {
        if map_mut(board, "worktreeKeeps").shift_remove(&id).is_none() {
            return Err(CoordError::refuse(format!("worktree {id} is not kept.")));
        }
        audit(board, who, &format!("cleared the keep of worktree {id}"));
        return Ok(None);
    }
    let n = days_of(keep.days.as_deref())?;
    let note = json!(keep.note);
    let why = require_text_max("note", &note, NOTE_MAX)?;
    let known = board["worktreeKeeps"][id.as_str()]["path"].clone();
    let path = path.map_or(known, |p| json!(p));
    let rec = json!({
        "id": id, "path": path, "until": iso_from_ms(now_ms + n * DAY_MS),
        "note": why, "by": who.actor, "at": at,
    });
    map_mut(board, "worktreeKeeps").insert(id.clone(), rec.clone());
    audit(
        board,
        who,
        &format!("kept worktree {id} for {n} days: {why}"),
    );
    Ok(Some(rec))
}

/// `coord worktree keep <id|path> --note "why" [--days n | --clear]`.
pub(super) fn keep(call: &mut Call<'_>) -> Outcome {
    let Some(target) = call.positional.first().filter(|t| !t.is_empty()).cloned() else {
        return Err(CoordError::refuse(
            "name the worktree: coord worktree keep <id|path> --note \"why\" [--days n].",
        ));
    };
    let asked = Keep {
        target,
        note: text_opt(&call.options, "note")?,
        days: text_opt(&call.options, "days")?,
        clear: call.options.flag("clear"),
    };
    as_caller(call, |board, who, scope| {
        let line = match keep_worktree(board, who, &asked)? {
            Some(rec) => format!(
                "{}: kept for {} days (until {}): {}. The stale signal for it is off until then.",
                cell(rec.get("id")),
                asked.days.as_deref().unwrap_or("7"),
                cell(rec.get("until")).chars().take(10).collect::<String>(),
                cell(rec.get("note"))
            ),
            None => "keep cleared: the stale signal for it is on again.".to_owned(),
        };
        scope.say(&[&line]);
        Ok(())
    })
}

/// The answer of a thread to an owner request (an entry that carries `worktree`).
fn answer_request(
    board: &mut Board,
    who: &Caller,
    id: &str,
    note: Option<&str>,
) -> Result<Value, CoordError> {
    let entry = board["entries"][id].clone();
    if !js::truthy(&entry["worktree"]) {
        return Err(CoordError::refuse(format!(
            "\"{id}\" is not an owner worktree request (coord inbox shows them as [worktree delete] or [worktree check])."
        )));
    }
    let text = require_text_max("note", &json!(note), NOTE_MAX)?;
    let entry_id = cell(entry.get("id"));
    let rec = json!({ "note": text, "at": now_iso(), "by": who.actor });
    map_mut(board, "worktreeAnswers").insert(entry_id.clone(), rec);
    let kind = cell(entry["worktree"].get("kind"));
    audit(
        board,
        who,
        &format!("answered the worktree {kind} request {entry_id}"),
    );
    Ok(entry)
}

/// `coord worktree done <request id> --note "..."`.
pub(super) fn done(call: &mut Call<'_>) -> Outcome {
    let Some(id) = call.positional.first().filter(|i| !i.is_empty()).cloned() else {
        return Err(CoordError::refuse(
            "name the request: coord worktree done <request id> --note \"...\" (the id is in the inbox message).",
        ));
    };
    let note = text_opt(&call.options, "note")?;
    as_caller(call, |board, who, scope| {
        let entry = answer_request(board, who, &id, note.as_deref())?;
        let worktree = &entry["worktree"];
        let place = match worktree.get("path") {
            Some(path) if !path.is_null() => cell(Some(path)),
            _ => cell(worktree.get("id")),
        };
        scope.say(&[&format!(
            "{}: answered. The owner sees it on the row of {place}.",
            cell(entry.get("id"))
        )]);
        Ok(())
    })
}
