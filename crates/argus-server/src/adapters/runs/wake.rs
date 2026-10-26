//! A run as the wake service reads it when it ended.

use argus_launchers::RunRow;
use argus_wake::policy::RunView;

/// The part of a finished run the wake reads.
#[must_use]
pub fn view_of(run: &RunRow) -> RunView {
    RunView {
        id: run.id.clone(),
        scope: run.scope.clone(),
        name: run.name.clone(),
        status: run.status.as_str().to_owned(),
        exit_code: run.exit_code.map(i64::from),
        error: run.error.clone(),
        last_line: run.last_line.clone(),
        orphan: run.orphan,
    }
}
