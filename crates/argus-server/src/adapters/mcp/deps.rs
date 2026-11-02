//! The dependency tools over the dependency calls of the server. The owner cancels; the list
//! reads the `deps` docs of the portfolio scope in the tool itself.

use std::sync::Arc;

use argus_board::deps::{CancelRequest, cancel as cancel_dep};
use argus_mcp::ToolError;
use serde_json::{Value, json};

use super::errors::api;
use crate::Services;
use crate::services::deps::{self as service, act};

/// Whether dependencies are on.
pub fn enabled(svc: &Services) -> bool {
    service::enabled(svc)
}

/// Cancels a dependency as the owner: `{dep, text}`.
pub async fn cancel(
    svc: &Arc<Services>,
    id: &str,
    reason: Option<&str>,
) -> Result<Value, ToolError> {
    let id = id.to_owned();
    let request = CancelRequest {
        reason: reason.map(str::to_owned),
        actor: "owner".to_owned(),
    };
    let done = act(svc, move |home| cancel_dep(home, &id, &request))
        .await
        .map_err(api)?;
    Ok(json!({ "dep": done.dep.to_value(), "text": done.text }))
}
