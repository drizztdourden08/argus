//! A purge with `--confirm <project key>`: back up, then remove.

use argus_board::ops::{apply_purge, count_line, purge_plan};

use super::criteria::criteria_of;
use super::{need_coordinator, total};
use crate::cli::exit::Outcome;
use crate::verbs::thread::frame::{Frame, who_of};
use crate::verbs::thread::passed::{lock_holders, say_passed};

/// The confirmed `purge ...`.
///
/// # Errors
/// Refused for a caller that is not the coordinator, criteria that do not hold, no board and a
/// backup that cannot be written.
pub fn run(frame: &mut Frame<'_>, words: &[String]) -> Outcome {
    let home = frame.home();
    let project = frame.project;
    frame.mutate(false, |board, fr| {
        let who = fr.caller(board)?;
        need_coordinator(&who, "purge")?;
        let wanted = criteria_of(board, &who, fr, words)?;
        let plan = purge_plan(board, &wanted.criteria)?;
        if total(&plan.counts) == 0.0 {
            fr.say(&["nothing matches: nothing was purged."]);
            return Ok(());
        }
        let before = lock_holders(board);
        let done = apply_purge(home, board, &plan, &who_of(&who))?;
        fr.say(&[
            &format!("purged {}.", count_line(&done.counts)),
            &format!("backup: {}", done.file),
            &format!(
                "restore with: coord restore --from {} --confirm {project}",
                done.file
            ),
        ]);
        say_passed(fr, board, &before);
        Ok(())
    })
}
