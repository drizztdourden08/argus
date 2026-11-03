//! `coord build add <tag> [flags]` and `coord build set <tag> [flags]`: make the bundle, or
//! change the fields the flags name. `add` is idempotent; `set` refuses a bundle that does not
//! exist and a call that changes nothing.

use argus_board::Board;
use argus_board::json::{doc, nullish, set};
use argus_board::ops::ensure_build;
use argus_core::clock::now_iso;
use argus_launchers::launcher::resolve_bundle;
use serde_json::Value;

use super::launch::{Applied, apply_launch};
use super::parts::{Parts, json_of, parts_from};
use super::support::{audit, authorize_manage, bundle_mut, tag_arg, thread_of};
use crate::cli::Call;
use crate::cli::exit::{CoordError, Outcome};
use crate::verbs::agent::base::write;
use crate::verbs::agent::rules::typed_board;
use crate::who::Caller;

/// The fields of the patch that differ from the record, in order.
fn changed_fields(rec: &Value, parts: &Parts) -> Vec<String> {
    parts
        .patch
        .entries()
        .into_iter()
        .filter(|(key, value)| {
            json_of(value.as_ref()) != rec.get(*key).cloned().unwrap_or(Value::Null)
        })
        .map(|(key, _)| key.to_owned())
        .collect()
}

/// Puts the flags to work on the record of `tag`: its own fields, then its launcher.
fn apply_parts(
    board: &mut Board,
    who: &Caller,
    (tag, cwd): (&str, &str),
    parts: &Parts,
) -> Result<Applied, CoordError> {
    let rec = bundle_mut(board, tag)?;
    let mut changed = changed_fields(rec, parts);
    for (key, value) in parts.patch.entries() {
        set(rec, key, json_of(value.as_ref()));
    }
    let mut applied = apply_launch(board, who, (tag, cwd), parts)?;
    changed.append(&mut applied.changed);
    applied.changed = changed;
    Ok(applied)
}

/// ` Launcher x set. Play ready (...).`: the notes, each with a capital, as sentences.
fn tail(notes: &[String]) -> String {
    if notes.is_empty() {
        return String::new();
    }
    let sentences: Vec<String> = notes
        .iter()
        .map(|n| {
            let mut chars = n.chars();
            chars
                .next()
                .map_or_else(String::new, |c| c.to_uppercase().chain(chars).collect())
        })
        .collect();
    format!(" {}.", sentences.join(". "))
}

/// `"Title", thread t, launcher l, cmd: c`: what a bundle is, for the line that adds it.
fn describe(board: &Board, tag: &str) -> Result<String, CoordError> {
    let rec = doc(board, "builds", tag).cloned().unwrap_or(Value::Null);
    let typed = typed_board(board)?;
    let starts = resolve_bundle(&typed, typed.builds.get(tag));
    let mut parts: Vec<String> = Vec::new();
    if let Some(title) = nullish(&rec, "title").filter(|t| !t.is_empty()) {
        parts.push(format!("\"{title}\""));
    }
    if let Some(thread) = nullish(&rec, "thread").filter(|t| !t.is_empty()) {
        parts.push(format!("thread {thread}"));
    }
    if let Some(launcher) = starts.launcher.filter(|l| !l.is_empty()) {
        parts.push(format!("launcher {launcher}"));
    }
    if let Some(cmd) = starts.cmd.filter(|c| !c.is_empty()) {
        parts.push(format!("cmd: {cmd}"));
    }
    Ok(parts.join(", "))
}

fn updated(tag: &str, changed: &[String]) -> String {
    if changed.is_empty() {
        format!("{tag}: unchanged.")
    } else {
        format!("{tag}: updated ({}).", changed.join(", "))
    }
}

fn stamp_and_audit(
    board: &mut Board,
    who: &Caller,
    tag: &str,
    what: &str,
) -> Result<(), CoordError> {
    set(bundle_mut(board, tag)?, "updatedAt", now_iso());
    audit(board, who, what);
    Ok(())
}

/// `coord build add`.
///
/// # Errors
/// A flag the bundle's rules refuse, a thread that may not manage the bundle.
pub fn add(call: &mut Call<'_>) -> Outcome {
    let tag = tag_arg(&call.positional, "add")?;
    write(call, |board, who, run| {
        let parts = parts_from(board, run.options)?;
        let have = doc(board, "builds", &tag).is_some();
        let own = thread_of(board, &tag);
        authorize_manage(board, who, &[own, parts.patch.thread.clone().flatten()])?;
        if !have {
            ensure_build(board, &tag, &who.actor, "draft");
        }
        let cwd = run.cwd.to_string_lossy().into_owned();
        let applied = apply_parts(board, who, (&tag, &cwd), &parts)?;
        if !have || !applied.changed.is_empty() {
            let fields = if applied.changed.is_empty() {
                String::new()
            } else {
                format!(" ({})", applied.changed.join(", "))
            };
            let word = if have { "updated" } else { "added" };
            stamp_and_audit(board, who, &tag, &format!("{word} bundle {tag}{fields}"))?;
        }
        let line = if have {
            updated(&tag, &applied.changed)
        } else {
            let what = describe(board, &tag)?;
            let shown = if what.is_empty() {
                String::new()
            } else {
                format!(" ({what})")
            };
            format!("{tag}: bundle added{shown}.")
        };
        let draft = if have {
            String::new()
        } else {
            format!(" It is a draft: no Play, cannot be handed until coord build ready {tag}.")
        };
        run.say(&format!("{line}{}{draft}", tail(&applied.notes)));
        Ok(())
    })
}

/// `coord build set`.
///
/// # Errors
/// No such bundle, nothing to change, a flag the rules refuse, a thread that may not manage it.
pub fn set_fields(call: &mut Call<'_>) -> Outcome {
    let tag = tag_arg(&call.positional, "set")?;
    write(call, |board, who, run| {
        let Some(rec) = doc(board, "builds", &tag) else {
            return Err(CoordError::refuse(format!(
                "no bundle \"{tag}\": coord build add {tag} makes it."
            )));
        };
        let own = nullish(rec, "thread");
        let parts = parts_from(board, run.options)?;
        if !parts.any() {
            return Err(CoordError::refuse(
                "nothing to change: pass --title --note --launcher --cmd --cwd --thread --window or --for (\"\" clears a field).",
            ));
        }
        authorize_manage(board, who, &[own, parts.patch.thread.clone().flatten()])?;
        let cwd = run.cwd.to_string_lossy().into_owned();
        let applied = apply_parts(board, who, (&tag, &cwd), &parts)?;
        if !applied.changed.is_empty() {
            let what = format!("updated bundle {tag} ({})", applied.changed.join(", "));
            stamp_and_audit(board, who, &tag, &what)?;
        }
        run.say(&format!(
            "{}{}",
            updated(&tag, &applied.changed),
            tail(&applied.notes)
        ));
        Ok(())
    })
}
