//! A worktree whose files just changed is busy: something works in it that no process list shows (a command that
//! creates a worktree, installs and upgrades in it takes over a minute, and a child process may hide its folder).

use crate::fact::Fact;
use crate::paths::join_path;
use crate::service::Service;

impl Service {
    /// The newest change time of the folder, its `.git` file and the entries directly under it.
    fn newest_change_ms(&self, path: &str) -> Option<i64> {
        let fs = self.deps.fs.as_ref();
        let own = [path.to_string(), join_path(path, ".git")];
        let below: Vec<String> = fs
            .read_dir(path)
            .unwrap_or_default()
            .iter()
            .map(|entry| join_path(path, &entry.name))
            .collect();
        own.iter()
            .chain(below.iter())
            .filter_map(|p| fs.lstat(p).ok().and_then(|stat| stat.mtime_ms))
            .max()
    }

    /// Why the worktree counts as busy: its folder, `.git` file or top-level files changed within the grace.
    pub(super) fn busy_why(&self, fact: &Fact) -> Option<String> {
        let grace = self.opts.busy_grace_ms;
        if grace <= 0 {
            return None;
        }
        let age = self.now() - self.newest_change_ms(&fact.path)?;
        (age < grace).then(|| {
            format!(
                "its files changed {} s ago: it counts as busy for {} minutes",
                age.max(0) / 1000,
                grace / 60_000
            )
        })
    }
}
