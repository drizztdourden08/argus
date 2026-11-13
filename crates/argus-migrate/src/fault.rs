//! The fault-injection hook: a run given a `Fault` stops (or damages its backup) at that point.

use crate::error::{Error, Result};
use crate::options::{Fault, Options};

/// Fails the run when `point` is the injected fault.
pub fn at(opts: &Options, point: Fault) -> Result<()> {
    if armed(opts, point) {
        return Err(Error::Fault(format!("{point:?}")));
    }
    Ok(())
}

/// Whether `point` is the injected fault.
pub fn armed(opts: &Options, point: Fault) -> bool {
    opts.fault == Some(point)
}
