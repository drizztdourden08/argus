//! The stall rules that read the boards alone (rules 3, 4 and 5 of the stall watch). Pure: `find` looks at the
//! boards at one moment and returns the stalls that hold; nothing is written and no wake is sent.
//!
//! | rule | stall | holds when |
//! |---|---|---|
//! | 3 | `unstarted` | a dependency is `open` for the limit and its item is still `todo` |
//! | 4 | `idle` | the lead has no turn, agent or process for the limit and unblocked work is open |
//! | 5 | `silent` | an item is `doing` for the limit with no board write, no agent and no blocker |

mod found;
mod idle;
mod idle_lead;
mod lead;
mod silent;
mod text;
mod unstarted;

#[cfg(test)]
mod fixtures;

use std::sync::Arc;

pub use found::{Found, Kind};
pub use idle::IdleWatch;

use crate::board::BoardView;
use crate::reasons::PORTFOLIO;
use crate::source::BoardSource;

/// How long each condition must hold before it is a stall (ms).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Limits {
    /// Rule 3: a dependency open with its item still `todo`.
    pub unstarted_ms: i64,
    /// Rule 4: a lead idle with unblocked work.
    pub idle_ms: i64,
    /// Rule 5: an item `doing` with nothing happening.
    pub silent_ms: i64,
}

impl Default for Limits {
    fn default() -> Self {
        Limits {
            unstarted_ms: 30 * 60_000,
            idle_ms: 10 * 60_000,
            silent_ms: 60 * 60_000,
        }
    }
}

/// What the rules read.
pub struct RuleInput<'a> {
    /// The boards of every project and the portfolio board with the ledger.
    pub boards: &'a dyn BoardSource,
    /// Does the project have a running process or run that counts as work?
    pub busy: &'a dyn Fn(&str) -> bool,
    /// Since when each lead has been idle.
    pub idle: &'a IdleWatch,
    /// The thresholds.
    pub limits: Limits,
}

/// Every stall of rules 3, 4 and 5 that holds at `now` (epoch ms).
#[must_use]
pub fn find(now: i64, input: &RuleInput<'_>) -> Vec<Found> {
    let mut found = unstarted::find(now, input);
    found.extend(idle_lead::find(now, input));
    found.extend(silent::find(now, input));
    found
}

/// The project boards the rules of one project watch: not the main scope and not the ledger.
fn projects(input: &RuleInput<'_>) -> Vec<(String, Arc<BoardView>)> {
    let overlord = input.boards.overlord();
    input
        .boards
        .scopes()
        .into_iter()
        .filter(|scope| *scope != overlord && scope != PORTFOLIO)
        .filter_map(|scope| {
            let board = input.boards.board(&scope)?;
            Some((scope, board))
        })
        .collect()
}
