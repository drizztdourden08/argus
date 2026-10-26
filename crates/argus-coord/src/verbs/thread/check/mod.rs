//! `coord check [--stale <minutes>] [--json]` (`verbs/check.mjs`): the checker's report. Read
//! only; printed as text for a human, `--json` for the checker agent.

mod builds;
mod report;
mod text;

use argus_board::json::to_number;

use super::frame::{Frame, with_frame};
use crate::cli::exit::Outcome;
use crate::cli::route::Call;

const STALE_DEFAULT: f64 = 30.0;

fn run_frame(frame: &mut Frame<'_>) -> Outcome {
    let board = frame.read_board()?;
    let stale_minutes = frame
        .options
        .json("stale")
        .map_or(STALE_DEFAULT, |v| to_number(Some(&v)));
    let clock = report::Clock {
        now_ms: frame.now_ms,
        stale_minutes,
    };
    let r = report::report(&board, clock);
    if frame.options.flag("json") {
        frame.json(&r);
    } else {
        text::print(frame, &r, stale_minutes);
    }
    Ok(())
}

/// The verb `check`.
///
/// # Errors
/// Refused when there is no board.
pub fn run(call: &mut Call<'_>) -> Outcome {
    with_frame(call, run_frame)
}
