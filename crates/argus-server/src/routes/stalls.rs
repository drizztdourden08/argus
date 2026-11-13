//! The stalls of the stall watch (`argus_wake::stalls`). The page reads the docs of
//! `_portfolio/stalls`; the CLI uses these routes:
//!
//! - `GET /api/stalls[?project=<key>]` is `{stalls, text}`: the open stalls and one line each
//! - `POST /api/stalls/ack {project, note, by}` acknowledges every open stall of a project; never
//!   from a browser, and only for the Overlord. The answer is `{acked, text}`; a project with no
//!   open stall answers 409.

use argus_core::ids::is_key;
use argus_core::{ApiError, ErrorCode};
use argus_wake::board::BoardView;
use argus_wake::external::{RequestError, overlord_only};
use argus_wake::service::driver::lock;
use argus_wake::stalls::StallInfo;
use argus_wake::text::one_line;
use serde_json::{Value, json, to_value};

use super::blocking::blocking;
use super::wake::{overlord_call, service};
use crate::reply::json_reply;
use crate::router::Reply;
use crate::{Ctx, Router};

const FROM_PAGE: &str = "a stall cannot be acknowledged from the page: only the Overlord acknowledges it, with `coord stall ack`";
const NOTE_MIN: usize = 8;
const NOTE_MAX: usize = 300;

/// One line per stall; an acknowledged one says what was done about it.
fn lines(stalls: &[StallInfo]) -> String {
    let line = |s: &StallInfo| match &s.ack_note {
        Some(note) => format!("{} (acked: {note})", s.text),
        None => s.text.clone(),
    };
    stalls.iter().map(line).collect::<Vec<_>>().join("\n")
}

async fn list(ctx: Ctx) -> Reply {
    let wake = service(&ctx)?;
    let project = ctx.query("project").filter(|p| !p.is_empty());
    if let Some(bad) = project.filter(|p| !is_key(p)) {
        return Err(ApiError::bad_request(format!(
            "invalid project key \"{bad}\""
        )));
    }
    let project = project.map(str::to_owned);
    let stalls = blocking(move || lock(&wake).stalls(project.as_deref())).await?;
    Ok(json_reply(
        200,
        &json!({ "stalls": to_value(&stalls).unwrap_or_default(), "text": lines(&stalls) }),
    ))
}

/// A checked acknowledgement.
struct AckRequest {
    project: String,
    note: String,
    by: String,
}

fn field<'a>(body: &'a Value, name: &str) -> &'a str {
    body[name].as_str().unwrap_or("")
}

fn request(
    body: &Value,
    overlord: &str,
    portfolio: Option<&BoardView>,
) -> Result<AckRequest, RequestError> {
    let project = field(body, "project");
    if !is_key(project) {
        return Err(RequestError::BadRequest(
            "project must be a project key".to_owned(),
        ));
    }
    let note = one_line(field(body, "note"));
    if !(NOTE_MIN..=NOTE_MAX).contains(&note.chars().count()) {
        return Err(RequestError::BadRequest(format!(
            "note must be {NOTE_MIN} to {NOTE_MAX} characters: say what you did about the stall"
        )));
    }
    let by = field(body, "by");
    overlord_only(by, overlord, portfolio, "acknowledge a stall")?;
    Ok(AckRequest {
        project: project.to_owned(),
        note,
        by: by.to_owned(),
    })
}

fn acked_text(project: &str, count: usize) -> String {
    let noun = if count == 1 { "stall" } else { "stalls" };
    format!("acked {count} {noun} of {project}")
}

async fn ack(ctx: Ctx) -> Reply {
    let (wake, req) = overlord_call(&ctx, FROM_PAGE, request).await?;
    let project = req.project.clone();
    let by = format!("{}/{}", ctx.svc.overlord(), req.by);
    let acked = blocking(move || lock(&wake).ack_stalls(&req.project, &by, &req.note)).await?;
    if acked.is_empty() {
        return Err(ApiError::new(
            ErrorCode::Conflict,
            format!("{project} has no open stall"),
        ));
    }
    Ok(json_reply(
        200,
        &json!({
            "acked": to_value(&acked).unwrap_or_default(),
            "text": acked_text(&project, acked.len()),
        }),
    ))
}

/// Registers the stall routes.
pub fn register(r: &mut Router) {
    r.get("/api/stalls", list);
    r.post("/api/stalls/ack", ack);
}
