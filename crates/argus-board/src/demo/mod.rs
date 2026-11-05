//! The demo board (`coord demo seed`): the steps, the cross-board requests and the release
//! records that fill a preview board with at least one record of every kind the dashboard shows.
//! This is data only. The verb in `argus-coord` runs the steps through the real `coord` verbs
//! and the board ops, so every record passes the same checks as a hand-made one.

mod agents;
mod board;
mod changes;
mod entries;
mod links;
mod plans;
mod releases;
mod runs;
mod tasks;
mod testing;
mod work;

pub use agents::FOLDERS;
pub use changes::{MERGE_COMMIT, MERGED_PR};
pub use links::{DepSpec, INCOMING, OUTGOING, TARGETS};
pub use releases::write as write_releases;

/// The key of the demo board when `--project` is not given.
pub const DEFAULT_KEY: &str = "demo";

/// The lead thread of every demo board.
pub const LEAD: &str = "lead";

/// One call of `coord`: which board, who makes it, the words, and the name its first id is kept
/// under (a `{name}` in a later step is replaced by it).
#[derive(Debug, Clone, Copy)]
pub struct Step {
    /// The project of the call; empty is the demo board.
    pub project: &'static str,
    /// The thread acted as (`--as`); empty is none.
    pub who: &'static str,
    /// The name the first id of the output is bound to; empty binds nothing.
    pub bind: &'static str,
    /// The words after `coord`, with `{name}` placeholders.
    pub words: &'static [&'static str],
}

/// A step on the demo board.
#[must_use]
pub const fn step(who: &'static str, bind: &'static str, words: &'static [&'static str]) -> Step {
    on("", who, bind, words)
}

/// A step on another board.
#[must_use]
pub const fn on(
    project: &'static str,
    who: &'static str,
    bind: &'static str,
    words: &'static [&'static str],
) -> Step {
    Step {
        project,
        who,
        bind,
        words,
    }
}

/// The steps that fill the demo board, in order: threads, work, tests, runs, agents, entries,
/// pull requests, and the item the incoming request comes from.
#[must_use]
pub fn steps() -> Vec<Step> {
    let mut parts: Vec<&[Step]> = vec![board::STEPS];
    parts.extend([
        work::EPICS,
        work::STORIES,
        tasks::TASKS,
        tasks::OTHERS,
        testing::STEPS,
        plans::STEPS,
        runs::STEPS,
        agents::STEPS,
        entries::STEPS,
        changes::STEPS,
        links::INCOMING_ITEM,
    ]);
    parts.concat()
}
