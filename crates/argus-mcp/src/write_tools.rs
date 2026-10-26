//! The tools that act as the owner: posts, answers, decisions, test results, runs, notes to
//! the lead, and the ping.

use argus_core::js;
use serde_json::{Map, Value, json};

use crate::args::{arg_str, flag};
use crate::backend::{NotifyRequest, StartOpts};
use crate::call::Call;
use crate::error::ToolError;
use crate::ids::new_id;
use crate::json::{pick, text, truthy};
use crate::project::{need, require_project};

/// The refusal of a fail, partial or blocked result without a note.
pub const RESULT_NOTE_REQUIRED: &str =
    "a note is required for fail, partial and blocked: say what you saw";

fn arg(call: &Call, key: &str) -> Option<String> {
    arg_str(call.args, key).map(str::to_owned)
}

async fn post(call: &Call<'_>, fields: &[(&str, Option<String>)]) -> Result<Value, ToolError> {
    let mut entry = Map::new();
    entry.insert("id".into(), json!(new_id()));
    entry.insert("author".into(), json!("human"));
    for (name, value) in fields {
        if let Some(v) = value.as_ref().filter(|v| !v.is_empty()) {
            entry.insert((*name).to_owned(), json!(v));
        }
    }
    call.backend
        .post_entry(call.key(), Value::Object(entry), &call.via)
        .await
}

/// `post_entry`.
pub async fn post_entry(call: &Call<'_>) -> Result<Value, ToolError> {
    let kind = arg(call, "kind").unwrap_or_default();
    let has_text = arg_str(call.args, "text").is_some();
    let has_reply = arg_str(call.args, "replyTo").is_some();
    if matches!(kind.as_str(), "followup" | "decision") && !has_text {
        return Err(ToolError::bad_request(format!("a {kind} needs text")));
    }
    if matches!(kind.as_str(), "ack" | "done" | "pause" | "resume") && !has_reply {
        let of = if matches!(kind.as_str(), "pause" | "resume") {
            ": the thread"
        } else {
            ""
        };
        return Err(ToolError::bad_request(format!(
            "{kind} needs replyTo (the id it answers{of})"
        )));
    }
    let fields = [
        ("kind", Some(kind)),
        ("text", arg(call, "text")),
        ("replyTo", arg(call, "replyTo")),
        ("item", arg(call, "item")),
        ("thread", arg(call, "thread")),
    ];
    post(call, &fields).await
}

/// `answer`.
pub async fn answer(call: &Call<'_>) -> Result<Value, ToolError> {
    let entry = arg(call, "entry").unwrap_or_default();
    let key = call.key();
    need(
        call.backend.doc(key, "entries", &entry),
        &format!("entry {entry} in {key}"),
    )?;
    let kind = if flag(call.args, "decision") {
        "decision"
    } else {
        "followup"
    };
    let fields = [
        ("kind", Some(kind.to_owned())),
        ("text", arg(call, "text")),
        ("replyTo", Some(entry)),
    ];
    post(call, &fields).await
}

/// `decision`.
pub async fn decision(call: &Call<'_>) -> Result<Value, ToolError> {
    let fields = [
        ("kind", Some("decision".to_owned())),
        ("text", arg(call, "text")),
        ("item", arg(call, "item")),
        ("thread", arg(call, "thread")),
    ];
    post(call, &fields).await
}

fn note_missing(outcome: &str, note: &str) -> bool {
    matches!(outcome, "fail" | "partial" | "blocked") && js::trim(note).is_empty()
}

/// `record_test_result`.
pub async fn record_test_result(call: &Call<'_>) -> Result<Value, ToolError> {
    let test = arg(call, "test").unwrap_or_default();
    let key = call.key();
    need(
        call.backend.doc(key, "tests", &test),
        &format!("test {test} in {key}"),
    )?;
    let outcome = arg(call, "outcome").unwrap_or_default();
    let note = arg(call, "text");
    if note_missing(&outcome, note.as_deref().unwrap_or_default()) {
        return Err(ToolError::bad_request(RESULT_NOTE_REQUIRED));
    }
    let fields = [
        ("kind", Some("result".to_owned())),
        ("outcome", Some(outcome)),
        ("text", note),
        ("replyTo", Some(test)),
    ];
    post(call, &fields).await
}

/// `run_start`.
pub async fn run_start(call: &Call<'_>) -> Result<Value, ToolError> {
    let name = arg(call, "name").unwrap_or_default();
    let opts = StartOpts {
        by: call.via.clone(),
        agent: arg(call, "agent"),
        reason: arg(call, "reason"),
    };
    let (run, already) = call.backend.run_start(call.key(), &name, &opts).await?;
    Ok(json!({ "run": run, "already": already }))
}

/// `run_stop`.
pub async fn run_stop(call: &Call<'_>) -> Result<Value, ToolError> {
    let run = arg(call, "run").unwrap_or_default();
    let stopped = call.backend.run_stop(call.key(), &run, &call.via).await?;
    Ok(json!({ "run": stopped }))
}

/// `send_to_lead`: a project key, or `all`.
pub async fn send_to_lead(call: &Call<'_>) -> Result<Value, ToolError> {
    let target = arg_str(call.args, "project").or(call.ctx.default_project.as_deref());
    let Some(target) = target else {
        return Err(ToolError::bad_request(
            "project is required: a project key, or \"all\"",
        ));
    };
    if target != "all" {
        require_project(call.backend, Some(target))?;
    }
    let text = arg_str(call.args, "text").unwrap_or_default();
    call.backend
        .send_to_lead(target, text, flag(call.args, "decision"), &call.via)
        .await
}

/// `notify_owner`: speaks to the owner, so it does not post as him.
pub async fn notify_owner(call: &Call<'_>) -> Result<Value, ToolError> {
    let request = NotifyRequest {
        title: arg(call, "title"),
        text: arg(call, "text").unwrap_or_default(),
        level: arg(call, "level").unwrap_or_else(|| "ping".to_owned()),
        project: arg(call, "project").or_else(|| call.ctx.default_project.clone()),
        thread: call.via.clone(),
    };
    let sent = call.backend.notify(&request).await?;
    if !truthy(&sent, "recorded") {
        let why = text(&sent, "why").unwrap_or("the notification was not recorded");
        return Err(ToolError::bad_request(why));
    }
    let mut out = Map::new();
    out.insert("recorded".into(), json!(true));
    pick(&mut out, &sent, &["id", "level", "toast", "why"]);
    Ok(Value::Object(out))
}
