//! The owner's settings of the hosted child (I-200): the model, the effort and the permission mode,
//! saved in `config.overlord`. A change restarts a running child on the same session.

use std::sync::Arc;

use argus_overlord::session::choices::{
    BadChoice, Change, EFFORTS, check, models, permission_modes,
};
use serde_json::{Value, json};

use super::config::{self, is_live};
use super::control::{self, Refused};
use super::live_session;
use super::setting::save_fields;
use super::settings_limits::{self, Limits};
use crate::Services;

/// Why a change was not applied.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum SettingsFault {
    /// A field is not one of the choices; nothing was saved.
    Bad(BadChoice),
    /// `config.json` could not be written; nothing was saved.
    Storage,
    /// A folder, program or limit is missing or out of range; nothing was saved.
    Invalid(String),
    /// Saved, but the running child could not be restarted.
    Restart(Refused),
}

/// The settings and their choices: the shape of `GET /api/overlord/settings`.
pub(crate) fn body(svc: &Services) -> Value {
    let live = config::live(svc);
    let mut body = json!({
        "model": live.model,
        "effort": live.effort,
        "permissionMode": live.permission_mode,
        "choices": {
            "models": models(&live.model),
            "efforts": EFFORTS,
            "permissionModes": permission_modes(),
        },
    });
    if let (Some(all), Value::Object(limits)) = (body.as_object_mut(), settings_limits::body(svc))
    {
        all.extend(limits);
    }
    body
}

/// The `config.overlord` fields a change writes.
fn fields(change: &Change) -> Vec<(String, Value)> {
    [
        ("model", &change.model),
        ("effort", &change.effort),
        ("permissionMode", &change.permission_mode),
    ]
    .into_iter()
    .filter_map(|(name, value)| value.as_ref().map(|v| (name.to_owned(), json!(v))))
    .collect()
}

/// Restarts the running child on the session it runs: its id is saved as the live session, and
/// the start that follows resumes the saved session with `--resume <id>` and no fork.
async fn restart_same_session(svc: &Arc<Services>) -> Result<(), Refused> {
    let running = svc.overlord.state().session_id.filter(|_| is_live(svc));
    if let Some(id) = running {
        live_session::save(&svc.db, &id);
    }
    control::restart(svc).await
}

/// Checks `change` and `limits` and saves them. When they changed a setting and a child runs, the
/// child is restarted on the same session so the new setting takes effect.
///
/// # Errors
/// A [`SettingsFault`] naming what failed.
pub(crate) async fn apply(
    svc: &Arc<Services>,
    change: &Change,
    limits: &Limits,
) -> Result<(), SettingsFault> {
    let before = (config::live(svc), config::budget(svc));
    check(change, &before.0.model).map_err(SettingsFault::Bad)?;
    let mut all = fields(change);
    all.extend(limits.fields().map_err(SettingsFault::Invalid)?);
    save_fields(svc, all).map_err(|_| SettingsFault::Storage)?;
    if (config::live(svc), config::budget(svc)) == before || !svc.overlord.up() {
        return Ok(());
    }
    restart_same_session(svc)
        .await
        .map_err(SettingsFault::Restart)
}
