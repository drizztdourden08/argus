//! `coord worktree keep | done`: the Overlord's exception list (Argus keeps it, the verb POSTs), and
//! the answer to a request of the owner (the Delete or Check button of the Worktrees tab).

use argus_board::Board;
use argus_board::json::map_mut;
use argus_board::limits::require_text_max;
use argus_core::clock::now_iso;
use argus_core::js;
use serde_json::{Map, Value, json};

use super::opts::text_opt;
use super::remove::{id_arg, slow};
use super::rule_add::audit;
use crate::cli::Options;
use crate::cli::exit::{CoordError, Outcome};
use crate::cli::route::Call;
use crate::verbs::remote::support::link::{connect, project_of};
use crate::verbs::remote::support::scope::{as_caller, require_board};
use crate::verbs::remote::support::text::cell;
use crate::who::{Caller, caller, find_project};

const NOTE_MAX: usize = 300;
const KEEP_USAGE: &str = "name the worktree: coord worktree keep <id|path> --note \"why\" [--until <date>] [--asked-by <thread>] [--clear] [--project p].";

/// The thread that makes the call and the project it belongs to: its own project, which is not
/// the `--project` of the call when the Overlord acts on another board.
pub(super) fn actor_of(call: &Call<'_>) -> Result<(String, String), CoordError> {
    let env = &call.inv.env;
    let own = find_project(env, &call.inv.cwd, &Options::default(), env.session_id())
        .unwrap_or_else(|_| project_of(call).to_owned());
    let board = require_board(call, &own)?;
    let who = caller(&board, &call.inv.cwd, env, &call.options)?;
    Ok((who.actor, own))
}

/// An ISO date stays text, a run of digits is milliseconds.
fn until_of(text: &str) -> Value {
    match text.parse::<i64>() {
        Ok(ms) => json!(ms),
        Err(_) => json!(text),
    }
}

/// `coord worktree keep <id|path> --note "why" [--until <date>] [--asked-by <thread>] [--clear]`:
/// POSTs `/api/p/<p>/worktrees/keep`; from a thread that is not the Overlord, Argus files a pending keep (24 h) and asks him.
pub(super) fn keep(call: &mut Call<'_>) -> Outcome {
    if call.options.get("days").is_some() {
        return Err(CoordError::refuse(
            "--days is gone: a keep lasts until --until <date>, or until --clear.",
        ));
    }
    let id = id_arg(call.positional.first(), KEEP_USAGE)?;
    let (by, by_project) = actor_of(call)?;
    let mut body = Map::new();
    body.insert("id".to_owned(), json!(id));
    body.insert("note".to_owned(), json!(text_opt(&call.options, "note")?));
    if let Some(until) = text_opt(&call.options, "until")? {
        body.insert("until".to_owned(), until_of(&until));
    }
    if let Some(asked) = text_opt(&call.options, "asked-by")? {
        body.insert("askedBy".to_owned(), json!(asked));
    }
    if call.options.flag("clear") {
        body.insert("clear".to_owned(), json!(true));
    }
    body.insert("by".to_owned(), json!(by));
    body.insert("byProject".to_owned(), json!(by_project));
    let project = project_of(call).to_owned();
    let out = connect(call, false)?
        .post(
            &format!("/api/p/{project}/worktrees/keep"),
            &Value::Object(body),
        )
        .map_err(|e| slow(e, "the keep", "coord worktree list"))?;
    call.inv.say(&[&format!("{}.", cell(out.get("text")))]);
    Ok(())
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
