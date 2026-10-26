//! An owner post as the board will take it: checked, author forced to the owner, server fields
//! dropped. The owner's page is held only to the 50000-character safety net; the CLI and MCP
//! (what a model writes) keep the short limits of `limits.mjs`.

use argus_board::limits::{OWNER_TEXT_MAX, limit};
use argus_board::sync::{is_op_kind, is_reply_kind};
use argus_core::clock::{now_iso, parse_ms};
use argus_core::ids::{PORTFOLIO, is_key};
use argus_core::{ApiError, ErrorCode, js};
use serde_json::{Map, Value, json};

use crate::posts::{Source, new_post_id};

const MAX_ENTRY_BYTES: usize = 256 * 1024;
const RESULTS: [&str; 4] = ["pass", "fail", "partial", "blocked"];
const PORTFOLIO_KINDS: [&str; 2] = ["decision", "followup"];
const SERVER_FIELDS: [&str; 12] = [
    "appliedAt",
    "current",
    "by",
    "results",
    "updatedAt",
    "updatedBy",
    "createdBy",
    "rev",
    "rank",
    "deliveredTo",
    "pendingFor",
    "deliveredAt",
];

/// The refusal of a fail, partial or blocked result without a note.
pub const RESULT_NOTE_REQUIRED: &str =
    "a note is required for fail, partial and blocked: say what you saw";

/// A fail, partial or blocked result must say what the owner saw.
pub(crate) fn result_note_missing(e: &Value) -> bool {
    let outcome = e["outcome"].as_str().unwrap_or_default();
    ["fail", "partial", "blocked"].contains(&outcome)
        && js::trim(e["text"].as_str().unwrap_or_default()).is_empty()
}

fn bad(m: impl Into<String>) -> ApiError {
    ApiError::bad_request(m)
}

fn is_post_id(id: &str, portfolio: bool) -> bool {
    let Some(rest) = id.strip_prefix("O-") else {
        return false;
    };
    let part = |s: &str| {
        !s.is_empty()
            && s.bytes()
                .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit())
    };
    match rest.split_once('-') {
        Some((time, tail)) => part(time) && tail.len() == 4 && part(tail),
        None => portfolio && part(rest),
    }
}

fn optional_string(e: &Map<String, Value>, field: &str) -> Result<(), ApiError> {
    match e.get(field) {
        None | Some(Value::Null | Value::String(_)) => Ok(()),
        Some(_) => Err(bad(format!("{field} must be a string"))),
    }
}

fn text_limit(kind: &str) -> usize {
    let name = match kind {
        "followup" | "decision" | "result" => kind,
        _ => "answer",
    };
    limit(name).unwrap_or(OWNER_TEXT_MAX)
}

fn check_text(e: &mut Map<String, Value>, source: Source) -> Result<(), ApiError> {
    let text = match e.get("text") {
        None | Some(Value::Null) => return Ok(()),
        Some(Value::String(s)) => js::trim(s).to_owned(),
        Some(_) => return Err(bad("text must be a string")),
    };
    let owner = source == Source::Page;
    let kind = e.get("kind").map(js::string).unwrap_or_default();
    let max = if owner {
        OWNER_TEXT_MAX
    } else {
        text_limit(&kind)
    };
    let size = js::len(&text);
    if size > max {
        return Err(bad(if owner {
            format!(
                "this text is {size} characters and one post takes {max} at most: shorten it or split it into two posts"
            )
        } else {
            format!("{kind} text is {size} chars, the limit is {max}")
        }));
    }
    e.insert("text".into(), json!(text));
    Ok(())
}

fn kinds_message(portfolio: bool) -> String {
    let kinds: Vec<&str> = if portfolio {
        PORTFOLIO_KINDS.to_vec()
    } else {
        argus_board::sync::REPLY_KINDS
            .iter()
            .chain(argus_board::sync::OP_KINDS.iter())
            .copied()
            .collect()
    };
    format!("entry kind must be one of {}", kinds.join(", "))
}

