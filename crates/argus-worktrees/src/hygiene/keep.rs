//! `worktrees.keep`: path globs of worktrees the automatic hygiene never touches (frozen test builds). A matching
//! worktree is still listed; only the automatic steps skip it. A removal request for a kept worktree (a keep pattern
//! or an exception that still holds) is refused until the keep is cleared (decision D-117).

use crate::error::WtError;
use crate::fact::Fact;
use crate::paths::{clean_path, is_worktree_id, name_glob_matches, worktree_id};
use crate::remove::forbidden;
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

    /// The refusal of a removal request for a worktree on the keep list (an exception that still holds, or a keep
    /// pattern): clear the keep first. None when it is not kept.
    pub(crate) fn kept_refusal(&self, project: &str, target: &str) -> Option<WtError> {
        let target = target.trim();
        let id = &if is_worktree_id(target) {
            target.to_string()
        } else {
            worktree_id(&clean_path(target), self.ci())
        };
        let fact = self.facts_of(project).into_iter().find(|f| f.id == *id);
        let path = fact.as_ref().map_or("", |f| f.path.as_str());
        let now = self.now();
        let held = self.with_hygiene(project, |m| {
            m.exceptions
                .get(id)
                .filter(|e| e.holds(now))
                .map(|e| e.reason.clone())
        });
        let why = match (held, self.keep_pattern(path)) {
            (Some(reason), _) => reason,
            (None, Some(glob)) if !path.is_empty() => format!("keep pattern {glob}"),
            _ => return None,
        };
        let name = if path.is_empty() { id } else { path };
        Some(forbidden(format!(
            "{name} is on the keep list ({why}): clear the keep first with coord worktree keep {id} --clear"
        )))
    }
}
