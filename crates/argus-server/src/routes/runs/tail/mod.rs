//! The live tail of a run (`GET /api/p/:key/runs/:rid/logs/stream`). The stream listens before
//! it replays, and one cursor under one lock orders the replay and the live lines, so no line
//! is missed or sent twice. It ends after a final `status` event. The tail core writes through
//! a sink, so another route can reuse it with its own framing.

use std::sync::Arc;

use super::logs::number;
use super::{ids, service, unknown_run};
use crate::router::Reply;
use crate::sse::open_stream;
use crate::{Ctx, Router};

pub(crate) mod core;
pub(crate) mod sink;

use self::core::{Tail, Watching, locked};
use self::sink::LegacySink;

fn start_cursor(ctx: &Ctx) -> u64 {
    let last = ctx
        .header("last-event-id")
        .and_then(|v| v.trim().parse::<i64>().ok());
    let seen = number(ctx, "after").or(last).unwrap_or(0);
    u64::try_from(seen).unwrap_or(0)
}

async fn stream(ctx: Ctx) -> Reply {
    let runs = service(&ctx)?;
    let (key, rid) = ids(&ctx)?;
    runs.get(&key, &rid)
        .ok_or_else(|| unknown_run(&key, &rid))?;
    let watching: Arc<Watching> = Arc::default();
    let held = Arc::clone(&watching);
    let on_drop = Box::new(move || locked(&held).clear());
    let (sender, res) = open_stream(ctx.svc.heartbeat, &ctx.svc.streams, Some(on_drop));
    let sink = Box::new(LegacySink::new(sender));
    let tail = Tail::new(runs, (key, rid), start_cursor(&ctx), sink, watching);
    tail.start(&ctx.svc);
    Ok(res)
}

pub(super) fn register(r: &mut Router) {
    r.get("/api/p/:key/runs/:rid/logs/stream", stream);
}
