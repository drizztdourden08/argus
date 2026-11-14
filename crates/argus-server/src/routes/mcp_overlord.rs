//! `POST /mcp/overlord`, the private MCP endpoint of the hosted Overlord's child. It answers only the
//! bearer made for the running child (never the API key, never the dashboard token): any other call is
//! 401. Its one tool, `permission_prompt`, holds until the owner answers in the Overlord tab.
//!
//! The route takes no gate credential (`AuthMode::None`), because the gate would compare a bearer with
//! the dashboard token and the `/mcp` Origin rules do not fit a CLI child. The handler checks the bearer.

use std::sync::Arc;

use argus_core::{ApiError, ErrorCode};
use argus_overlord::approvals::{Parsed, parse, timeout_ms, tool_result};
use axum::body::Body;

use crate::adapters::overlord::approvals::NotifyToast;
use crate::adapters::overlord::gate::Ask;
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
        Parsed::Ask(prompt) if config::is_shadow(&ctx.svc) => {
            let reply = shadow::judge(&ctx.svc, &prompt);
            Ok(json_reply(200, &tool_result(&prompt.id, &reply)))
        }
        Parsed::Ask(prompt) => {
            let id = prompt.id.clone();
            let ask = Ask {
                prompt,
                wait_ms: timeout_ms(config::live(&ctx.svc).approval_timeout_min),
                link_base: format!("http://127.0.0.1:{}", ctx.svc.port()),
            };
            let toast = Arc::new(NotifyToast(ctx.svc.notify.clone()));
            let reply = gate.ask(&ask, toast).await;
            Ok(json_reply(200, &tool_result(&id, &reply)))
        }
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
