//! Which of the owner's actions wake a thread. Pure: `wakes_for` looks at one applied owner post
//! and returns the wakes it causes; nothing is read or written.
//!
//! | owner action | wakes | reason |
//! |---|---|---|
//! | result fail, partial, blocked | the test's thread | `test-<outcome>` |
//! | followup or decision | the thread that asked, or the thread the note is about | `owner-answer`, `owner-note`, `owner-decision` |
//! | move of an item to a thread | the new thread | `item-assigned` |
//! | resume of a paused thread | that thread | `resume` |
//! | portfolio send | the project's lead | `portfolio-decision`, `portfolio-note` |

mod assign;
mod note;
mod portfolio;
mod result;
mod run;

pub use run::{RunView, failed_run_line, wake_for_run};

use crate::board::{BoardView, EntryView};
use crate::reasons::PORTFOLIO;
use crate::wake::WakeInput;

const NOT_APPLIED: [&str; 5] = ["duplicate", "cancelled", "refused", "conflict", "error"];

/// Statuses of a test that count as a failing outcome.
pub const FAILING: [&str; 3] = ["fail", "partial", "blocked"];

/// The id of a project's lead thread, when it has one.
pub type LeadOf<'a> = &'a dyn Fn(&str) -> Option<String>;

/// One applied owner post.
pub struct ApplyCtx<'a> {
    /// Project key of the board the post was applied to, or `_portfolio`.
    pub scope: &'a str,
    /// The entry as the board stored it after applying.
    pub entry: Option<&'a EntryView>,
    /// The post result: applied, pending, duplicate, cancelled, refused, conflict...
    pub status: Option<&'a str>,
    /// The project board after the post (none for a portfolio post).
    pub board: Option<&'a BoardView>,
    /// The project board just before (the last export).
    pub before: Option<&'a BoardView>,
    /// The lead thread of a project.
    pub lead_of: LeadOf<'a>,
}

/// The wakes one applied owner post causes.
#[must_use]
pub fn wakes_for(ctx: &ApplyCtx<'_>) -> Vec<WakeInput> {
    let Some(entry) = ctx.entry else {
        return Vec::new();
    };
    let status = ctx.status.unwrap_or_else(|| entry.status.as_str());
    if NOT_APPLIED.contains(&status) {
        return Vec::new();
    }
    if ctx.scope == PORTFOLIO {
        return portfolio::from_portfolio(ctx, entry);
    }
    match entry.kind.as_str() {
        "result" => result::from_result(ctx, entry),
        "followup" | "decision" => note::from_note(ctx, entry),
        "resume" => assign::from_resume(ctx, entry),
        "move" | "bulk" => assign::from_move(ctx, entry),
        _ => Vec::new(),
    }
}

/// The lead thread of a board: the first one that has not ended.
#[must_use]
pub fn lead_in(board: Option<&BoardView>) -> Option<String> {
    board.and_then(BoardView::lead)
}
