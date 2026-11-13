//! The path guard of the scratch roots. It is its own guard and never relaxes the leftover guard's four-segment rule:
//! a scratch folder is only ever a direct child of a root the owner configured, matched by an include glob.

use super::ScratchRoot;
use super::walk::{Look, look};
use crate::paths::{base_name, clean_path, is_below, name_allowed, parent_of, same_path};
use crate::rule::Rule;
use crate::service::Service;

/// What the guard says about one candidate folder.
#[derive(Debug, PartialEq, Eq)]
pub(super) enum Verdict {
    /// Untouched for long enough, nothing in the way: it may go.
    Remove,
    /// Touched inside the age limit: kept, with nothing to say.
    Young,
    /// Refused, with the plain sentence.
    Refused(String),
}

/// What is wrong with the path itself, or none: the root, the direct child, the include globs and the folders of
/// repositories and boards (`protected`: every main checkout and rule folder the scans know).
pub(super) fn path_problem(
    entry: &ScratchRoot,
    path: &str,
    protected: &[String],
    ci: bool,
) -> Option<String> {
    let root = clean_path(&entry.root);
    if let Some(problem) = Rule::folder("scratch", &root, "").problem() {
        return Some(format!("scratch root {root} is invalid: {problem}"));
    }
    if same_path(&root, path, ci) {
        return Some(format!("{path} is the scratch root itself: never removed"));
    }
    if !same_path(&root, &parent_of(path), ci) {
        return Some(format!(
            "{path} is not a direct child of scratch root {root}"
        ));
    }
    if entry.include.is_empty() || !name_allowed(&entry.include, &[], &base_name(path), ci) {
        return Some(format!(
            "{} matches no include glob of scratch root {root}",
            base_name(path)
        ));
    }
    protected
        .iter()
        .find(|p| same_path(p, path, ci) || is_below(p, path, ci) || is_below(path, p, ci))
        .map(|p| format!("{path} is, holds or sits inside {p}, a main checkout or board folder"))
}

impl Service {
    /// Every main checkout and every rule folder of every board: scratch never touches one.
    pub(super) fn protected_paths(&self) -> Vec<String> {
        let mut paths = self.known_repos();
        for project in self.deps.store.projects() {
            if let Some(board) = self.deps.store.board(&project) {
                paths.extend(board.rules.iter().map(|r| clean_path(&r.path)));
            }
        }
        paths.sort();
        paths.dedup();
        paths
    }

    /// The guard, in order: the path, a `.git` anywhere inside or a recent touch (one bounded walk), a process that
    /// uses the folder.
    pub(super) fn scratch_verdict(
        &self,
        entry: &ScratchRoot,
        path: &str,
        protected: &[String],
        cutoff_ms: i64,
    ) -> Verdict {
        if let Some(problem) = path_problem(entry, path, protected, self.ci()) {
            return Verdict::Refused(problem);
        }
        match look(self.deps.fs.as_ref(), path, cutoff_ms) {
            Look::Young => return Verdict::Young,
            Look::Git(git) => {
                return Verdict::Refused(format!("{path} holds a {git}: not scratch"));
            }
            Look::TooBig => {
                return Verdict::Refused(format!(
                    "{path} is too big to check: nothing was removed"
                ));
            }
            Look::Unreadable(why) => {
                return Verdict::Refused(format!("could not read {why}: nothing was removed"));
            }
            Look::Old => {}
        }
        match self.busy_by(path, true, "removed") {
            Ok(None) => Verdict::Remove,
            Ok(Some(user)) => Verdict::Refused(Self::in_use(path, &user, "removed").message),
            Err(err) => Verdict::Refused(err.message),
        }
    }
}
