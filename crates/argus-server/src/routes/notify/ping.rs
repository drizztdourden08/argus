//! The two routes that ping the owner: a thread's `POST /api/notify` and the page's test button.

use argus_board::names::MAIN_KEY;
use argus_core::{ApiError, ErrorCode};
use argus_native::TEXT_MAX;
use argus_notify::{By, Level, PingRequest};
use serde_json::Value;

use super::{LEVEL_WORDS, answer, service};
use crate::router::Reply;
use crate::routes::blocking::blocking;
use crate::{AuthKind, Ctx};

const THREAD_MAX: usize = 60;
const KEY_MAX: usize = 40;
const NO_TEXT: &str = "send {text, title?, level?, project?}: a notification needs text";
const TEST_TITLE: &str = "Argus: test notification";
const TEST_TEXT: &str =
    "If you can read this as a toast, notifications from your threads will reach you.";

fn text_of(body: &Value) -> Result<String, ApiError> {
    let text = body["text"].as_str().map(str::trim).unwrap_or_default();
    if text.is_empty() {
        return Err(ApiError::bad_request(NO_TEXT));
    }
    Ok(text.to_owned())
}

fn level_of(body: &Value) -> Result<Level, ApiError> {
    let named = match &body["level"] {
        Value::Null => Some(Level::Ping),
        Value::String(name) if name.is_empty() => Some(Level::Ping),
        Value::String(name) => Level::parse(name),
        _ => None,
    };
    named.ok_or_else(|| ApiError::bad_request(format!("level is {LEVEL_WORDS}")))
}

/// `^[a-z0-9][a-z0-9-]{0,39}$`.
fn is_project_key(text: &str) -> bool {
    text.chars().count() <= KEY_MAX && argus_core::ids::is_key(text)
}

fn project_of(body: &Value) -> Result<Option<String>, ApiError> {
    let Some(project) = body["project"].as_str().filter(|p| !p.is_empty()) else {
        return Ok(None);
    };
    if is_project_key(project) {
        return Ok(Some(project.to_owned()));
    }
    let shown: String = project.chars().take(KEY_MAX).collect();
    Err(ApiError::bad_request(format!(
        "\"{shown}\" is not a project key"
    )))
}

fn thread_of(body: &Value) -> Option<String> {
    let thread = body["thread"].as_str()?.trim();
    (!thread.is_empty()).then(|| thread.chars().take(THREAD_MAX).collect())
}

fn is_overlord(ctx: &Ctx, project: Option<&str>, thread: Option<&str>) -> bool {
    let (Some(MAIN_KEY), Some(thread)) = (project, thread) else {
        return false;
    };
    let board = ctx.svc.boards().board(MAIN_KEY);
    board.is_some_and(|b| b["threads"][thread]["role"] == "coordinator")
}

fn by_of(ctx: &Ctx, body: &Value, project: Option<&str>, thread: Option<&str>) -> By {
    if ctx.auth == Some(AuthKind::Page) {
        By::Page
    } else if body["by"] == "overlord" && is_overlord(ctx, project, thread) {
        By::Overlord
    } else {
        By::Thread
    }
}

fn too_long(text: &str) -> Option<ApiError> {
    let length = text.encode_utf16().count();
    (length > TEXT_MAX).then(|| {
        ApiError::bad_request(format!(
            "text is {length} chars, the limit is {TEXT_MAX}: shorten it"
        ))
    })
}

fn request_of(ctx: &Ctx, body: &Value) -> Result<PingRequest, ApiError> {
    let text = text_of(body)?;
    let level = level_of(body)?;
    let project = project_of(body)?;
    let thread = thread_of(body);
    if ctx.auth != Some(AuthKind::Page) {
        if let Some(error) = too_long(&text) {
            return Err(error);
        }
    }
    let by = by_of(ctx, body, project.as_deref(), thread.as_deref());
    Ok(PingRequest {
        title: body["title"].as_str().map(str::to_owned),
        text,
        level,
        project,
        thread,
        by,
        url: None,
    })
}

/// `POST /api/notify`.
pub async fn send(ctx: Ctx) -> Reply {
    let notify = service(&ctx)?;
    let body = ctx.json().await?;
    let request = request_of(&ctx, &body)?;
    let outcome = blocking(move || notify.send(request)).await?;
    Ok(answer(&outcome))
}

/// `POST /api/notify/test`.
pub async fn test(ctx: Ctx) -> Reply {
    let notify = service(&ctx)?;
    if ctx.auth != Some(AuthKind::Page) {
        return Err(ApiError::new(
            ErrorCode::Forbidden,
            "only the owner's page sends a test notification",
        ));
    }
    let request = PingRequest {
        title: Some(TEST_TITLE.to_owned()),
        by: By::Page,
        ..PingRequest::new(TEST_TEXT)
    };
    let outcome = blocking(move || notify.send(request)).await?;
    Ok(answer(&outcome))
}
