//! Reads a rule input or a patch from a JSON body, with a plain refusal for a wrong type.

use serde_json::Value;

use super::{RuleInput, RulePatch};

fn text(body: &Value, name: &str) -> Result<Option<String>, String> {
    match &body[name] {
        Value::Null => Ok(None),
        Value::String(value) => Ok(Some(value.clone())),
        _ => Err(format!("{name} must be text")),
    }
}

fn minutes(body: &Value) -> Result<Option<i64>, String> {
    match &body["everyMin"] {
        Value::Null => Ok(None),
        other => other
            .as_i64()
            .map(Some)
            .ok_or_else(|| "everyMin must be a whole number".to_owned()),
    }
}

fn flag(body: &Value) -> Result<Option<bool>, String> {
    match &body["enabled"] {
        Value::Null => Ok(None),
        Value::Bool(value) => Ok(Some(*value)),
        _ => Err("enabled must be true or false".to_owned()),
    }
}

/// A patch from a body; every field is optional.
///
/// # Errors
/// A sentence naming the field of the wrong type.
pub fn parse_patch(body: &Value) -> Result<RulePatch, String> {
    Ok(RulePatch {
        title: text(body, "title")?,
        prompt: text(body, "prompt")?,
        every_min: minutes(body)?,
        enabled: flag(body)?,
    })
}

/// A new rule's input from a body: title, prompt and everyMin are needed, enabled defaults to true.
///
/// # Errors
/// A sentence naming the missing field or the field of the wrong type.
pub fn parse_input(body: &Value) -> Result<RuleInput, String> {
    let fields = parse_patch(body)?;
    Ok(RuleInput {
        title: fields.title.ok_or("send the title as text")?,
        prompt: fields.prompt.ok_or("send the prompt as text")?,
        every_min: fields.every_min.ok_or("send everyMin as a whole number")?,
        enabled: fields.enabled.unwrap_or(true),
    })
}
