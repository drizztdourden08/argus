//! The wake routes. The stream is what a thread listens on, usually through `argus wake-stream`:
//!
//! - `GET /api/p/:key/wakes/stream?thread=<id|*>&v=2&id=<label>` sends `ready`
//!   `{scope, thread, windowMs}` first, then one `wake` `{id, scope, thread, at, lines, count}`
//!   per coalesced event. `*` hears every thread. An event nobody heard is replayed once to the
//!   first v2 listener of 24 hours. `v=2` makes the listener unique per project and thread key:
//!   a newer one replaces the older, which gets `replaced` `{reason: "newer listener"}` and is
//!   closed. Without `v=2` the stream is legacy: served, never replacing or replaced, not
//!   counted and given no replay.
//! - `GET /api/p/:key/wakes/listeners` lists the open streams `[{thread, since, label, v}]`
//! - `GET /api/p/:key/wakes?thread=<id|*>&limit=<n>` is `{wakes}`, the rows the page mirrors
//! - `GET /api/p/:key/wakes/notices?thread=<id|*>&since=<iso>` is `{count, kinds, newest, line}`,
//!   the passive notices the wake policy kept off the listener (at most a day back); the prompt
//!   hook prints `line`
//! - `POST /api/p/:key/wakes {thread, reason, text}` wakes from outside the policy
//!   (`lead-message`, `lead-info`, `thread-message`, `wake-fault`); the thread must exist on the
//!   board, and a passive reason is recorded as a notice only
//! - `GET /api/p/:key/presence?thread=<id>&tab=<id>` is held open by a board page opened with
//!   `?thread=`; `GET /api/presence` is `{open}`, the threads with an open board
//! - `GET /api/wake-faults[?all=1]` is `{faults}`
//! - `POST /api/wake-faults/clear {project, thread, note, by}` clears one; never from a
//!   browser, and only for the Overlord

mod faults;
mod listing;
mod presence;
mod stream;

use argus_core::{ApiError, ErrorCode};
use argus_wake::external::{RequestError, stream_thread};
use argus_wake::service::driver::SharedWake;

use crate::{Ctx, RouteOpts, Router};

const POST_BYTES: usize = 8 * 1024;
const CLEAR_BYTES: usize = 4 * 1024;

fn service(ctx: &Ctx) -> Result<SharedWake, ApiError> {
    ctx.svc
        .wake
        .clone()
        .ok_or_else(|| ApiError::unavailable("the wake service is not running"))
}

fn refusal(e: &RequestError) -> ApiError {
    let code = match e {
        RequestError::BadRequest(_) => ErrorCode::BadRequest,
        RequestError::Forbidden(_) => ErrorCode::Forbidden,
        RequestError::NotFound(_) => ErrorCode::NotFound,
    };
    ApiError::new(code, e.message())
}

/// The `thread` query: a thread id, or `*` when none is given.
fn thread(ctx: &Ctx) -> Result<String, ApiError> {
    stream_thread(ctx.query("thread")).map_err(|e| refusal(&e))
}

/// Registers the wake routes.
pub fn register(r: &mut Router) {
    let post = RouteOpts {
        max_bytes: POST_BYTES,
        ..RouteOpts::default()
    };
    let clear = RouteOpts {
        max_bytes: CLEAR_BYTES,
        ..RouteOpts::default()
    };
    r.get("/api/p/:key/wakes/stream", stream::stream);
    r.add("POST", "/api/p/:key/wakes", post, listing::submit);
    r.get("/api/p/:key/presence", presence::presence);
    r.get("/api/presence", presence::open);
    r.get("/api/p/:key/wakes/listeners", listing::listeners);
    r.get("/api/p/:key/wakes/notices", listing::notices);
    r.get("/api/wake-faults", faults::list);
    r.add("POST", "/api/wake-faults/clear", clear, faults::clear);
    r.get("/api/p/:key/wakes", listing::wakes);
}
