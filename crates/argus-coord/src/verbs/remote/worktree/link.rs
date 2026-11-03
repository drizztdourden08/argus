//! `coord worktree link <id|path> <I-n>` and `coord worktree unlink <id|path>`: the item a
//! worktree belongs to. A worktree is removed with its item (after a short grace, the backup
//! first), so the link says what it is for. A link made by hand wins over the ones Argus works out
//! from the owning agent, the branch name and the text of items.

use argus_board::Board;
use argus_board::json::{doc, map_mut};
use argus_board::served::closure;
use argus_core::clock::now_iso;
use serde_json::json;

use super::path::target_of;
use super::rule_add::audit;
use crate::cli::exit::{CoordError, Outcome};
use crate::cli::route::Call;
use crate::verbs::remote::support::link::project_of;
use crate::verbs::remote::support::scope::locked;
use crate::verbs::remote::support::text::cell;

const LINKS: &str = "worktreeLinks";

/// The open item a worktree may be linked to.
fn check_item(board: &Board, item: &str) -> Result<(), CoordError> {
    let Some(found) = doc(board, "items", item) else {
        return Err(CoordError::refuse(format!(
            "no item \"{item}\" on this board: a worktree is linked to a work item (I-n)."
        )));
    };
    match closure(found) {
        Some(why) => Err(CoordError::refuse(format!(
            "{item} is {why}: link the worktree to an open item."
        ))),
        None => Ok(()),
    }
}

fn word(call: &Call<'_>, at: usize) -> Option<String> {
    call.positional.get(at).filter(|w| !w.is_empty()).cloned()
}

/// `coord worktree link <id|path> <I-n>`.
pub(super) fn link(call: &mut Call<'_>) -> Outcome {
    let (Some(target), Some(item)) = (word(call, 0), word(call, 1)) else {
        return Err(CoordError::refuse(
            "name the worktree and its item: coord worktree link <id|path> <I-n>.",
        ));
    };
    let project = project_of(call).to_owned();
    locked(call, &project, false, |board, scope| {
        let who = scope.begin(board)?;
        let (id, _) = target_of(&target)?;
        check_item(board, &item)?;
        let was = cell(board[LINKS][id.as_str()].get("item"));
        let rec = json!({ "item": item, "by": who.actor, "at": now_iso() });
        map_mut(board, LINKS).insert(id.clone(), rec);
        audit(board, &who, &format!("linked worktree {id} to {item}"));
        let before = if was.is_empty() {
            String::new()
        } else {
            format!(" It was linked to {was}.")
        };
        scope.say(&[&format!(
            "{id}: linked to {item}.{before} It goes with {item}: when {item} is done or dropped, Argus removes it after a short grace (the backup is written first). Link it to another open item to keep it."
        )]);
        Ok(())
    })
}

/// `coord worktree unlink <id|path>`.
pub(super) fn unlink(call: &mut Call<'_>) -> Outcome {
    let Some(target) = word(call, 0) else {
        return Err(CoordError::refuse(
            "name the worktree: coord worktree unlink <id|path>.",
        ));
    };
    let project = project_of(call).to_owned();
    locked(call, &project, false, |board, scope| {
        let who = scope.begin(board)?;
        let (id, _) = target_of(&target)?;
        let Some(gone) = map_mut(board, LINKS).shift_remove(&id) else {
            return Err(CoordError::refuse(format!(
                "worktree {id} has no link made by hand."
            )));
        };
        let item = cell(gone.get("item"));
        audit(board, &who, &format!("unlinked worktree {id} from {item}"));
        scope.say(&[&format!(
            "{id}: unlinked from {item}. Argus still reads its agent, its branch name and the text of items; with none of them it shows no item."
        )]);
        Ok(())
    })
}
