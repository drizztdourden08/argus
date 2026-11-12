//! The pure rules of the sub-agent hooks that the dashboard side shares: where a sub-agent writes
//! its transcript, the agent id Claude prints for a launched agent, the model a transcript shows,
//! and what a finished agent leaves on its record. The hooks themselves are part of the coord port.

use crate::fs::FileSystem;
use crate::iso::format_iso;
use crate::record::{Record, filled, is_live, is_set, truthy};
use crate::text::{cut, same_model};
use serde_json::Value;
use std::path::{Path, PathBuf};

const PATH_MAX: usize = 1000;
const SESSION_MAX: usize = 120;
const RESULT_MAX: usize = 300;
const HEAD_BYTES: u64 = 512 * 1024;

fn path_chars(text: &str, extra: &[char]) -> bool {
    !text.is_empty()
        && text
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '_' || extra.contains(&c))
}

/// Whether text is a Claude sub-agent id: letters, digits, `_`, `.` and `-`, at most 120.
#[must_use]
pub fn valid_claude_id(id: &str) -> bool {
    id.chars().count() <= SESSION_MAX && path_chars(id, &['.', '-'])
}

/// Where a sub-agent writes its own transcript: next to the parent session's transcript
/// (`<dir>/<sessionId>.jsonl`), in `<dir>/<sessionId>/subagents/agent-<agentId>.jsonl`. `None`
/// when the payload lacks a piece or the ids hold characters that must not reach a path.
#[must_use]
pub fn subagent_transcript(
    parent: Option<&str>,
    session_id: Option<&str>,
    agent_id: &str,
) -> Option<PathBuf> {
    let parent = cut(parent?, PATH_MAX);
    let session = cut(session_id?, SESSION_MAX);
    if parent.is_empty() || !path_chars(&session, &['.', '-']) || !path_chars(agent_id, &['.', '-'])
    {
        return None;
    }
    let dir = Path::new(&parent).parent()?;
    Some(
        dir.join(&session)
            .join("subagents")
            .join(format!("agent-{agent_id}.jsonl")),
    )
}

fn id_after_label(text: &str) -> Option<String> {
    text.match_indices("agentId:").find_map(|(at, label)| {
        let rest = text[at + label.len()..].trim_start();
        let id: String = rest
            .chars()
            .take_while(|c| c.is_ascii_alphanumeric() || matches!(c, '_' | '.' | '-'))
            .collect();
        (!id.is_empty()).then_some(id)
    })
}

/// The agent id Claude prints for a sub-agent it launched: the structured field of the tool
/// response when it has one, else the `agentId: <id>` line of the result text.
#[must_use]
pub fn launched_id(response: &Value) -> Option<String> {
    if !truthy(response) {
        return None;
    }
    let direct = response
        .as_object()
        .and_then(|o| o.get("agentId").or_else(|| o.get("agent_id")))
        .and_then(Value::as_str);
    if let Some(id) = direct.filter(|id| valid_claude_id(id)) {
        return Some(id.to_owned());
    }
    let text = match response {
        Value::String(s) => s.clone(),
        other => {
            let body = other
                .get("content")
                .filter(|c| !c.is_null())
                .unwrap_or(other);
            body.to_string()
        }
    };
    id_after_label(&text)
}

fn head_entries(fs: &dyn FileSystem, file: &Path) -> Option<Vec<Value>> {
    let size = fs.stat(file)?.size.min(HEAD_BYTES);
    let bytes = fs.read_at(file, 0, usize::try_from(size).ok()?).ok()?;
    Some(
        String::from_utf8_lossy(&bytes)
            .split('\n')
            .filter_map(|line| serde_json::from_str::<Value>(line).ok())
            .collect(),
    )
}

fn is_user(entry: &Value) -> bool {
    let by_role = entry.pointer("/message/role").and_then(Value::as_str);
    let by_type = entry.get("type").and_then(Value::as_str);
    by_role == Some("user") || by_type == Some("user")
}

fn text_of(content: &Value) -> Option<String> {
    match content {
        Value::String(s) => Some(s.clone()),
        Value::Array(parts) => {
            let texts: Vec<&str> = parts
                .iter()
                .filter_map(|p| p.get("text").and_then(Value::as_str))
                .collect();
            (!texts.is_empty()).then(|| texts.join("\n"))
        }
        _ => None,
    }
}

/// The text of the first user line of a transcript (the prompt a sub-agent was given), read from
/// the head of the file only.
#[must_use]
pub fn transcript_first_prompt(fs: &dyn FileSystem, file: &Path) -> Option<String> {
    head_entries(fs, file)?
        .iter()
        .filter(|entry| is_user(entry))
        .find_map(|entry| text_of(entry.pointer("/message/content")?))
        .filter(|text| !text.trim().is_empty())
}

/// The model of the first assistant line of a transcript, read from the head of the file only.
#[must_use]
pub fn transcript_model(fs: &dyn FileSystem, file: &Path) -> Option<String> {
    head_entries(fs, file)?
        .into_iter()
        .filter(|entry| {
            let by_role = entry.pointer("/message/role").and_then(Value::as_str);
            let by_type = entry.get("type").and_then(Value::as_str);
            by_role == Some("assistant") || by_type == Some("assistant")
        })
        .find_map(|entry| {
            let model = entry.pointer("/message/model")?.as_str()?;
            (!model.is_empty() && !model.starts_with('<')).then(|| model.to_owned())
        })
}

/// Whether a last message reads as a failure: it starts with `API Error` or `Error:`, in any case.
#[must_use]
pub fn failed(last: &str) -> bool {
    let head = last.trim_start().to_ascii_lowercase();
    head.starts_with("api error") || head.starts_with("error:")
}

/// What a finished sub-agent leaves on its record: status, the answer, the model it ran on. A
/// record the dashboard closed for silence (the reaper marks it `reaped`) that finishes after all
/// is corrected here.
pub fn close_record(agent: &mut Record, last: &str, used: Option<&str>, now_ms: i64) {
    let reaped =
        is_set(agent, "reaped") && !is_set(agent, "archivedAt") && !is_set(agent, "deletedAt");
    if is_live(agent) || reaped {
        let status = if failed(last) { "failed" } else { "done" };
        agent.insert("status".into(), Value::from(status));
        agent.insert("finishedAt".into(), Value::from(format_iso(now_ms)));
        if !last.is_empty() {
            let result: String = last.chars().take(RESULT_MAX).collect();
            agent.insert("result".into(), Value::from(result));
        }
        if is_set(agent, "reaped") {
            agent.shift_remove("reaped");
            agent.insert("note".into(), Value::Null);
        }
    }
    if let Some(used) = used {
        record_model_used(agent, used);
    }
}

fn record_model_used(agent: &mut Record, used: &str) {
    agent.insert("modelUsed".into(), Value::from(used));
    match filled(agent, "model").map(str::to_owned) {
        Some(asked) => {
            agent.insert(
                "modelMismatch".into(),
                Value::Bool(!same_model(&asked, used)),
            );
        }
        None => {
            agent.insert("model".into(), Value::from(used));
        }
    }
}
