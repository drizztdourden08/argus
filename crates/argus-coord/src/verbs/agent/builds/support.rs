//! What the build verbs share: the tag argument, who may manage a bundle, the audit line, and the
//! small readings of the board (live tests of a tag, the bundles that link a launcher).

use argus_board::Board;
use argus_board::json::{doc, map_mut, nullish, section, truthy};
use argus_board::limits::require_str;
use argus_board::ops::Role;
use argus_board::store::{LogLine, append_log};
use argus_board::text::clip_text;
use argus_core::clock::now_iso;
use serde_json::{Value, json};

use crate::cli::exit::CoordError;
use crate::verbs::agent::base::authority::manage_thread;
use crate::verbs::agent::base::opt::word;
use crate::who::Caller;

const AUDIT_LINE: usize = 120;

/// The build tag the first word names (`tagArg`).
///
/// # Errors
/// No word, or a tag that is empty or too long.
pub fn tag_arg(positional: &[String], verb: &str) -> Result<String, CoordError> {
    if positional.first().is_none_or(String::is_empty) {
        return Err(CoordError::refuse(format!(
            "name the build: coord build {verb} <tag>."
        )));
    }
    Ok(require_str("build", word(positional, 0))?)
}

/// `authorizeManage`: the coordinator, or the thread that manages the bundle (the one named by
/// `--thread`, or the record's own). A bundle with no thread is the coordinator's.
///
/// # Errors
/// A thread that does not manage the bundle.
pub fn authorize_manage(
    board: &Board,
    who: &Caller,
    thread_ids: &[Option<String>],
) -> Result<(), CoordError> {
    let mut ids: Vec<&str> = Vec::new();
    for id in thread_ids.iter().flatten().filter(|i| !i.is_empty()) {
        if !ids.contains(&id.as_str()) {
            ids.push(id);
        }
    }
    if ids.is_empty() {
        if who.role == Role::Thread {
            return Err(CoordError::refuse(format!(
                "name your thread with --thread {}: a bundle belongs to one thread, and only the coordinator manages one with none.",
                who.thread
            )));
        }
        return Ok(());
    }
    ids.into_iter()
        .try_for_each(|id| manage_thread(who, doc(board, "threads", id)))
}

/// One audit line of the board, as the actor.
pub fn audit(board: &mut Board, who: &Caller, text: &str) {
    let line = clip_text(text, AUDIT_LINE);
    let entry = LogLine {
        thread: &who.actor,
        item: None,
        text: &line,
    };
    append_log(board, "audit", &entry);
}

/// A test that counts: not in the trash, not archived.
#[must_use]
pub fn live(test: &Value) -> bool {
    !truthy(test, "deletedAt") && !truthy(test, "archivedAt")
}

/// The live tests that carry a build tag.
#[must_use]
pub fn carrying<'a>(board: &'a Board, tag: &str) -> Vec<&'a Value> {
    section(board, "tests")
        .values()
        .filter(|t| t["build"].as_str() == Some(tag) && live(t))
        .collect()
}

/// `ensureBuild`: the record of a tag, made as a draft when missing.
pub fn ensure_build(board: &mut Board, tag: &str, actor: &str) {
    let at = now_iso();
    map_mut(board, "builds")
        .entry(tag.to_owned())
        .or_insert_with(|| {
            json!({
                "tag": tag, "status": "draft", "title": null, "note": null, "launcher": null,
                "thread": null, "createdAt": at, "updatedAt": at, "createdBy": actor,
            })
        });
}

/// The thread of a bundle record, when it has one.
#[must_use]
pub fn thread_of(board: &Board, tag: &str) -> Option<String> {
    doc(board, "builds", tag).and_then(|b| nullish(b, "thread"))
}

/// A bundle's record, to change.
///
/// # Errors
/// The bundle is not on the board.
pub fn bundle_mut<'a>(board: &'a mut Board, tag: &str) -> Result<&'a mut Value, CoordError> {
    argus_board::json::doc_mut(board, "builds", tag)
        .ok_or_else(|| CoordError::refuse(format!("no bundle \"{tag}\".")))
}
