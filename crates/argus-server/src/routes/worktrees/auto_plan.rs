//! `GET /api/worktrees/auto-plan`: what the hygiene would do with every worktree under an automatic repo rule, read
//! only: `{mode, plan: [{board, path, class, wouldRemove, reason}]}`.

use serde_json::json;

use super::service;
use crate::reply::json_reply;
use crate::router::Reply;
use crate::routes::blocking::blocking;
use crate::{Ctx, RouteOpts, Router};

async fn plan(ctx: Ctx) -> Reply {
    let service = service(&ctx)?;
    let mode = service.auto_mode().as_str();
    let rows = blocking(move || service.auto_plan()).await?;
    Ok(json_reply(200, &json!({ "mode": mode, "plan": rows })))
}

pub(super) fn register(r: &mut Router) {
    r.add(
        "GET",
        "/api/worktrees/auto-plan",
        RouteOpts::default(),
        plan,
    );
}
