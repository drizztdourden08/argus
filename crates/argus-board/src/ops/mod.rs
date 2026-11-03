//! What may change on the board, by whom, and how a change is made. One op is the same shape the
//! page posts as an entry: `create`, `edit`, `move`, `ac`, `archive`, `unarchive`, `delete`,
//! `restore`, `bulk`, `purge`, `build`, the test plan run ops (`plan-run-start`, `plan-run-record`, `plan-run-end`) and the pull request ops of the local remote (`pr-open`,
//! `pr-update`, `pr-merge`, `pr-close`, `pr-checks`). [`apply_op`] checks the caller, validates, checks the
//! revision the writer saw, applies, stamps, and tells the owning threads.

mod ac;
mod announce;
mod authorize;
mod backup;
mod build;
mod bulk;
mod closed_with;
mod create;
mod edit;
mod fields;
mod move_;
mod plan;
mod plan_run;
mod pr;
mod purge;
mod restore;
mod result;
mod soft;
mod target;
mod typed;
mod who;

use std::path::Path;

use serde_json::Value;

use crate::Board;
use crate::error::BoardError;
use crate::json::shown;

pub use announce::refresh_needs_human;
pub use authorize::{Check, authorize};
pub use backup::{PurgeDone, apply_purge, count_line};
pub use build::ensure_build;
pub use plan::{PurgePlan, Selection, purge_plan};
pub use pr::is_pr_kind;
pub use restore::restore_from;
pub use result::{Notice, OpResult, Status};
pub use target::{record_arg, record_thread};
pub use typed::create_note;
pub use who::{Role, Who};

fn dispatch(
    board: &mut Board,
    op: &Value,
    who: &Who,
    home: Option<&Path>,
) -> Option<Result<OpResult, BoardError>> {
    Some(match op["kind"].as_str()? {
        "create" => create::handle(board, op, who),
        "edit" => edit::handle(board, op, who),
        "move" => move_::handle(board, op, who),
        "ac" => ac::handle(board, op, who),
        "archive" | "unarchive" | "delete" | "restore" => soft::handle(board, op, who),
        "bulk" => bulk::handle(board, op, who, home),
        "purge" => purge::handle(board, op, who, home),
        "build" => build::handle(board, op, who),
        kind if plan_run::is_kind(kind) => plan_run::handle(board, op, who),
        kind if pr::is_pr_kind(kind) => pr::handle(board, op, who),
        _ => return None,
    })
}

/// Apply one op to the board as `who`. `home` is the coord home, needed only by `purge` (its
/// backup goes under it); without it a purge is refused.
///
/// Returns how it went: `applied` (with the inbox lines already queued for the owning threads, a
/// bulk's `results`, a create's `created`, a purge's `file` and `counts`), `conflict` (nothing
/// applied: the record changed since `baseRev`; `current` holds the fields that differ) or
/// `refused` (nothing applied; `outcome` says why: `gone`, `in trash`, a refusal from the ownership
/// rules, or a bad field or value).
pub fn apply_op(board: &mut Board, op: &Value, who: &Who, home: Option<&Path>) -> OpResult {
    let was = closed_with::before(board);
    let result = match dispatch(board, op, who, home) {
        Some(Ok(result)) => result,
        Some(Err(e)) => OpResult::refused(e.to_string()),
        None => OpResult::refused(format!("unknown op \"{}\"", shown(op.get("kind")))),
    };
    closed_with::tell(board, &was, result)
}

/// `authorize` of `ops.mjs`: refuses with the reason when `who` may not do `verb` on `doc`, a
/// record of `coll` (`None` for a create or a collection-wide verb).
///
/// # Errors
/// Refused with the sentence the rules give.
pub fn check_authority(
    who: &Who,
    verb: &str,
    coll: &str,
    doc: Option<&Value>,
) -> Result<(), BoardError> {
    authorize::authorize(who, &authorize::Check::new(verb, coll, doc))
}
