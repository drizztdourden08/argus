//! The shadow's "would do" log, for the owner's page only: `GET /api/overlord/shadow` is
//! `{entries}`, the last entries oldest first. An entry is `{kind: "would-do", at, wakes, said}`
//! for a turn of the shadow, or `{kind: "denied", at, tool, summary}` for a call it was refused.

use serde_json::json;

use super::blocking::blocking;
use super::overlord::web_only;
use crate::reply::json_reply;
use crate::router::Reply;
use crate::services::overlord::shadow::{RECENT, ShadowLog};
use crate::{Ctx, Router};

async fn show(ctx: Ctx) -> Reply {
    web_only(&ctx, false)?;
    let log = ShadowLog::new(&ctx.svc.paths.home);
    let entries = blocking(move || log.recent(RECENT)).await?;
    Ok(json_reply(200, &json!({ "entries": entries })))
}

/// Registers `GET /api/overlord/shadow`.
pub fn register(r: &mut Router) {
    r.get("/api/overlord/shadow", show);
}
