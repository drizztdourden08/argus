//! `notify_owner` over the notification service: a ping from the MCP client, as a thread.

use argus_mcp::{NotifyRequest, ToolError};
use argus_notify::{By, Level, PingRequest};
use serde_json::{Value, json};

use super::errors::internal;
use crate::Services;

const THREAD_MAX: usize = 60;

fn ping_of(request: &NotifyRequest) -> Result<PingRequest, ToolError> {
    let level = Level::parse(&request.level)
        .ok_or_else(|| ToolError::bad_request("level is info, ping, urgent"))?;
    Ok(PingRequest {
        title: request.title.clone(),
        text: request.text.clone(),
        level,
        project: request.project.clone(),
        thread: Some(request.thread.chars().take(THREAD_MAX).collect()),
        by: By::Thread,
        url: None,
    })
}

/// Records the ping and shows the toast its level asks for.
pub async fn notify(svc: &Services, request: &NotifyRequest) -> Result<Value, ToolError> {
    let service = svc
        .notify
        .clone()
        .ok_or_else(|| ToolError::unavailable("notifications are off"))?;
    let ping = ping_of(request)?;
    let outcome = tokio::task::spawn_blocking(move || service.send(ping))
        .await
        .map_err(|_| internal())?;
    let note = outcome.notification.as_ref();
    Ok(json!({
        "recorded": outcome.is_recorded(),
        "id": note.map(|n| n.id.clone()),
        "level": note.map(|n| n.level.as_str()),
        "toast": outcome.toast.as_str(),
        "why": outcome.why,
    }))
}
