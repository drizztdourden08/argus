//! A purge without `--confirm`: what it would remove, the backup it would write, and the exact
//! command that does it. Nothing is written.

use argus_board::json::display;
use argus_board::ops::purge_plan;
use argus_core::js;

use super::criteria::criteria_of;
use super::{need_coordinator, total};
use crate::cli::exit::Outcome;
use crate::cli::out::table;
use crate::verbs::thread::frame::Frame;

/// The dry run of `purge ...`.
///
/// # Errors
/// Refused for a caller that is not the coordinator, criteria that do not hold, and no board.
pub fn run(frame: &mut Frame<'_>, words: &[String]) -> Outcome {
    let board = frame.read_board()?;
    let who = frame.caller(&board)?;
    need_coordinator(&who, "purge")?;
    let wanted = criteria_of(&board, &who, frame, words)?;
    let plan = purge_plan(&board, &wanted.criteria)?;
    if total(&plan.counts) == 0.0 {
        frame.say(&["nothing matches: there is nothing to purge."]);
        return Ok(());
    }
    let rows: Vec<Vec<String>> = plan
        .counts
        .iter()
        .filter(|(_, n)| js::truthy(n))
        .map(|(key, n)| vec![key.clone(), display(n)])
        .collect();
    let shown: Vec<String> = table(&rows)
        .split('\n')
        .map(|line| format!("  {line}"))
        .collect();
    let args = &wanted.args;
    let dir = frame.home().join(frame.project).join("purged");
    frame.say(&[
        &format!("dry run, nothing written. coord purge {args} would remove:"),
        &shown.join("\n"),
        "",
        &format!(
            "A restorable backup is written to {} first. To do it, run:",
            dir.display()
        ),
        &format!("  coord purge {args} --confirm {}", frame.project),
    ]);
    Ok(())
}
