//! The one call an item write makes: its builds follow its lane.

use argus_core::clock::now_iso;

use super::close::{close_builds, reopen_builds};
use super::state::closure;
use crate::Board;
use crate::json::doc;

/// Makes the builds of `item` agree with it: closed with it while it is done, dropped or in the
/// trash, brought back while it is open. Called from [`crate::store::stamp`], so every path that
/// changes an item (a verb, an owner post, a drag, a bulk move, a dependency closing it) is
/// covered by the same board write, and a board with no `for` anywhere is untouched.
pub fn follow(board: &mut Board, item: &str, who: &str) {
    let Some(closed) = doc(board, "items", item).map(|i| closure(i).is_some()) else {
        return;
    };
    let at = now_iso();
    if closed {
        close_builds(board, item, who, &at);
    } else {
        reopen_builds(board, item, who, &at);
    }
}
