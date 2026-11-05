//! The OS switch and the five automatic pings: `{os, auto: {handed, ask, assigned, verify, run}}`.

use argus_core::{ApiError, ErrorCode};
use argus_notify::AutoPatch;
use serde_json::{Map, Value, to_value};

use super::{saved_reply, service};
use crate::reply::json_reply;
use crate::router::Reply;
use crate::routes::blocking::blocking;
use crate::{AuthKind, Ctx};

const SWITCHES: [&str; 5] = ["handed", "ask", "assigned", "verify", "run"];

fn flag(map: &Map<String, Value>, name: &str) -> Result<Option<bool>, ApiError> {
    match map.get(name) {
        None => Ok(None),
        Some(Value::Bool(on)) => Ok(Some(*on)),
        Some(_) => Err(ApiError::bad_request(format!("{name} is true or false"))),
    }
}

fn refuse_unknown(map: &Map<String, Value>, known: &[&str], place: &str) -> Result<(), ApiError> {
    match map.keys().find(|key| !known.contains(&key.as_str())) {
        Some(key) => Err(ApiError::bad_request(format!("{place}: unknown key {key}"))),
        None => Ok(()),
    }
}

fn object<'a>(value: &'a Value, name: &str) -> Result<&'a Map<String, Value>, ApiError> {
    value
        .as_object()
        .ok_or_else(|| ApiError::bad_request(format!("{name} is an object")))
}

fn patch_of(body: &Value) -> Result<AutoPatch, ApiError> {
    let top = object(body, "the body")?;
    refuse_unknown(top, &["os", "auto"], "the body")?;
    let empty = Map::new();
    let auto = match top.get("auto") {
        Some(value) => object(value, "auto")?,
        None => &empty,
    };
    refuse_unknown(auto, &SWITCHES, "auto")?;
    Ok(AutoPatch {
        os: flag(top, "os")?,
        handed: flag(auto, "handed")?,
        ask: flag(auto, "ask")?,
        assigned: flag(auto, "assigned")?,
        verify: flag(auto, "verify")?,
        run: flag(auto, "run")?,
    })
}

/// `GET /api/notify/auto`.
pub async fn read(ctx: Ctx) -> Reply {
    let settings = service(&ctx)?.auto_settings();
    Ok(json_reply(200, &to_value(settings).unwrap_or(Value::Null)))
}

/// `POST /api/notify/auto`.
pub async fn change(ctx: Ctx) -> Reply {
    let notify = service(&ctx)?;
    if ctx.auth != Some(AuthKind::Page) {
        return Err(ApiError::new(
            ErrorCode::Forbidden,
            "only the owner's page changes the automatic notifications",
        ));
    }
    let patch = patch_of(&ctx.json().await?)?;
    let saved = blocking(move || notify.set_auto(patch)).await?;
    saved_reply(saved)
}
