//! What a purge is asked to remove: the criteria the plan is made from, and the same purge in
//! the words the owner typed, so the dry run can print the exact command that confirms it.

use argus_board::Board;
use argus_board::json::{display, section, to_number};
use argus_board::limits::optional_text;
use argus_core::js;
use serde_json::{Map, Value, json};

use super::text_of;
use crate::cli::exit::CoordError;
use crate::verbs::thread::frame::Frame;
use crate::who::Caller;

const STATUSES: [&str; 4] = ["done", "archived", "placeholder", "dispatched"];
const WHAT: &str = "say what to purge, exactly one of: thread <name>... | --keep <name>... | --status <list> | --idle <days> | --trash [--older <days>].";

/// The criteria of a purge, and its command line.
pub struct Criteria {
    /// What `purge_plan` plans from.
    pub criteria: Value,
    /// The purge in the words the owner typed.
    pub args: String,
}

/// The threads a purge never takes: every lead and the thread making the call.
fn protected_ids(board: &Board, who: &Caller) -> Vec<String> {
    let mut ids: Vec<String> = section(board, "threads")
        .values()
        .filter(|t| t["role"] == "lead")
        .map(|t| display(&t["id"]))
        .collect();
    ids.push(who.thread.clone());
    unique(ids)
}

fn unique(list: Vec<String>) -> Vec<String> {
    let mut seen: Vec<String> = Vec::new();
    for item in list {
        if !seen.contains(&item) {
            seen.push(item);
        }
    }
    seen
}

fn known_threads(board: &Board, names: &[String]) -> Result<Vec<String>, CoordError> {
    if names.is_empty() {
        return Err(CoordError::refuse("name at least one thread."));
    }
    for name in names {
        if board["threads"][name.as_str()].is_null() {
            return Err(CoordError::refuse(format!(
                "no thread \"{name}\" on this board."
            )));
        }
    }
    Ok(names.to_vec())
}

fn days(value: Option<&Value>, name: &str) -> Result<Value, CoordError> {
    let n = to_number(value);
    if !n.is_finite() || n <= 0.0 {
        return Err(CoordError::refuse(format!(
            "{name} takes a number of days, e.g. {name} 30."
        )));
    }
    Ok(js::number(n))
}

fn by_threads(
    board: &Board,
    words: &[String],
    keep: &[String],
) -> Result<(Map<String, Value>, String), CoordError> {
    let names = known_threads(board, words.get(1..).unwrap_or_default())?;
    let refused: Vec<&str> = names
        .iter()
        .filter(|n| keep.contains(n))
        .map(String::as_str)
        .collect();
    if !refused.is_empty() {
        return Err(CoordError::refuse(format!(
            "{} cannot be purged: the lead and the calling thread are never purged.",
            refused.join(", ")
        )));
    }
    let args = format!("thread {}", names.join(" "));
    Ok((Map::from_iter([("threads".into(), json!(names))]), args))
}

fn by_keep(
    board: &Board,
    frame: &Frame<'_>,
    words: &[String],
    keep: &[String],
) -> Result<(Map<String, Value>, String), CoordError> {
    let mut typed: Vec<String> = frame.options.many("keep").iter().map(text_of).collect();
    typed.extend(words.iter().cloned());
    let names = known_threads(board, &typed)?;
    let args: Vec<String> = names.iter().map(|n| format!("--keep {n}")).collect();
    let all = unique(names.into_iter().chain(keep.iter().cloned()).collect());
    Ok((
        Map::from_iter([("keep".into(), json!(all))]),
        args.join(" "),
    ))
}

fn by_status(frame: &Frame<'_>) -> Result<(Map<String, Value>, String), CoordError> {
    let typed = frame
        .options
        .json("status")
        .map_or_else(String::new, |v| js::string(&v));
    let list: Vec<String> = typed
        .split(',')
        .map(|s| js::trim(s).to_owned())
        .filter(|s| !s.is_empty())
        .collect();
    if list.is_empty() || list.iter().any(|s| !STATUSES.contains(&s.as_str())) {
        return Err(CoordError::refuse(format!(
            "--status takes a comma list of {}.",
            STATUSES.join(", ")
        )));
    }
    let args = format!("--status {}", list.join(","));
    Ok((Map::from_iter([("status".into(), json!(list))]), args))
}

fn by_idle(frame: &Frame<'_>) -> Result<(Map<String, Value>, String), CoordError> {
    let n = days(frame.options.json("idle").as_ref(), "--idle")?;
    let args = format!("--idle {}", display(&n));
    Ok((Map::from_iter([("idleDays".into(), n)]), args))
}

fn by_trash(frame: &Frame<'_>) -> Result<(Map<String, Value>, String), CoordError> {
    let mut criteria = Map::from_iter([("trash".to_owned(), json!(true))]);
    let mut args = "--trash".to_owned();
    if let Some(older) = frame.options.json("older") {
        let n = days(Some(&older), "--older")?;
        args = format!("{args} --older {}", display(&n));
        criteria.insert("olderDays".into(), n);
    }
    Ok((criteria, args))
}

fn modes(frame: &Frame<'_>, words: &[String]) -> Vec<&'static str> {
    let on = |name: &str| frame.options.get(name).is_some();
    [
        (
            words.first().map(String::as_str) == Some("thread"),
            "thread",
        ),
        (on("keep"), "keep"),
        (on("status"), "status"),
        (on("idle"), "idle"),
        (on("trash"), "trash"),
    ]
    .into_iter()
    .filter(|(chosen, _)| *chosen)
    .map(|(_, mode)| mode)
    .collect()
}

/// The criteria and the command line of the purge the words and options ask for.
///
/// # Errors
/// Refused when not exactly one way of choosing is given, a thread is unknown or protected, a
/// status or a number of days is wrong, or the reason is over its limit.
pub fn criteria_of(
    board: &Board,
    who: &Caller,
    frame: &Frame<'_>,
    words: &[String],
) -> Result<Criteria, CoordError> {
    let chosen = modes(frame, words);
    if chosen.len() != 1 {
        return Err(CoordError::refuse(WHAT));
    }
    let reason = optional_text("reason", &frame.value("reason"))?;
    let keep = protected_ids(board, who);
    let (mut criteria, args) = match chosen[0] {
        "thread" => by_threads(board, words, &keep)?,
        "keep" => by_keep(board, frame, words, &keep)?,
        "status" => by_status(frame)?,
        "idle" => by_idle(frame)?,
        _ => by_trash(frame)?,
    };
    let tail = reason
        .as_ref()
        .map_or_else(String::new, |r| format!(" --reason \"{r}\""));
    if let Some(reason) = reason {
        criteria.insert("reason".into(), json!(reason));
    }
    Ok(Criteria {
        criteria: Value::Object(criteria),
        args: format!("{args}{tail}"),
    })
}
