//! The threads that hold a queued wake and have no listener, with their Desktop session id.

use std::collections::HashMap;

use argus_accounts::find_app_data;
use argus_accounts::session_map::local_ids;
use argus_wake::registry::DueThread;
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

fn entry(due: &DueThread, local: &HashMap<String, String>) -> Value {
    let mut value = to_value(due).unwrap_or_default();
    let id = due.session_id.as_ref().and_then(|s| local.get(s));
    value["localId"] = json!(id);
    value
}

pub(super) async fn due(ctx: Ctx) -> Reply {
    let wake = service(&ctx)?;
    let due = blocking(move || lock(&wake).due()).await?;
    let local = mapping(&ctx, &due);
    let rows: Vec<Value> = due.iter().map(|d| entry(d, &local)).collect();
    Ok(json_reply(200, &json!({ "due": rows })))
}
