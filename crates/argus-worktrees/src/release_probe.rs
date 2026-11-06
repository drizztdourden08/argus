//! The release probe: before git touches the record of a worktree, prove that the folder can be let go.
//!
//! `git worktree remove` deletes the record and the folder's `.git` file first and the files after. When something holds
//! the folder open, git fails half way and leaves a folder git no longer knows. The same rename the finish fallback uses
//! (to a sibling name and straight back) fails the same way a held folder fails, and it changes nothing when it works.

use crate::error::{ErrorCode, WtError, io_code};
use crate::service::Service;

/// The mark of the sibling name the probe renames a folder to.
pub(crate) const PROBE_MARK: &str = ".argus-probe-";

fn probe_path(path: &str, stamp: i64) -> String {
    format!("{path}{PROBE_MARK}{stamp}")
}

impl Service {
    /// Rename the folder to a sibling name and back. A failure refuses the removal with nothing changed.
    pub(crate) fn probe_release(&self, path: &str) -> Result<(), WtError> {
        let aside = probe_path(path, self.now());
        let fs = self.deps.fs.as_ref();
        if let Err(err) = fs.rename(path, &aside) {
            return Err(WtError::new(
                ErrorCode::InUse,
                format!(
                    "{path} is in use (open in a process or locked, {}): nothing was removed. Close what holds it and request the removal again.",
                    io_code(&err)
                ),
            ));
        }
        fs.rename(&aside, path).map_err(|err| {
            WtError::new(
                ErrorCode::InUse,
                format!(
                    "{path} was renamed to {aside} to test it and could not be renamed back ({}): git was not run. Rename the folder back by hand and request the removal again.",
                    io_code(&err)
                ),
            )
        })
    }
}
