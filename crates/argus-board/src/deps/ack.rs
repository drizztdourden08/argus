//! `ack`: the assignee read its inbox while a request waits, so the request is acknowledged.
//! Argus writes `ackedAt` and `ackedBy` once; a reopen does not clear them.

use std::path::Path;

use argus_core::clock::now_iso;

use super::error::DepError;
use super::ledger::{COLL, dep_in, locked_ledger};
use super::model::DepState;
use crate::json::{doc_mut, set};
use crate::store::stamp;

/// Acknowledge dependency `id` for `project` (the assignee's project), on the read of `actor`.
/// Answers whether it was acknowledged now. Anything that does not qualify answers `false`: the
/// dependency is unknown, not `open`, asked of another project, or acknowledged already.
///
/// # Errors
/// A ledger that cannot be read or written.
pub fn ack(home: &Path, id: &str, project: &str, actor: &str) -> Result<bool, DepError> {
    locked_ledger(home, |p| {
        let due = dep_in(p, id).is_some_and(|d| {
            d.state == DepState::Open && d.to.project == project && d.acked_at.is_none()
        });
        if !due {
            return Ok(false);
        }
        if let Some(d) = doc_mut(p, COLL, id) {
            set(d, "ackedAt", now_iso());
            set(d, "ackedBy", actor);
        }
        stamp(p, COLL, id, actor)?;
        Ok(true)
    })
}
