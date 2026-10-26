//! Routes of services a later slice brings (MCP): until then they answer 503 `unavailable`,
//! as the Node server does when a module is missing.

use argus_core::ApiError;

use crate::router::{AuthMode, Reply};
use crate::{Ctx, RouteOpts, Router};

async fn mcp(_ctx: Ctx) -> Reply {
    Err(ApiError::unavailable(
        "mcp/http is not available in this build",
    ))
}

/// Registers the 503 routes.
pub fn register(r: &mut Router) {
    let bearer = RouteOpts {
        auth: AuthMode::Bearer,
        ..RouteOpts::default()
    };
    r.add("POST", "/mcp", bearer, mcp);
    r.add("GET", "/mcp", bearer, mcp);
}
