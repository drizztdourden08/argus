//! `/mcp`, the MCP streamable HTTP endpoint (DESIGN 7.1), stateless: `POST` takes one JSON-RPC
//! message and answers JSON, `GET` and `DELETE` are 405. A loopback caller needs no credential;
//! a bearer is required when the config says `requireToken`, and a wrong one is 401. The
//! protocol and the tools are `argus-mcp`; this file maps the request in and the reply out.

use std::sync::Arc;

use argus_mcp::{MAX_BYTES, McpReply, McpRequest, ToolHost, handle_other, handle_post};
use axum::body::Body;
use axum::http::{HeaderValue, header};
use axum::response::Response;

use crate::adapters::mcp::ServerBackend;
use crate::reply::{bytes_reply, json_reply};
use crate::router::{AuthMode, Reply};
use crate::{Ctx, RouteOpts, Router};

fn respond(reply: McpReply) -> Response {
    match reply {
        McpReply::Accepted => bytes_reply(202, Body::empty(), &[]),
        McpReply::Json { status, body } => json_reply(status, &body),
        McpReply::Refused {
            status,
            error,
            allow,
        } => {
            let mut res = json_reply(status, &error.result());
            if let Some(allow) = allow {
                res.headers_mut()
                    .insert(header::ALLOW, HeaderValue::from_static(allow));
            }
            res
        }
    }
}

async fn post(ctx: Ctx) -> Reply {
    let body = ctx.raw(MAX_BYTES).await?;
    let request = McpRequest {
        protocol_version: ctx.header("mcp-protocol-version"),
        default_project: ctx.header("x-coord-default-project"),
        client: ctx.header("x-coord-client"),
        user_agent: ctx.header("user-agent"),
        host: ctx.header("host"),
        body: &body,
    };
    let backend = Arc::new(ServerBackend::new(Arc::clone(&ctx.svc)));
    let host = ToolHost::new(backend);
    Ok(respond(handle_post(&host, &request).await))
}

async fn other(_ctx: Ctx) -> Reply {
    Ok(respond(handle_other()))
}

/// Registers `POST`, `GET` and `DELETE /mcp`.
pub fn register(r: &mut Router) {
    let opts = RouteOpts {
        auth: AuthMode::Bearer,
        max_bytes: MAX_BYTES,
        ..RouteOpts::default()
    };
    r.add("POST", "/mcp", opts, post);
    r.add("GET", "/mcp", opts, other);
    r.add("DELETE", "/mcp", opts, other);
}
