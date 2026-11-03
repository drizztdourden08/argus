//! The rows of a project, its open streams, and a wake from outside the policy.

use argus_board::Board;
use argus_board::store::read_board;
use argus_core::ApiError;
use argus_core::clock::{now_ms, parse_ms};
use argus_wake::external::external_wake;
use argus_wake::service::driver::lock;
use serde_json::{json, to_value};

use super::{refusal, service, thread};
use crate::Ctx;
use crate::reply::json_reply;
use crate::router::Reply;
use crate::routes::blocking::blocking;
use crate::routes::params::key;

const DEFAULT_LIMIT: usize = 50;
/// The notices route looks back at most this far.
const NOTICE_WINDOW_MS: i64 = 24 * 3_600_000;

/// The `limit` query as `Number()` and `Number.isInteger` read it: anything that is not a whole
/// number is the default, and the rest is held between 1 and 200.
fn limit_of(raw: Option<&str>) -> usize {
    let text = raw.map_or("", str::trim);
    match text.parse::<i64>() {
        Ok(n) => usize::try_from(n.clamp(1, 200)).unwrap_or(DEFAULT_LIMIT),
        Err(_) if raw.is_some() && text.is_empty() => 1,
        Err(_) => DEFAULT_LIMIT,
    }
}

pub(super) async fn wakes(ctx: Ctx) -> Reply {
    let wake = service(&ctx)?;
    let scope = key(&ctx)?;
    let thread = thread(&ctx)?;
    let limit = limit_of(ctx.query("limit"));
    let rows = blocking(move || lock(&wake).list(&scope, Some(limit), Some(&thread))).await?;
    let docs = rows.map_err(|_| ApiError::internal())?;
    Ok(json_reply(
        200,
        &json!({ "wakes": to_value(docs).unwrap_or_default() }),
    ))
}

/// The passive notices of a thread (`*`: every thread) since `since`, at most a day back: the
/// digest the prompt hook prints.
pub(super) async fn notices(ctx: Ctx) -> Reply {
    let wake = service(&ctx)?;
    let scope = key(&ctx)?;
    let thread = thread(&ctx)?;
    let floor = now_ms() - NOTICE_WINDOW_MS;
    let since = ctx
        .query("since")
        .and_then(parse_ms)
        .map_or(floor, |ms| ms.max(floor));
    let digest = blocking(move || lock(&wake).notices(&scope, &thread, since)).await?;
    let digest = digest.map_err(|_| ApiError::internal())?;
    Ok(json_reply(200, &to_value(digest).unwrap_or_default()))
}

pub(super) async fn listeners(ctx: Ctx) -> Reply {
    let wake = service(&ctx)?;
    let scope = key(&ctx)?;
    let open = lock(&wake).listeners(&scope);
    Ok(json_reply(200, &to_value(open).unwrap_or_default()))
}

fn has_thread(board: Option<Board>, thread: &str) -> bool {
    board.is_some_and(|b| b["threads"].get(thread).is_some_and(|t| !t.is_null()))
}

pub(super) async fn submit(ctx: Ctx) -> Reply {
    let wake = service(&ctx)?;
    let scope = key(&ctx)?;
    let body = ctx.json().await?;
    let home = ctx.svc.paths.coord_home.clone();
    let known = |scope: &str, thread: &str| {
        has_thread(ctx.svc.boards().board(scope), thread)
            || has_thread(read_board(&home, scope).ok().flatten(), thread)
    };
    let input = external_wake(&scope, &body, &known).map_err(|e| refusal(&e))?;
    let (thread, reason) = (input.thread.clone(), input.reason.clone());
    let window = blocking(move || {
        let mut service = lock(&wake);
        service
            .submit(&[input])
            .map(|_| service.engine().window_ms())
    })
    .await?
    .map_err(|_| ApiError::internal())?;
    Ok(json_reply(
        200,
        &json!({ "queued": true, "scope": scope, "thread": thread, "reason": reason, "windowMs": window }),
    ))
}
