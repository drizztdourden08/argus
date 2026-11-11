//! The doc collections of the page-facing mirror, as the Node `db.mjs` lists them.

use std::fmt;
use std::str::FromStr;

use serde::{Deserialize, Serialize};

/// One collection of the `docs` table.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Coll {
    /// Board threads.
    Threads,
    /// Kanban items.
    Items,
    /// Tests.
    Tests,
    /// Entries (questions, notices, owner posts).
    Entries,
    /// Resource locks.
    Locks,
    /// Owner actions.
    Actions,
    /// Sub-agent records.
    Agents,
    /// Work logs.
    Log,
    /// Board meta (`board`, plus foreign metas).
    Meta,
    /// Run docs.
    Runs,
    /// Run definitions as the page sees them.
    Launches,
    /// Portfolio cards.
    Cards,
    /// Bundles.
    Builds,
    /// Launchers.
    Launchers,
    /// Wake mirror.
    Wakes,
    /// Thread registry.
    Registry,
    /// Worktrees.
    Worktrees,
    /// Processes.
    Procs,
    /// Activity log.
    Activity,
    /// Owner notifications.
    Notifications,
    /// Local pull requests (`PR-n`), written by Brock's tooling; read only here.
    Prs,
    /// Cross-board dependencies (the portfolio ledger).
    Deps,
    /// Open stalls of the stall watch (`_portfolio` only), written by the wake engine.
    Stalls,
    /// Board items of the portfolio (`BD-n`), one per project board.
    Boards,
    /// Claude accounts of the owner (`_portfolio` only), one doc per account key.
    Accounts,
    /// The state docs of the accounts service: `drift`, `run`, `switch`, `cycle`.
    #[serde(rename = "accountsState")]
    AccountsState,
    /// The steps of the running or last account sync, by zero-padded sequence.
    #[serde(rename = "syncSteps")]
    SyncSteps,
    /// Handover records (`H-n`) of an account switch.
    Handovers,
}

impl Coll {
    /// Every collection, in the Node order.
    pub const ALL: [Coll; 28] = [
        Coll::Threads,
        Coll::Items,
        Coll::Tests,
        Coll::Entries,
        Coll::Locks,
        Coll::Actions,
        Coll::Agents,
        Coll::Log,
        Coll::Meta,
        Coll::Runs,
        Coll::Launches,
        Coll::Cards,
        Coll::Builds,
        Coll::Launchers,
        Coll::Wakes,
        Coll::Registry,
        Coll::Worktrees,
        Coll::Procs,
        Coll::Activity,
        Coll::Notifications,
        Coll::Prs,
        Coll::Deps,
        Coll::Stalls,
        Coll::Boards,
        Coll::Accounts,
        Coll::AccountsState,
        Coll::SyncSteps,
        Coll::Handovers,
    ];

    /// The collection's wire name.
    #[must_use]
    pub fn as_str(self) -> &'static str {
        match self {
            Coll::Threads => "threads",
            Coll::Items => "items",
            Coll::Tests => "tests",
            Coll::Entries => "entries",
            Coll::Locks => "locks",
            Coll::Actions => "actions",
            Coll::Agents => "agents",
            Coll::Log => "log",
            Coll::Meta => "meta",
            Coll::Runs => "runs",
            Coll::Launches => "launches",
            Coll::Cards => "cards",
            Coll::Builds => "builds",
            Coll::Launchers => "launchers",
            Coll::Wakes => "wakes",
            Coll::Registry => "registry",
            Coll::Worktrees => "worktrees",
            Coll::Procs => "procs",
            Coll::Activity => "activity",
            Coll::Notifications => "notifications",
            Coll::Prs => "prs",
            Coll::Deps => "deps",
            Coll::Stalls => "stalls",
            Coll::Boards => "boards",
            Coll::Accounts => "accounts",
            Coll::AccountsState => "accountsState",
            Coll::SyncSteps => "syncSteps",
            Coll::Handovers => "handovers",
        }
    }
}

impl fmt::Display for Coll {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The name is not one of the collections.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
#[error("unknown collection \"{0}\"")]
pub struct UnknownColl(pub String);

impl FromStr for Coll {
    type Err = UnknownColl;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Coll::ALL
            .into_iter()
            .find(|c| c.as_str() == s)
            .ok_or_else(|| UnknownColl(s.to_owned()))
    }
}

#[cfg(test)]
mod tests {
    use super::Coll;

    #[test]
    fn boards_parses_and_is_listed() {
        assert_eq!("boards".parse::<Coll>(), Ok(Coll::Boards));
        assert_eq!(Coll::Boards.as_str(), "boards");
        assert!(Coll::ALL.contains(&Coll::Boards));
    }
}
