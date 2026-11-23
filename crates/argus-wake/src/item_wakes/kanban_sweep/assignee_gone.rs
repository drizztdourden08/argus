//! `assignee-gone`: the assignee is an agent that no longer runs, or a thread that is gone or ended.

use super::{Ctx, Hit, OWNER};
use crate::board::{ItemView, LIVE_AGENT};

fn agent_gone(ctx: &Ctx<'_>, id: &str) -> bool {
    !ctx.board
        .agents
        .get(id)
        .is_some_and(|a| a.rec.is_live() && a.status.is_any(&LIVE_AGENT))
}

fn thread_gone(ctx: &Ctx<'_>, id: &str) -> bool {
    ctx.board.threads.get(id).is_none_or(|t| t.is_ended())
}

fn gone(ctx: &Ctx<'_>, assignee: &str) -> bool {
    match assignee.strip_prefix("agent:") {
        Some(agent) => agent_gone(ctx, agent),
        None => assignee != OWNER && thread_gone(ctx, assignee),
    }
}

pub(super) fn check(ctx: &Ctx<'_>, item: &ItemView) -> Option<Hit> {
    let assignee = item.assignee.some()?;
    gone(ctx, assignee).then(|| Hit {
        age: 0,
        note: format!("{}, assignee {assignee} is gone", item.status.as_str()),
    })
}
