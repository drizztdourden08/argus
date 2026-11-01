//! The event-driven scan: a cheap look at the disk on the short tick, and a scan only when it saw a change.
//!
//! The look reads file times and sizes (`probe`), never git, and never takes the turn of scans and removals. A scan
//! that follows it reads with git only the worktrees whose fingerprint changed or that are new; every other worktree
//! keeps the facts of the last scan. Every `Options::scan_ms` (30 minutes) the scan reads everything, which also
//! catches what no fingerprint shows: an edited tracked file (dirty), a fetch, a push.

use crate::doc::Derived;
use crate::probe::{Reuse, fingerprint, shape_of};
use crate::service::Service;

impl Service {
    /// The facts of the last scan that may be kept: none once the safety period passed since the last full read.
    pub(crate) fn kept_of(&self, project: &str) -> Option<Reuse> {
        let state = self.st();
        let scan = state.scans.get(project)?;
        (self.now() - scan.full_ms < self.opts.scan_ms)
            .then(|| Reuse::new(scan.full_ms, scan.prints.clone(), &scan.facts))
    }

    /// Did anything change that a scan would see? True for a project not scanned yet, when the safety period passed,
    /// when a folder under a rule came or went, when a worktree's fingerprint changed (or it is gone), or when a folder
    /// the last scan found missing is back. File times only: no git, no turn.
    pub fn changed(&self, project: &str) -> bool {
        let Some(board) = self.deps.store.board(project) else {
            return false;
        };
        let known = {
            let state = self.st();
            state.scans.get(project).map(|scan| {
                let missing: Vec<String> = scan
                    .facts
                    .iter()
                    .filter(|f| !f.exists)
                    .map(|f| f.path.clone())
                    .collect();
                (
                    scan.full_ms,
                    scan.shape.clone(),
                    scan.prints.clone(),
                    missing,
                )
            })
        };
        let Some((full_ms, shape, prints, missing)) = known else {
            return true;
        };
        let fs = self.deps.fs.as_ref();
        self.now() - full_ms >= self.opts.scan_ms
            || shape_of(&board.rules, fs, self.ci()) != shape
            || missing.iter().any(|path| fs.is_dir(path))
            || prints
                .iter()
                .any(|(path, print)| fingerprint(fs, path) != *print)
    }

    /// The timer's scan: the look first, a scan that keeps unchanged worktrees only when it saw a change. None when
    /// nothing changed (the docs stay as they are).
    pub fn scan_if_changed(&self, project: &str) -> Option<Derived> {
        if self.changed(project) {
            self.scan_with(project, true)
        } else {
            None
        }
    }
}
