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
}

impl Coll {
    /// Every collection, in the Node order.
    pub const ALL: [Coll; 23] = [
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
