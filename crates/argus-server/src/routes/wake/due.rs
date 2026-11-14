//! The threads that hold a queued wake and have no listener, with their Desktop session id.

use std::collections::HashMap;

use argus_accounts::find_app_data;
use argus_accounts::session_map::local_ids;
use argus_wake::registry::DueThread;
use argus_wake::relay::LastAttempt;
use argus_wake::service::driver::lock;
use serde_json::{Value, json, to_value};

use super::service;
use crate::Ctx;
use crate::reply::json_reply;
use crate::router::Reply;
use crate::routes::blocking::blocking;

/// The Desktop session ids of the CLI session ids, from the Desktop app's records; empty when the
/// app's data folder is not found.
fn mapping(ctx: &Ctx, due: &[DueThread]) -> HashMap<String, String> {
    let wanted: Vec<String> = due.iter().filter_map(|d| d.session_id.clone()).collect();
    let configured = ctx.svc.accounts.as_ref().map(|a| a.config().app_data);
    find_app_data(configured.flatten().as_deref())
        .map(|app| local_ids(&app, &wanted))
        .unwrap_or_default()
}

fn entry(due: &DueThread, local: &HashMap<String, String>, last: Option<&LastAttempt>) -> Value {
    let mut value = to_value(due).unwrap_or_default();
    let id = due.session_id.as_ref().and_then(|s| local.get(s));
    value["localId"] = json!(id);
    value["relay"] = last.map_or(Value::Null, LastAttempt::json);
    value
}

pub(super) async fn due(ctx: Ctx) -> Reply {
    let wake = service(&ctx)?;
    let (due, lasts) = blocking(move || {
        let mut guard = lock(&wake);
        let due = guard.due();
        let lasts: Vec<Option<LastAttempt>> = due.iter().map(|d| guard.relay_last(d)).collect();
        (due, lasts)
    })
    .await?;
    let local = mapping(&ctx, &due);
    let rows: Vec<Value> = due
        .iter()
        .zip(&lasts)
        .map(|(d, last)| entry(d, &local, last.as_ref()))
        .collect();
    Ok(json_reply(200, &json!({ "due": rows })))
}
