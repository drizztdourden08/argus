//! The connection of a verb to the dashboard server, and the shape of the answers.

use argus_core::Paths;
use serde_json::Value;

use crate::cli::route::Call;
use crate::client::{Client, ClientError, dashboard_client};

/// The data folders of this call.
#[must_use]
pub fn paths_of(call: &Call<'_>) -> Paths {
    call.inv.env.paths(&call.inv.cwd)
}

/// A client of the running dashboard; with `autostart` a stopped one is started and waited for.
///
/// # Errors
/// The dashboard is not running and may not, or could not, be started.
pub fn connect(call: &Call<'_>, autostart: bool) -> Result<Client, ClientError> {
    dashboard_client(&call.inv.env, &paths_of(call), autostart)
}

/// The `project` of a call that has one.
#[must_use]
pub fn project_of<'a>(call: &'a Call<'_>) -> &'a str {
    call.project.as_deref().unwrap_or_default()
}

/// The `items` list of an answer, empty when it holds none.
#[must_use]
pub fn list_of<'a>(answer: &'a Value, field: &str) -> &'a [Value] {
    answer[field].as_array().map_or(&[], Vec::as_slice)
}
