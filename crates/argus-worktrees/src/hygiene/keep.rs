//! `worktrees.keep`: path globs of worktrees the automatic hygiene never touches (frozen test builds). A matching
//! worktree is still listed; only the automatic steps skip it. A thread's own request, an approval or the owner's
//! delete still remove it.

use crate::fact::Fact;
use crate::paths::{clean_path, name_glob_matches};
use crate::service::Service;

impl Service {
    /// The keep pattern the worktree's full path matches, if any.
    pub(crate) fn keep_pattern(&self, path: &str) -> Option<&str> {
        let ci = self.ci();
        let path = clean_path(path);
        self.opts
            .keep
            .iter()
            .find(|g| name_glob_matches(&clean_path(g), &path, ci))
            .map(String::as_str)
    }

    /// Does a keep pattern hold this worktree back from the automatic steps?
    pub(crate) fn is_kept(&self, fact: &Fact) -> bool {
        self.keep_pattern(&fact.path).is_some()
    }
}
