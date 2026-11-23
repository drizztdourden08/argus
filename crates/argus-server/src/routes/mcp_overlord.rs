//! `POST /mcp/overlord`, the private MCP endpoint of the hosted Overlord's child. It answers only the
//! bearer made for the running child (never the API key, never the dashboard token): any other call is
//! 401. Its tool `permission_prompt` allows every ask at once: the owner wants no approvals. Its other tools
//! (`thread_message`, `thread_wake`, `sessions_list`, `session_message`) are run by the tools adapter.
//!
//! The route takes no gate credential (`AuthMode::None`), because the gate would compare a bearer with
//! the dashboard token and the `/mcp` Origin rules do not fit a CLI child. The handler checks the bearer.

use std::sync::Arc;

use argus_core::{ApiError, ErrorCode};
use argus_overlord::approvals::{Parsed, allow, parse, tool_result};
use argus_overlord::tools::text_result;
use axum::body::Body;

use crate::adapters::overlord::tools;
use crate::reply::{bytes_reply, json_reply};
use crate::router::{AuthMode, Reply};
use crate::services::overlord::{config, shadow};
use crate::{Ctx, RouteOpts, Router};

const MAX_BYTES: usize = 256 * 1024;

async fn post(ctx: Ctx) -> Reply {
    let gate = Arc::clone(&ctx.svc.overlord_approvals);
    if !gate.bearer_ok(ctx.header("authorization")) {
        let why = "the Overlord endpoint needs its own bearer";
        return Err(ApiError::new(ErrorCode::Unauthorized, why));
    }
    let body = ctx.raw(MAX_BYTES).await?;
    match parse(&body) {
        Parsed::Now(message) => Ok(json_reply(200, &message)),
        Parsed::Quiet => Ok(bytes_reply(202, Body::empty(), &[])),
        Parsed::Tool { id, call } => {
            let answer = tools::run(&ctx.svc, call).await;
            Ok(json_reply(
                200,
                &text_result(&id, &answer.text, answer.is_error),
            ))
        }
        Parsed::Ask(prompt) | Parsed::Allowed(prompt) if config::is_shadow(&ctx.svc) => {
            let reply = shadow::judge(&ctx.svc, &prompt);
            Ok(json_reply(200, &tool_result(&prompt.id, &reply)))
        }
        // The owner wants zero approvals (2026-11-22): every ask the child still makes is
        // allowed at once, and nothing waits on him.
        Parsed::Ask(prompt) | Parsed::Allowed(prompt) => Ok(json_reply(
            200,
            &tool_result(&prompt.id, &allow(&prompt.input)),
        )),
    }
}

/// Registers `POST /mcp/overlord`.
pub fn register(r: &mut Router) {
    let opts = RouteOpts {
        auth: AuthMode::None,
        max_bytes: MAX_BYTES,
        ..RouteOpts::default()
    };
    r.add("POST", "/mcp/overlord", opts, post);
}
