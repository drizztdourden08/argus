//! The next switch of Desktop: the accounts as the rules see them, the plan the owner sets in
//! the settings, and the `next` of `GET /api/accounts`. The pure parts are `argus_accounts::next`.

use argus_accounts::desktop::FsVault;
use argus_accounts::next::{Next, NextPlan, check_plan, next_of};
use argus_accounts::rules::View;
use argus_accounts::{
    Account, AccountsConfig, LoginVault, PollResult, Severity, binding_limit, find_app_data,
};
use argus_core::clock::{now_ms, parse_ms};
use serde_json::json;

use super::AccountsService;
use super::accounts::{last_known_account, save};
use super::accounts_switch::logins_root;
use crate::Services;

fn view_of(account: &Account, polled: Option<&PollResult>, has_login: bool) -> View {
    let limit = polled.and_then(|p| binding_limit(&p.usage));
    View {
        key: account.key.clone(),
        label: account.label.clone(),
        // A rule may only switch to an account whose Desktop login is saved.
        can_receive: account.can_receive && has_login,
        rules: account.rules.clone(),
        percent: limit.map(|l| l.percent),
        critical: limit.is_some_and(|l| l.severity == Severity::Critical),
        resets_at_ms: limit
            .and_then(|l| l.resets_at.as_deref())
            .and_then(parse_ms),
    }
}

/// Every account as the rules see it. Blocking.
pub(crate) fn views(svc: &Services, accounts: &AccountsService) -> Vec<View> {
    let config = accounts.config();
    let latest = accounts.latest_usage();
    let app = find_app_data(config.app_data.as_deref()).unwrap_or_default();
    let vault = FsVault::new(&app, &logins_root(svc));
    config
        .accounts
        .iter()
        .map(|a| view_of(a, latest.get(&a.key), vault.saved_at(a).is_some()))
        .collect()
}

/// The key of the account Desktop runs as.
pub(crate) fn active_key(config: &AccountsConfig) -> Option<String> {
    let app = find_app_data(config.app_data.as_deref())?;
    let uuid = last_known_account(&app)?;
    config
        .accounts
        .iter()
        .find(|a| a.desktop_account.as_deref() == Some(uuid.as_str()))
        .map(|a| a.key.clone())
}

/// Keeps `plan` in the settings, in memory and on disk. A shadow run keeps it in memory only.
fn keep(svc: &Services, accounts: &AccountsService, plan: Option<NextPlan>) -> Result<(), String> {
    let mut config = accounts.config();
    config.next_plan = plan;
    accounts.set_config(config.clone());
    if svc.shadow {
        return Ok(());
    }
    save(svc, &config).map_err(|e| e.to_string())
}

/// Sets the owner's plan: both ends, or one end with the other left to Argus. Blocking.
///
/// # Errors
/// A plain sentence when the plan is refused or cannot be saved.
pub(crate) fn set_plan(
    svc: &Services,
    accounts: &AccountsService,
    (from, to): (Option<&str>, Option<&str>),
) -> Result<NextPlan, String> {
    let plan = check_plan(&accounts.accounts(), &views(svc, accounts), from, to)?;
    keep(svc, accounts, Some(plan.clone()))?;
    Ok(plan)
}

/// Clears the owner's plan. Blocking.
///
/// # Errors
/// When the settings cannot be saved.
pub(crate) fn clear_plan(svc: &Services, accounts: &AccountsService) -> Result<(), String> {
    keep(svc, accounts, None)
}

/// Drops a plan Desktop already carried out: it runs as the account the plan goes to. Blocking.
pub(crate) fn settle_plan(svc: &Services, accounts: &AccountsService, active: Option<&str>) {
    let done = accounts
        .config()
        .next_plan
        .is_some_and(|p| p.to.is_some() && p.to.as_deref() == active);
    if done {
        if let Err(e) = keep(svc, accounts, None) {
            svc.log
                .warn("accounts plan not cleared", json!({ "error": e }));
        }
    }
}

/// The `next` of `GET /api/accounts`. Blocking.
pub(crate) fn next_shown(svc: &Services, accounts: &AccountsService) -> Next {
    let config = accounts.config();
    let active = active_key(&config);
    let latest = accounts.latest_usage();
    let burn = |key: &str| latest.get(key).and_then(|p| p.burn_per_min);
    next_of(
        &views(svc, accounts),
        active.as_deref(),
        config.next_plan.as_ref(),
        &burn,
        now_ms(),
    )
}
