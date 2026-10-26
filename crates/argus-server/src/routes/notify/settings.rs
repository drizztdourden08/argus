//! The per-level settings: toast on or off and sound on or off for `info`, `ping` and `urgent`.

use argus_core::{ApiError, ErrorCode};
use argus_notify::{Level, PerLevel, SettingsPatch};
use serde_json::{Value, to_value};

use super::{LEVEL_WORDS, service};
use crate::reply::json_reply;
use crate::router::Reply;
use crate::routes::blocking::blocking;
use crate::{AuthKind, Ctx};

/// A part of the body: absent is nothing to change, else an object of booleans per level.
fn part_of(body: &Value, name: &str) -> Result<PerLevel<Option<bool>>, ApiError> {
    let given = &body[name];
    if body.get(name).is_none() {
        return Ok(PerLevel::default());
    }
    let Some(levels) = given.as_object() else {
        return Err(ApiError::bad_request(format!(
            "{name} is an object of true or false per level"
        )));
    };
    let mut out = PerLevel::default();
    for (level, on) in levels {
        let known = Level::parse(level).ok_or_else(|| {
            ApiError::bad_request(format!("{name}.{level}: the levels are {LEVEL_WORDS}"))
        })?;
        let on = on
            .as_bool()
            .ok_or_else(|| ApiError::bad_request(format!("{name}.{level} is true or false")))?;
        set(&mut out, known, on);
    }
    Ok(out)
}

fn set(values: &mut PerLevel<Option<bool>>, level: Level, on: bool) {
    match level {
        Level::Info => values.info = Some(on),
        Level::Ping => values.ping = Some(on),
        Level::Urgent => values.urgent = Some(on),
    }
}

/// `GET /api/notify/settings`.
pub async fn read(ctx: Ctx) -> Reply {
    let settings = service(&ctx)?.settings();
    Ok(json_reply(200, &to_value(settings).unwrap_or(Value::Null)))
}

/// `POST /api/notify/settings`.
pub async fn change(ctx: Ctx) -> Reply {
    let notify = service(&ctx)?;
    if ctx.auth != Some(AuthKind::Page) {
        return Err(ApiError::new(
            ErrorCode::Forbidden,
            "only the owner's page changes the notification settings",
        ));
    }
    let body = ctx.json().await?;
    let patch = SettingsPatch {
        toast: part_of(&body, "toast")?,
        sound: part_of(&body, "sound")?,
    };
    let saved = blocking(move || notify.configure(patch)).await?;
    let settings = saved.map_err(|e| {
        let why: String = e.to_string().chars().take(120).collect();
        ApiError::new(
            ErrorCode::Internal,
            format!("the settings could not be saved: {why}"),
        )
    })?;
    Ok(json_reply(200, &to_value(settings).unwrap_or(Value::Null)))
}
