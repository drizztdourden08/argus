//! Board presence on the mux: a tab of the browser says which thread has its board open, and the
//! connection keeps the claim until the tab says otherwise or the stream ends. It is the same
//! claim the legacy presence stream holds, so the open-boards list and the `board-closed` grace
//! behave the same.

use std::collections::HashMap;

use argus_core::ApiError;
use argus_core::ids::is_key;
use argus_wake::external::{clean_tab, presence_thread};
use argus_wake::presence::{MAX_TABS_PER_THREAD, PresenceToken};
use argus_wake::service::WakeService;
use argus_wake::service::driver::{SharedWake, lock};
use serde_json::Value;

use super::parse::OpError;

/// A checked `presence` op.
pub struct Claim {
    scope: String,
    thread: String,
    tab: String,
}

/// What one tab holds.
pub struct Held {
    thread: String,
    token: PresenceToken,
}

/// The claims of a connection, by project and tab id.
pub type Tokens = HashMap<(String, String), Held>;

fn scope_of(raw: &Value) -> Result<String, OpError> {
    match raw["scope"].as_str().filter(|s| is_key(s)) {
        Some(s) => Ok(s.to_owned()),
        None => Err(ApiError::bad_request("scope must be a project key").into()),
    }
}

fn tab_of(raw: &Value) -> String {
    clean_tab(raw["tab"].as_str().unwrap_or_default())
}

/// Checks a `presence` op.
///
/// # Errors
/// `bad_request` for a bad scope or a thread that is not a thread id.
pub fn claim_of(raw: &Value) -> Result<Claim, OpError> {
    let thread =
        presence_thread(raw["thread"].as_str()).map_err(|e| ApiError::bad_request(e.message()))?;
    Ok(Claim {
        scope: scope_of(raw)?,
        thread,
        tab: tab_of(raw),
    })
}

/// Checks an `unpresence` op: the project and the tab id.
///
/// # Errors
/// `bad_request` for a bad scope.
pub fn release_of(raw: &Value) -> Result<(String, String), OpError> {
    Ok((scope_of(raw)?, tab_of(raw)))
}

fn not_running() -> OpError {
    ApiError::unavailable("the wake service is not running").into()
}

fn tabs_open(service: &WakeService, claim: &Claim) -> usize {
    let rows = service.presence_list();
    let row = rows
        .iter()
        .find(|r| r.project == claim.scope && r.thread == claim.thread);
    row.map_or(0, |r| r.tabs)
}

/// Registers the tab. The same tab id again replaces its claim, on the same thread or another.
///
/// # Errors
/// `unavailable` without a wake service, or when the thread already has the most tabs open.
pub fn attach(wake: Option<&SharedWake>, tokens: &mut Tokens, claim: Claim) -> Result<(), OpError> {
    let wake = wake.ok_or_else(not_running)?;
    let slot = (claim.scope.clone(), claim.tab.clone());
    let renewal = tokens.get(&slot).is_some_and(|h| h.thread == claim.thread);
    let mut service = lock(wake);
    if !renewal && tabs_open(&service, &claim) >= MAX_TABS_PER_THREAD {
        let message = format!(
            "{} already has {MAX_TABS_PER_THREAD} tabs open on {}",
            claim.thread, claim.scope
        );
        return Err(ApiError::unavailable(message).into());
    }
    let token = service.presence_attach(&claim.scope, &claim.thread, &claim.tab);
    let held = Held {
        thread: claim.thread,
        token,
    };
    if let Some(old) = tokens.insert(slot, held) {
        service.presence_detach(&old.token);
    }
    Ok(())
}

/// Gives the tab's claim back. A tab that holds none is not an error.
pub fn detach(wake: Option<&SharedWake>, tokens: &mut Tokens, scope: &str, tab: &str) {
    let held = tokens.remove(&(scope.to_owned(), tab.to_owned()));
    if let (Some(held), Some(wake)) = (held, wake) {
        lock(wake).presence_detach(&held.token);
    }
}

/// Gives every claim back (the stream ended).
pub fn release_all(wake: Option<&SharedWake>, tokens: &mut Tokens) {
    let Some(wake) = wake else {
        return;
    };
    let mut service = lock(wake);
    for (_, held) in tokens.drain() {
        service.presence_detach(&held.token);
    }
}
