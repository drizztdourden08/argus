//! The usage of the hosted Overlord's session, for the owner's page only (I-200):
//! `GET /api/overlord/usage` is `{session, lastTurn, context: {usedTokens, windowTokens}, model,
//! month: {spentUsd, budgetUsd}, updatedAt}`. `session` and `lastTurn` are `{inputTokens,
//! outputTokens, cacheReadTokens, cacheWriteTokens, costUsd, turns, durationMs}`: the sum over the
//! session's results and the last result (null before the first). The supervisor keeps them in
//! memory and starts again at a new session id.

use argus_core::clock::now_iso;
use argus_overlord::session::usage::window_tokens;
use serde_json::json;

use super::overlord::web_only;
use super::overlord_spend::body as spend_body;
use crate::reply::json_reply;
use crate::router::Reply;
use crate::services::overlord::config::live;
use crate::{Ctx, Router};

async fn show(ctx: Ctx) -> Reply {
    web_only(&ctx, false)?;
    let spend = spend_body(&ctx).await?;
    let tally = ctx.svc.overlord.meter.snapshot();
    let model = tally.model.unwrap_or_else(|| live(&ctx.svc).model);
    let answer = json!({
        "session": tally.session,
        "lastTurn": tally.last_turn,
        "context": {
            "usedTokens": tally.context_tokens,
            "windowTokens": window_tokens(&model),
        },
        "model": model,
        "month": {
            "spentUsd": spend["spentUsd"],
            "budgetUsd": spend["budgetUsd"],
        },
        "updatedAt": now_iso(),
    });
    Ok(json_reply(200, &answer))
}

/// Registers the usage route.
pub fn register(r: &mut Router) {
    r.get("/api/overlord/usage", show);
}
