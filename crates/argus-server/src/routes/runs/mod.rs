//! The run routes (DESIGN 3.3). A page or the CLI passes only a launch name: no route takes a
//! command, a working directory or an env.
//!
//! - `GET /api/p/:key/launches` is `{defs}`
//! - `POST /api/p/:key/runs {name, by?, visible?, agent?, reason?}` is `{run, already}`
//! - `GET /api/p/:key/runs` and `GET /api/runs` (`status`, `name`, `limit`) are `{runs}`
//! - `GET /api/p/:key/runs/:rid` is `{run}`; `POST .../stop` is `{run}`
//! - `POST .../stdin {text}` types one line into the process: `{ok, bytes}`
//! - `GET .../logs` (`after`, `tail`, `limit`, `stream`) is `{lines, next, dropped}`; `GET
//!   .../logs/stream` is the live tail as server-sent events
//! - `POST .../attachments?caption=` takes an image body: `{attachment}`; `GET` lists them
//!
//! The caller is `human` when it is the owner's page and `cli` otherwise. Shadow mode answers
//! `read_only` to everything that starts, stops, types or attaches.

use std::sync::Arc;

use argus_core::{ApiError, ErrorCode};
use argus_launchers::{RunError, RunService, check_key, check_rid};

use crate::adapters::runs::RunHost;
use crate::{AuthKind, Ctx, Router};

mod attach;
mod control;
mod logs;
mod tail;

pub(crate) fn service(ctx: &Ctx) -> Result<Arc<RunService<RunHost>>, ApiError> {
    ctx.svc
        .runs
        .clone()
        .ok_or_else(|| ApiError::unavailable("the run service is not running"))
}

pub(crate) fn refusal(e: &RunError) -> ApiError {
    let code = match e.status {
        400 => ErrorCode::BadRequest,
        404 => ErrorCode::UnknownRun,
        409 => ErrorCode::Conflict,
        413 => ErrorCode::TooLarge,
        415 => ErrorCode::UnsupportedMedia,
        503 => ErrorCode::Unavailable,
        _ => ErrorCode::Internal,
    };
    ApiError::new(code, e.message.clone())
}

fn unknown_run(key: &str, rid: &str) -> ApiError {
    ApiError::new(ErrorCode::UnknownRun, format!("no run {rid} in {key}"))
}

fn writable(ctx: &Ctx) -> Result<(), ApiError> {
    if ctx.svc.shadow {
        return Err(ApiError::new(
            ErrorCode::ReadOnly,
            "shadow mode: Argus starts and changes no run",
        ));
    }
    Ok(())
}

fn is_page(ctx: &Ctx) -> bool {
    ctx.auth == Some(AuthKind::Page)
}

/// Who acted, as the run records it.
fn actor(ctx: &Ctx) -> &'static str {
    if is_page(ctx) { "human" } else { "cli" }
}

/// The project key and run id of the route, checked.
fn ids(ctx: &Ctx) -> Result<(String, String), ApiError> {
    let key = check_key(ctx.param("key")).map_err(|e| refusal(&e))?;
    let rid = check_rid(ctx.param("rid")).map_err(|e| refusal(&e))?;
    Ok((key.to_owned(), rid.to_owned()))
}

/// Registers the run routes.
pub fn register(r: &mut Router) {
    control::register(r);
    logs::register(r);
    tail::register(r);
    attach::register(r);
}
