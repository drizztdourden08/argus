//! What the tray menu needs of the accounts, in one small read:
//!
//! - `GET /api/accounts/tray` is `{accounts: [{key, label, weekly, active}], switching}`: one
//!   entry per account with its weekly percent (`null` before a usage read) and whether Desktop
//!   runs as it, and whether a switch countdown is running now

use argus_core::ids::PORTFOLIO;
use argus_core::{ApiError, Coll, ErrorCode};
use serde_json::{Value, json};

use crate::reply::json_reply;
use crate::router::Reply;
use crate::{Ctx, Router};

const SWITCH_DOC: &str = "switch";
const COUNTDOWN: &str = "countdown";
const WEEKLY: &str = "weekly_all";

fn weekly(doc: &Value) -> Value {
    doc["limits"]
        .as_array()
        .into_iter()
        .flatten()
        .find(|limit| limit["kind"] == WEEKLY)
        .map_or(Value::Null, |limit| limit["percent"].clone())
}

/// The tray's view of the `Accounts` docs and the `switch` state doc.
fn view(accounts: &[Value], switch: Option<&Value>) -> Value {
    let lines: Vec<Value> = accounts
        .iter()
        .filter_map(|doc| {
            let key = doc["key"].as_str()?;
            Some(json!({
                "key": key,
                "label": doc["label"].as_str().unwrap_or(key),
                "weekly": weekly(doc),
                "active": doc["active"].as_bool().unwrap_or(false),
            }))
        })
        .collect();
    let switching = switch.is_some_and(|doc| doc["phase"] == COUNTDOWN);
    json!({ "accounts": lines, "switching": switching })
}

async fn tray(ctx: Ctx) -> Reply {
    let internal = |e: argus_store::StoreError| ApiError::new(ErrorCode::Internal, e.to_string());
    let docs = ctx.svc.db.docs();
    let accounts: Vec<Value> = docs
        .list(PORTFOLIO, Coll::Accounts)
        .map_err(internal)?
        .into_iter()
        .map(|row| row.data)
        .collect();
    let switch = docs
        .get(PORTFOLIO, Coll::AccountsState, SWITCH_DOC)
        .map_err(internal)?
        .map(|row| row.data);
    Ok(json_reply(200, &view(&accounts, switch.as_ref())))
}

/// Registers the tray read of the accounts.
pub fn register(r: &mut Router) {
    r.get("/api/accounts/tray", tray);
}

// test-tag: critical
#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::view;

    #[test]
    fn one_line_per_account_with_the_weekly_percent() {
        let docs = [
            json!({"key": "a", "label": "Alpha", "active": true,
                   "limits": [{"kind": "five_hour", "percent": 5}, {"kind": "weekly_all", "percent": 42}]}),
            json!({"key": "b", "label": "Beta", "active": false, "limits": []}),
        ];
        let out = view(&docs, None);
        assert_eq!(out["accounts"][0]["weekly"], 42);
        assert_eq!(out["accounts"][0]["active"], true);
        assert_eq!(out["accounts"][1]["weekly"], json!(null));
        assert_eq!(out["switching"], false);
    }

    #[test]
    fn a_countdown_phase_means_switching() {
        let out = view(&[], Some(&json!({"phase": "countdown"})));
        assert_eq!(out["switching"], true);
        assert_eq!(
            view(&[], Some(&json!({"phase": "done"})))["switching"],
            false
        );
    }
}
