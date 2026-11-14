//! The owner's chat line, for the owner's page only: `POST /api/overlord/message` with `{text}`
//! writes the text to the hosted Overlord as a user message marked as the owner's. No MCP tool or
//! coord verb reaches it. Without a running child the answer is 409.

use argus_core::{ApiError, ErrorCode};
use argus_overlord::session::owner::{Refusal, deliver};
use serde_json::json;

use super::overlord::web_only;
use crate::reply::json_reply;
use crate::router::Reply;
use crate::{Ctx, RouteOpts, Router};

/// 20,000 characters at four bytes each, and the JSON around them.
const BODY_MAX: usize = 96 * 1024;

fn refusal(fault: &Refusal) -> ApiError {
    let code = match fault {
        Refusal::NotRunning => ErrorCode::Conflict,
        Refusal::Empty | Refusal::TooLong => ErrorCode::BadRequest,
    };
    ApiError::new(code, fault.sentence())
}

async fn post(ctx: Ctx) -> Reply {
    web_only(&ctx, true)?;
    let body = ctx.json().await?;
    let text = body["text"]
        .as_str()
        .ok_or_else(|| ApiError::bad_request("send the message as text"))?;
    deliver(ctx.svc.overlord.link().as_ref(), text).map_err(|f| refusal(&f))?;
    Ok(json_reply(200, &json!({ "sent": true })))
}

/// Registers the owner chat route.
pub fn register(r: &mut Router) {
    let opts = RouteOpts {
        max_bytes: BODY_MAX,
        ..RouteOpts::default()
    };
    r.add("POST", "/api/overlord/message", opts, post);
}
