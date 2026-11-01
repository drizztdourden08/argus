//! Speed: a worktree's `node_modules` (pnpm: tens of thousands of files and junctions) is what makes `git worktree
//! remove` slow. When it is a real folder, git ignores it and it holds no tracked file, it is regenerable: Argus deletes
//! it itself first, the junction-safe way (links as links), once every guard and the backup have passed. A delete that
//! fails midway is not a refusal: git gets what is left, as before.

use crate::fact::Fact;
use crate::paths::join_path;
use crate::service::Service;
use crate::tree::remove_entry;
use std::io;

/// What Argus did to the `node_modules` of a worktree before git ran.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum NodeModules {
    /// There was none, or it is not Argus's to delete: git gets it.
    Untouched,
    /// Gone.
    Cleared,
    /// The delete started and failed midway.
    Partial,
}

impl NodeModules {
    pub(crate) fn cleared(self) -> bool {
        self == Self::Cleared
    }

    pub(crate) fn begun(self) -> bool {
        self != Self::Untouched
    }
}

impl Service {
    /// A real folder (not a link), ignored by git (`git check-ignore -q node_modules` exits 0), with no tracked file.
    fn modules_are_ours(&self, path: &str, modules: &str) -> bool {
        if !self.deps.fs.lstat(modules).is_ok_and(|s| s.is_dir) {
            return false;
        }
        let git = &self.deps.git;
        if git
            .write(path, &["check-ignore", "-q", "node_modules"])
            .code
            != Some(0)
        {
            return false;
        }
        let tracked = git.write(path, &["ls-files", "--cached", "-z", "--", "node_modules"]);
        tracked.code == Some(0) && tracked.stdout.is_empty()
    }

    /// One entry through the junction-safe delete, counted on the removal's step.
    fn remove_counted(&self, id: &str, path: &str, done: &mut u64) -> io::Result<()> {
        remove_entry(self.deps.fs.as_ref(), path, self.opts.drop_links_first)?;
        *done += 1;
        self.step_done(id, *done);
        Ok(())
    }

    /// The folder entry by entry, and one level deeper in a real folder (pnpm's `.pnpm` holds nearly everything), so
    /// the step shows how far it got. Every entry goes through `remove_entry`: a link as a link, never followed.
    fn clear_counted(&self, id: &str, modules: &str) -> io::Result<()> {
        let fs = self.deps.fs.as_ref();
        let mut done = 0;
        for entry in fs.read_dir(modules)? {
            let path = join_path(modules, &entry.name);
            if fs.lstat(&path).is_ok_and(|s| s.is_dir && !s.is_link) {
                for inner in fs.read_dir(&path)? {
                    self.remove_counted(id, &join_path(&path, &inner.name), &mut done)?;
                }
            }
            self.remove_counted(id, &path, &mut done)?;
        }
        fs.remove_dir(modules)
    }

    pub(crate) fn clear_ignored_node_modules(&self, now: &Fact) -> NodeModules {
        let modules = join_path(&now.path, "node_modules");
        if !self.modules_are_ours(&now.path, &modules) {
            return NodeModules::Untouched;
        }
        let _ = self.clear_counted(&now.id, &modules);
        if self.deps.fs.exists(&modules) {
            NodeModules::Partial
        } else {
            NodeModules::Cleared
        }
    }
}
