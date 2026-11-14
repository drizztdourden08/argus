//! The approvals of the hosted Overlord, for the owner's page only:
//! `GET /api/overlord/approvals` is `{approvals: [...]}`, pending and recent, newest first, and
//! `POST /api/overlord/approvals/<id>` takes `{decision: "allow" | "deny", message?}` and answers the
//! settled approval. A second answer to the same approval is 409.

use argus_core::{ApiError, ErrorCode};
use argus_overlord::approvals::{DecideFault, Decision};
use serde_json::json;

use super::overlord::web_only;
use crate::reply::json_reply;
use crate::router::Reply;
use crate::{Ctx, RouteOpts, Router};

const LIST: usize = 50;
const SMALL: usize = 8 * 1024;
const MESSAGE_MAX: usize = 500;

async fn list(ctx: Ctx) -> Reply {
    web_only(&ctx, false)?;
    let approvals = ctx.svc.overlord_approvals.recent(LIST);
    Ok(json_reply(200, &json!({ "approvals": approvals })))
}

fn refusal(fault: DecideFault) -> ApiError {
    match fault {
        DecideFault::Unknown => ApiError::new(ErrorCode::NotFound, "no such approval"),
        DecideFault::Settled => ApiError::new(ErrorCode::Conflict, "it was answered already"),
    }
}

async fn decide(ctx: Ctx) -> Reply {
    web_only(&ctx, true)?;
    let body = ctx.json().await?;
    let decision = body["decision"]
        .as_str()
        .and_then(Decision::parse)
        .ok_or_else(|| ApiError::bad_request("decision is allow or deny"))?;
    let message: Option<String> = body["message"]
        .as_str()
        .map(|m| m.chars().take(MESSAGE_MAX).collect())
        .filter(|m: &String| !m.trim().is_empty());
    let done = ctx
        .svc
        .overlord_approvals
        .decide(ctx.param("id"), decision, message.as_deref())
        .map_err(refusal)?;
    Ok(json_reply(200, &json!(done)))
}

/// Registers the approval routes.
pub fn register(r: &mut Router) {
    let small = RouteOpts {
        max_bytes: SMALL,
        ..RouteOpts::default()
    };
    r.get("/api/overlord/approvals", list);
    r.add("POST", "/api/overlord/approvals/:id", small, decide);
}
