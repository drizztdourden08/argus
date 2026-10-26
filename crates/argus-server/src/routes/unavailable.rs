//! Routes of services a later slice brings (runs, MCP): until then they answer 503
//! `unavailable`, as the Node server does when a module is missing.

use argus_core::ApiError;

use crate::router::{AuthMode, Reply};
use crate::{Ctx, RouteOpts, Router};

const RUNS: [(&str, &str); 9] = [
    ("GET", "/api/p/:key/launches"),
    ("GET", "/api/p/:key/runs"),
    ("POST", "/api/p/:key/runs"),
    ("GET", "/api/runs"),
    ("GET", "/api/p/:key/runs/:rid"),
    ("POST", "/api/p/:key/runs/:rid/stop"),
    ("GET", "/api/p/:key/runs/:rid/logs"),
    ("GET", "/api/p/:key/runs/:rid/logs/stream"),
    ("POST", "/api/p/:key/runs/:rid/attachments"),
];

async fn runs(_ctx: Ctx) -> Reply {
    Err(ApiError::unavailable(
        "http/routes-runs is not available in this build",
    ))
}

async fn mcp(_ctx: Ctx) -> Reply {
    Err(ApiError::unavailable(
        "mcp/http is not available in this build",
    ))
}

/// Registers the 503 routes.
pub fn register(r: &mut Router) {
    for (method, path) in RUNS {
        r.add(method, path, RouteOpts::default(), runs);
    }
    let bearer = RouteOpts {
        auth: AuthMode::Bearer,
        ..RouteOpts::default()
    };
    r.add("POST", "/mcp", bearer, mcp);
    r.add("GET", "/mcp", bearer, mcp);
}
