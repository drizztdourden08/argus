//! What waits for the hosted Overlord, for the owner's page only: `GET /api/overlord/queue` is
//! `{wakes, held, approvals, alerts}`, each an array (empty when nothing waits or no child runs).

use std::sync::Arc;

use super::blocking::blocking;
use super::overlord::web_only;
use crate::adapters::overlord::queue::body;
use crate::reply::json_reply;
use crate::router::Reply;
use crate::{Ctx, Router};

async fn show(ctx: Ctx) -> Reply {
    web_only(&ctx, false)?;
    let svc = Arc::clone(&ctx.svc);
    let queue = blocking(move || body(&svc)).await?;
    Ok(json_reply(200, &queue))
}

/// Registers the queue route.
pub fn register(r: &mut Router) {
    r.get("/api/overlord/queue", show);
}
