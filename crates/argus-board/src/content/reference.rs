//! The kinds that point at something: an id of a kind, and a person.

use argus_core::js;
use serde_json::Value;

use super::{Context, Rejection, reject};
use crate::registry::{Field, registry};

/// An id of one of the kinds the field allows; on the board when the context lists the ids.
pub(super) fn id(
    field: &str,
    spec: &Field,
    text: &str,
    cx: &Context<'_>,
) -> Result<Value, Rejection> {
    let text = js::trim(text);
    let kinds = &registry().content.reference.kinds;
    let shaped = spec
        .to
        .iter()
        .any(|k| kinds.get(k).is_some_and(|p| p.0.is_match(text)));
    if !shaped {
        return Err(reject(
            "bad-ref",
            format!(
                "{field} is an id of {}, not \"{text}\".",
                spec.to.join(" or ")
            ),
        ));
    }
    if cx.ids.is_some_and(|ids| !ids.iter().any(|i| i == text)) {
        return Err(reject(
            "unknown-ref",
            format!("{text} is not on this board."),
        ));
    }
    Ok(Value::String(text.to_owned()))
}

/// The owner, a thread (of the board when the context lists them), or `lead:<key>` of a known
/// project when the field allows leads.
pub(super) fn person(
    field: &str,
    spec: &Field,
    text: &str,
    cx: &Context<'_>,
) -> Result<Value, Rejection> {
    let text = js::trim(text);
    let reg = registry();
    if text == "owner" {
        return Ok(Value::String(text.to_owned()));
    }
    if let Some(key) = text.strip_prefix("lead:") {
        if spec.leads && reg.knows_lead(key) {
            return Ok(Value::String(text.to_owned()));
        }
        return Err(reject(
            "bad-person",
            format!("{field} cannot be {text}: a lead of another board is only for to-dos."),
        ));
    }
    if !reg.content.person.pattern.0.is_match(text) {
        return Err(reject(
            "bad-person",
            format!("{field} is the owner or a thread id like coordinator, not \"{text}\"."),
        ));
    }
    if cx.threads.is_some_and(|all| !all.iter().any(|t| t == text)) {
        return Err(reject(
            "unknown-person",
            format!("{text} is not a thread of this board."),
        ));
    }
    Ok(Value::String(text.to_owned()))
}
