//! The transcript of the hosted Overlord, for the owner's page only:
//!
//! - `GET /api/overlord/transcript?session=<id>&after=<n>` is `{session, lines, next}`: the events
//!   after the first `n` lines as JSON (`next` is the `after` of the next read)
//! - `GET /api/overlord/transcript/stream?session=<id>&after=<n>` is the same lines as server-sent
//!   `line` events (`{session, index, event}`, the `id` is the index), replayed from `after` or from
//!   `Last-Event-ID`, then live. It is told of a new line by the transcript feed; nothing polls.
//!
//! With no `session` the newest transcript file is used. The files are already redacted.

use std::sync::{Arc, Mutex, PoisonError};

use argus_core::ApiError;
use argus_overlord::ports::{TranscriptFault, TranscriptStore};
use argus_overlord::transcript::{Page, newest, valid_session};
use serde_json::{Value, json};

use self::follow::Follow;
use super::blocking::blocking;
use super::overlord::web_only;
use crate::adapters::overlord::feed::FeedListening;
use crate::adapters::overlord::transcript::FileTranscripts;
use crate::reply::json_reply;
use crate::router::Reply;
use crate::sse::open_stream;
use crate::{Ctx, Router};

mod follow;

fn store_error(_: TranscriptFault) -> ApiError {
    ApiError::internal()
}

fn number(ctx: &Ctx, name: &str) -> Option<usize> {
    ctx.query(name).and_then(|n| n.trim().parse().ok())
}

fn start_cursor(ctx: &Ctx) -> usize {
    let last = ctx
        .header("last-event-id")
        .and_then(|v| v.trim().parse().ok());
    number(ctx, "after").or(last).unwrap_or(0)
}

fn pick(store: &FileTranscripts, given: Option<&str>) -> Result<Option<String>, ApiError> {
    match given.filter(|s| !s.is_empty()) {
        Some(id) if valid_session(id) => Ok(Some(id.to_owned())),
        Some(_) => Err(ApiError::bad_request("that is not a session id")),
        None => Ok(newest(&store.list().map_err(store_error)?)),
    }
}

async fn session_of(ctx: &Ctx) -> Result<Option<String>, ApiError> {
    let home = ctx.svc.paths.home.clone();
    let given = ctx.query("session").map(str::to_owned);
    blocking(move || pick(&FileTranscripts::new(&home), given.as_deref())).await?
}

async fn page_of(ctx: &Ctx, session: &str, after: usize) -> Result<Page, ApiError> {
    let home = ctx.svc.paths.home.clone();
    let session = session.to_owned();
    blocking(move || {
        FileTranscripts::new(&home)
            .read_after(&session, after)
            .map_err(store_error)
    })
    .await?
}

fn as_event(line: &str) -> Value {
    serde_json::from_str(line).unwrap_or_else(|_| Value::String(line.to_owned()))
}

async fn read(ctx: Ctx) -> Reply {
    web_only(&ctx, false)?;
    let Some(session) = session_of(&ctx).await? else {
        return Ok(json_reply(
            200,
            &json!({ "session": null, "lines": [], "next": 0 }),
        ));
    };
    let page = page_of(&ctx, &session, number(&ctx, "after").unwrap_or(0)).await?;
    let lines: Vec<Value> = page.lines.iter().map(|l| as_event(l)).collect();
    Ok(json_reply(
        200,
        &json!({ "session": session, "lines": lines, "next": page.next }),
    ))
}

async fn stream(ctx: Ctx) -> Reply {
    web_only(&ctx, false)?;
    let session = session_of(&ctx)
        .await?
        .ok_or_else(|| ApiError::not_found("there is no Overlord transcript yet"))?;
    let after = start_cursor(&ctx);
    let held: Arc<Mutex<Option<FeedListening>>> = Arc::default();
    let dropper = Arc::clone(&held);
    let on_drop = Box::new(move || {
        dropper
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .take();
    });
    let (sender, res) = open_stream(ctx.svc.heartbeat, &ctx.svc.streams, Some(on_drop));
    let follow = Follow::new(sender, session.clone(), after);
    let live = Arc::clone(&follow);
    let listening = ctx.svc.overlord_feed.listen(move |e| live.live(e));
    *held.lock().unwrap_or_else(PoisonError::into_inner) = Some(listening);
    follow.replayed(&page_of(&ctx, &session, after).await?);
    Ok(res)
}

/// Registers the transcript routes.
pub(super) fn register(r: &mut Router) {
    r.get("/api/overlord/transcript", read);
    r.get("/api/overlord/transcript/stream", stream);
}