fn check_identity(e: &mut Map<String, Value>, portfolio: bool) -> Result<(), ApiError> {
    if e.get("id").is_none_or(Value::is_null) {
        e.insert("id".into(), json!(new_post_id()));
    }
    let id = &e["id"];
    if !id.as_str().is_some_and(|s| is_post_id(s, portfolio)) {
        return Err(bad(format!(
            "entry id \"{}\" must look like O-<base36 time>-<4 chars>",
            js::string(id)
        )));
    }
    let kind = e.get("kind").and_then(Value::as_str).unwrap_or_default();
    let known = if portfolio {
        PORTFOLIO_KINDS.contains(&kind)
    } else {
        is_reply_kind(kind) || is_op_kind(kind)
    };
    if known {
        Ok(())
    } else {
        Err(bad(kinds_message(portfolio)))
    }
}

fn stamp_owner(e: &mut Map<String, Value>, source: Source, via: Option<&str>) {
    e.insert("author".into(), json!("human"));
    let created = e
        .get("createdAt")
        .and_then(Value::as_str)
        .filter(|s| parse_ms(s).is_some())
        .map(str::to_owned);
    e.insert("createdAt".into(), json!(created.unwrap_or_else(now_iso)));
    e.insert(
        "via".into(),
        json!(via.filter(|v| !v.is_empty()).unwrap_or(source.as_str())),
    );
}

fn finish_portfolio(mut e: Map<String, Value>) -> Result<Value, ApiError> {
    let project = e.get("project").and_then(Value::as_str);
    if !project.is_some_and(|p| p == "all" || is_key(p)) {
        return Err(bad("project must be a project key or \"all\""));
    }
    if !e.get("text").is_some_and(js::truthy) {
        return Err(bad(format!(
            "{} text is required",
            e.get("kind").map(js::string).unwrap_or_default()
        )));
    }
    e.remove("status");
    e.remove("outcome");
    Ok(Value::Object(e))
}

fn finish_project(mut e: Map<String, Value>) -> Result<Value, ApiError> {
    let kind = e
        .get("kind")
        .and_then(Value::as_str)
        .unwrap_or_default()
        .to_owned();
    if kind == "rerun" && !e.get("replyTo").is_some_and(js::truthy) {
        return Err(bad("a rerun names the test in replyTo"));
    }
    if kind == "result"
        && !e
            .get("outcome")
            .and_then(Value::as_str)
            .is_some_and(|o| RESULTS.contains(&o))
    {
        return Err(bad(format!(
            "a result's outcome must be one of {}",
            RESULTS.join(", ")
        )));
    }
    if kind != "result" {
        e.remove("outcome");
    }
    if (kind == "followup" || kind == "decision") && !e.get("text").is_some_and(js::truthy) {
        return Err(bad(format!("{kind} text is required")));
    }
    e.insert("status".into(), json!("new"));
    Ok(Value::Object(e))
}

/// Validates an owner post for `scope`.
///
/// # Errors
/// 400 with the Node message, or 413 for a body over 256 KB.
pub(crate) fn validate_entry(
    scope: &str,
    body: &Value,
    source: Source,
    via: Option<&str>,
) -> Result<Value, ApiError> {
    let Some(obj) = body.as_object() else {
        return Err(bad("an entry must be a JSON object"));
    };
    if serde_json::to_string(body).map_or(0, |s| s.len()) > MAX_ENTRY_BYTES {
        return Err(ApiError::new(
            ErrorCode::TooLarge,
            format!("an entry is at most {MAX_ENTRY_BYTES} bytes"),
        ));
    }
    let mut e = obj.clone();
    let portfolio = scope == PORTFOLIO;
    check_identity(&mut e, portfolio)?;
    for f in SERVER_FIELDS {
        e.remove(f);
    }
    for f in ["replyTo", "item", "thread"] {
        optional_string(&e, f)?;
    }
    check_text(&mut e, source)?;
    stamp_owner(&mut e, source, via);
    if portfolio {
        finish_portfolio(e)
    } else {
        finish_project(e)
    }
}
