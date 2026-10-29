//! Speed: a worktree's `node_modules` (pnpm: tens of thousands of files and junctions) is what makes `git worktree
//! remove` slow. When it is a real folder, git ignores it and it holds no tracked file, it is regenerable: Argus deletes
//! it itself first, the junction-safe way (links as links), once every guard and the backup have passed. A delete that
//! fails midway is not a refusal: git gets what is left, as before.

use crate::fact::Fact;
use crate::paths::join_path;
use crate::service::Service;
use crate::tree::remove_entry;

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

    pub(crate) fn clear_ignored_node_modules(&self, now: &Fact) -> NodeModules {
        let modules = join_path(&now.path, "node_modules");
        if !self.modules_are_ours(&now.path, &modules) {
            return NodeModules::Untouched;
        }
        let _ = remove_entry(self.deps.fs.as_ref(), &modules, self.opts.drop_links_first);
        if self.deps.fs.exists(&modules) {
            NodeModules::Partial
        } else {
            NodeModules::Cleared
        }
    }
}
