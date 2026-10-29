//! The build cache of one root: a report of it, and a prune.

use crate::entry::Entry;
use crate::outcome::Outcome;
use crate::places::{Place, Usage, read_usage};
use crate::plan::plan;
use crate::prune;
use crate::report::{Report, Survey, rows_and_totals};
use crate::scan::entries;
use crate::tmp::{Tmp, stale};
use argus_worktrees::{Fs, RealFs, clean_path};
use std::collections::BTreeSet;
use std::path::Path;

const DAY_MS: i64 = 86_400_000;

/// Why a prune did not run.
#[derive(Debug, thiserror::Error)]
pub enum PruneError {
    /// A lockfile of a known place could not be read, so no entry can be called unused.
    #[error("nothing deleted, a lockfile could not be read: {0}")]
    Unreadable(String),
}

/// What one look at the cache and its users found.
struct Look {
    entries: Vec<Entry>,
    usage: Usage,
    stale_tmp: Vec<Tmp>,
}

/// The cache folder of a user.
pub struct BuildCache {
    root: String,
    fs: Box<dyn Fs>,
}

impl BuildCache {
    /// The cache at `root` on the real file system.
    pub fn new(root: &Path) -> Self {
        BuildCache {
            root: clean_path(&root.to_string_lossy()),
            fs: Box::new(RealFs),
        }
    }

    /// The root, forward slashes.
    pub fn root(&self) -> &str {
        &self.root
    }

    fn look(&self, places: &[Place], now_ms: i64) -> Look {
        let entries = entries(self.fs.as_ref(), &self.root, now_ms);
        let names: BTreeSet<String> = entries.iter().map(|e| e.name.clone()).collect();
        Look {
            usage: read_usage(self.fs.as_ref(), places, &names),
            stale_tmp: stale(self.fs.as_ref(), &self.root, now_ms),
            entries,
        }
    }

    /// The rows, the totals and the problems, judged now.
    pub fn report(&self, places: &[Place], grace_days: u32, now_ms: i64) -> Report {
        let look = self.look(places, now_ms);
        let survey = Survey {
            entries: &look.entries,
            usage: &look.usage,
            stale_tmp: &look.stale_tmp,
            now_ms,
            grace_ms: i64::from(grace_days) * DAY_MS,
        };
        let (rows, totals) = rows_and_totals(&survey);
        Report {
            root: self.root.clone(),
            exists: self.fs.is_dir(&self.root),
            grace_days,
            rows,
            totals,
            problems: look.usage.problems,
        }
    }

    /// Deletes the unused entries past the grace period and the old temp folders.
    pub fn prune(
        &self,
        places: &[Place],
        grace_days: u32,
        now_ms: i64,
    ) -> Result<Outcome, PruneError> {
        let look = self.look(places, now_ms);
        if !look.usage.problems.is_empty() {
            return Err(PruneError::Unreadable(look.usage.problems.join("; ")));
        }
        let grace_ms = i64::from(grace_days) * DAY_MS;
        let todo = plan(&look.entries, &look.usage, look.stale_tmp, now_ms, grace_ms);
        Ok(prune::run(self.fs.as_ref(), &self.root, &todo, now_ms))
    }
}
