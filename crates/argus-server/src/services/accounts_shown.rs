//! What `GET /api/accounts` and `POST /api/accounts/next` show beyond the accounts themselves:
//! each account's `limit`, the `next` switch and the `run` of the current switch.

use argus_core::ApiError;
use serde_json::{Value, json};

use super::AccountsService;
use super::accounts_next::{clear_plan, next_shown, set_plan};
use super::accounts_run::run_shown;
use super::accounts_switch::shown_switch;
use crate::Services;

/// The parts of the answer of `GET /api/accounts`.
pub(crate) struct Shown {
    /// The accounts, each with its `limit`.
    pub accounts: Value,
    /// The next switch.
    pub next: Value,
    /// The current switch run.
    pub run: Value,
}

fn with_limit(accounts: &AccountsService) -> Value {
    let list: Vec<Value> = accounts
        .accounts()
        .iter()
        .map(|account| {
            let mut out = json!(account);
            out["limit"] = accounts.limit_shown(&account.key);
            out
        })
        .collect();
    json!(list)
}

/// The accounts with their limits, the next switch and the run. Blocking.
pub(crate) fn shown_of(svc: &Services, accounts: &AccountsService) -> Shown {
    let doc = shown_switch(svc);
    let leaving = doc
        .as_ref()
        .and_then(|d| d["from"].as_str())
        .is_some_and(|key| accounts.limit_confirmed(key));
    Shown {
        accounts: with_limit(accounts),
        next: json!(next_shown(svc, accounts)),
        run: run_shown(doc.as_ref(), leaving, argus_core::clock::now_ms()),
    }
}

/// Sets the plan from the body of `POST /api/accounts/next`, or clears it, and answers the next
/// switch now. An end that is null, empty or left out is left to Argus. Blocking.
///
/// # Errors
/// 400 with a plain sentence when the plan is refused.
pub(crate) fn plan_of(
    svc: &Services,
    accounts: &AccountsService,
    body: &Value,
) -> Result<Value, ApiError> {
    let key = |name: &str| {
        let sent = body.get(name).and_then(Value::as_str);
        sent.filter(|k| !k.is_empty())
    };
    if body.get("clear").and_then(Value::as_bool) == Some(true) {
        clear_plan(svc, accounts).map_err(ApiError::bad_request)?;
    } else {
        let ends = (key("from"), key("to"));
        if ends == (None, None) {
            return Err(ApiError::bad_request(
                "Send `from` or `to`, or `clear` true.",
            ));
        }
        set_plan(svc, accounts, ends).map_err(ApiError::bad_request)?;
    }
    Ok(json!(next_shown(svc, accounts)))
}
