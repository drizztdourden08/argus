//! Where the dashboard serves a page, as the verbs print it.

use super::link::paths_of;
use crate::cli::route::Call;
use crate::client::board_port;

/// `http://127.0.0.1:<port><path>`: the port comes from the dashboard's `config.json` once it ran.
#[must_use]
pub fn dashboard_url(call: &Call<'_>, path: &str) -> String {
    let port = board_port(&call.inv.env, &paths_of(call));
    format!("http://127.0.0.1:{port}{path}")
}
