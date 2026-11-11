//! The Claude accounts service: the accounts config, the discovery of the accounts on this PC
//! and their docs in `_portfolio`. The usage poller, the sync and the switch arrive in later
//! pieces and hang their work on [`AccountsService`].

use std::collections::HashMap;
use std::path::Path;
use std::sync::{Arc, Mutex, OnceLock, PoisonError, RwLock};

use argus_accounts::{
    Account, AccountsConfig, AuthProbe, BurnHistory, ClaudeAuthProbe, CliCredentials, Credentials,
    HttpUsage, Poll, PollResult, UsageSource, discover, find_app_data, next_interval, poll_all,
};
use argus_core::ConfigError;
use argus_core::clock::now_ms;
use argus_core::config::save_config;
use argus_core::log::Logger;
use serde_json::{Value, json};
use tokio::sync::watch;

use crate::adapters::accounts::AccountsDocs;
use crate::services::AccountAlerts;
use crate::services::tasks::Timer;
use crate::{ServeOptions, Services};
use argus_accounts::alerts::Snapshot;

const NAME: &str = "accounts";

/// How the accounts service starts; the fields after `enabled` are seams for tests.
#[derive(Clone)]
pub struct AccountsOptions {
    /// Run the service.
    pub enabled: bool,
    /// Poll usage on a timer.
    pub timer: bool,
    /// Another way to ask the CLI who is logged in than running it.
    pub probe: Option<Arc<dyn AuthProbe>>,
    /// Another usage endpoint.
    pub usage: Option<Arc<dyn UsageSource>>,
    /// Other CLI logins.
    pub credentials: Option<Arc<dyn Credentials>>,
}

impl Default for AccountsOptions {
    fn default() -> Self {
        AccountsOptions {
            enabled: true,
            timer: true,
            probe: None,
            usage: None,
            credentials: None,
        }
    }
}

/// The accounts and their settings.
pub struct AccountsService {
    config: RwLock<AccountsConfig>,
    probe: Arc<dyn AuthProbe>,
    pub(crate) docs: AccountsDocs,
    source: Arc<dyn UsageSource>,
    creds: Arc<dyn Credentials>,
    burn: Mutex<BurnHistory>,
    latest: RwLock<HashMap<String, PollResult>>,
    pub(crate) log: Arc<Logger>,
    polled: watch::Sender<Arc<Vec<PollResult>>>,
    pub(crate) handovers: OnceLock<Arc<super::accounts_handover::Handovers>>,
    pub(crate) sync: super::accounts_sync::SyncState,
    pub(crate) rules: OnceLock<Arc<super::accounts_rules::RulesDesk>>,
}

/// The outside world of the usage poll.
pub struct UsagePorts {
    /// The usage endpoint.
    pub source: Arc<dyn UsageSource>,
    /// The CLI logins.
    pub creds: Arc<dyn Credentials>,
    /// Where the poll reports what went wrong.
    pub log: Arc<Logger>,
}

impl AccountsService {
    /// A service over a config, a probe, the docs and the usage ports.
    #[must_use]
    pub fn new(
        config: AccountsConfig,
        probe: Arc<dyn AuthProbe>,
        docs: AccountsDocs,
        ports: UsagePorts,
    ) -> Self {
        AccountsService {
            config: RwLock::new(config),
            probe,
            docs,
            source: ports.source,
            creds: ports.creds,
            log: ports.log,
            burn: Mutex::default(),
            latest: RwLock::default(),
            polled: watch::channel(Arc::new(Vec::new())).0,
            handovers: OnceLock::new(),
            sync: super::accounts_sync::SyncState::default(),
            rules: OnceLock::new(),
        }
    }

    /// The last poll result of each account, by key.
    #[must_use]
    pub fn latest_usage(&self) -> HashMap<String, PollResult> {
        self.latest
            .read()
            .unwrap_or_else(PoisonError::into_inner)
            .clone()
    }

    /// A receiver that sees the results of every poll pass as it lands, for the alerts and the
    /// rules. It starts with an empty list before the first pass.
    #[must_use]
    pub fn subscribe(&self) -> watch::Receiver<Arc<Vec<PollResult>>> {
        self.polled.subscribe()
    }

    /// Reads every account's usage now, writes the docs and tells the subscribers.
    ///
    /// # Errors
    /// When the blocking task fails.
    pub async fn poll_now(self: &Arc<Self>) -> Result<Vec<PollResult>, String> {
        let me = Arc::clone(self);
        tokio::task::spawn_blocking(move || me.poll_blocking())
            .await
            .map_err(|e| e.to_string())
    }

    fn poll_blocking(&self) -> Vec<PollResult> {
        let config = self.config();
        let app = find_app_data(config.app_data.as_deref());
        let poll = Poll {
            source: &*self.source,
            creds: &*self.creds,
            app: app.as_deref(),
        };
        let results = {
            let mut burn = self.burn.lock().unwrap_or_else(PoisonError::into_inner);
            poll_all(&config.accounts, &poll, &mut burn, now_ms())
        };
        let active = app.as_deref().and_then(last_known_account);
        if let Err(e) = self
            .docs
            .put_usage(&config.accounts, &results, active.as_deref())
        {
            self.log
                .error("accounts usage docs not written", json!({ "error": e }));
        }
        for failed in results.iter().filter(|r| r.failure.is_some()) {
            self.log.warn(
                "accounts usage read failed",
                json!({ "account": failed.key, "error": failed.failure }),
            );
        }
        *self.latest.write().unwrap_or_else(PoisonError::into_inner) =
            results.iter().map(|r| (r.key.clone(), r.clone())).collect();
        self.polled.send_replace(Arc::new(results.clone()));
        results
    }

    /// A copy of the settings.
    #[must_use]
    pub fn config(&self) -> AccountsConfig {
        self.config
            .read()
            .unwrap_or_else(PoisonError::into_inner)
            .clone()
    }

    /// Replaces the settings.
    pub fn set_config(&self, config: AccountsConfig) {
        *self.config.write().unwrap_or_else(PoisonError::into_inner) = config;
    }

    /// The known accounts.
    #[must_use]
    pub fn accounts(&self) -> Vec<Account> {
        self.config().accounts
    }

    /// The settings with the keys of `patch` replaced, kept in memory.
    ///
    /// # Errors
    /// When the patch makes the settings unreadable.
    pub fn update(&self, patch: &serde_json::Value) -> Result<AccountsConfig, String> {
        let next = self.config().patched(patch)?;
        self.set_config(next.clone());
        Ok(next)
    }

    /// Looks for the accounts of this PC and keeps them in the settings.
    fn discover(&self) -> AccountsConfig {
        let mut config = self.config();
        let app = find_app_data(config.app_data.as_deref());
        config.accounts = discover(app.as_deref(), Path::new(&config.cli_root), &*self.probe);
        self.set_config(config.clone());
        config
    }

    /// Writes the docs of the accounts.
    fn publish(&self, accounts: &[Account]) -> Result<(), String> {
        self.docs.put_accounts(accounts)
    }
}

/// Saves the settings into `config.json` under `accounts`.
///
/// # Errors
/// When the file cannot be written.
pub(crate) fn save(svc: &Services, config: &AccountsConfig) -> Result<(), ConfigError> {
    let mut all = svc.config.write().unwrap_or_else(PoisonError::into_inner);
    all.extra.insert(NAME.to_owned(), config.to_json());
    save_config(&svc.paths, &all, svc.env_port)
}

pub(crate) fn build(svc: &Services, opts: &ServeOptions) -> Option<Arc<AccountsService>> {
    let setting = svc.config().extra.get(NAME).cloned().unwrap_or_default();
    let config = AccountsConfig::from_json(&setting);
    let probe = opts
        .accounts
        .probe
        .clone()
        .unwrap_or_else(|| Arc::new(ClaudeAuthProbe));
    let docs = AccountsDocs::new(Arc::clone(&svc.db));
    let ports = UsagePorts {
        source: opts
            .accounts
            .usage
            .clone()
            .unwrap_or_else(|| Arc::new(HttpUsage)),
        creds: opts.accounts.credentials.clone().unwrap_or_else(|| {
            let app = find_app_data(config.app_data.as_deref());
            Arc::new(CliCredentials::new(app))
        }),
        log: Arc::clone(&svc.log),
    };
    (opts.accounts.enabled && config.enabled)
        .then(|| Arc::new(AccountsService::new(config, probe, docs, ports)))
}

/// The Desktop account that is logged in now, from `<app>/config.json`.
pub(crate) fn last_known_account(app: &Path) -> Option<String> {
    let text = std::fs::read_to_string(app.join("config.json")).ok()?;
    let config: Value = serde_json::from_str(&text).ok()?;
    config
        .get("lastKnownAccountUuid")
        .and_then(Value::as_str)
        .map(str::to_owned)
}

fn load(svc: &Services, accounts: &AccountsService) {
    let mut config = accounts.config();
    if config.accounts.is_empty() {
        config = accounts.discover();
        if !config.accounts.is_empty() && !svc.shadow {
            if let Err(e) = save(svc, &config) {
                svc.log.error(
                    "accounts config not saved",
                    json!({ "error": e.to_string() }),
                );
            }
        }
    }
    if let Err(e) = accounts.publish(&config.accounts) {
        svc.log
            .error("accounts docs not written", json!({ "error": e }));
    }
    super::accounts_switch::publish_logins(svc, accounts);
}

pub(crate) fn start(svc: &Arc<Services>) {
    let Some(accounts) = svc.accounts.clone() else {
        return;
    };
    super::accounts_handover::attach(svc, &accounts);
    super::accounts_sync::attach(svc, &accounts);
    super::accounts_sync::start(svc, &accounts);
    super::accounts_rules::start(svc, &accounts);
    super::accounts_resume::start(svc);
    let owned = Arc::clone(svc);
    let polling = svc.tasks.runs(Timer::Accounts);
    let task = tokio::spawn(async move {
        let loader = Arc::clone(&accounts);
        if tokio::task::spawn_blocking(move || load(&owned, &loader))
            .await
            .is_ok()
            && polling
        {
            poll_loop(&accounts).await;
        }
    });
    svc.tasks.keep(NAME, task);
    if polling {
        let alerting = Arc::clone(svc);
        let listener = tokio::spawn(async move { alert_loop(alerting).await });
        svc.tasks.keep(NAME, listener);
    }
}

/// Hands every finished poll to the usage toasts.
async fn alert_loop(svc: Arc<Services>) {
    let Some(accounts) = svc.accounts.clone() else {
        return;
    };
    let alerts = Arc::new(AccountAlerts::default());
    let mut polled = accounts.subscribe();
    while polled.changed().await.is_ok() {
        let results = Arc::clone(&polled.borrow_and_update());
        let (svc, alerts, accounts) =
            (Arc::clone(&svc), Arc::clone(&alerts), Arc::clone(&accounts));
        let _ = tokio::task::spawn_blocking(move || {
            let snapshots = snapshots_of(&accounts, &results);
            alerts.on_poll(&svc, snapshots);
        })
        .await;
    }
}

/// One alert snapshot per known account that the poll covered.
fn snapshots_of(accounts: &AccountsService, results: &[PollResult]) -> Vec<Snapshot> {
    let config = accounts.config();
    let active = find_app_data(config.app_data.as_deref())
        .as_deref()
        .and_then(last_known_account);
    config
        .accounts
        .into_iter()
        .filter_map(|account| {
            let usage = results.iter().find(|r| r.key == account.key)?.usage.clone();
            let is_active = active.is_some() && active == account.desktop_account;
            Some(Snapshot {
                account,
                usage,
                active: is_active,
            })
        })
        .collect()
}

/// Polls on the computed interval until the task is aborted.
async fn poll_loop(accounts: &Arc<AccountsService>) {
    loop {
        let Ok(results) = accounts.poll_now().await else {
            return;
        };
        tokio::time::sleep(next_interval(&accounts.config(), &results)).await;
    }
}

pub(crate) fn stop(svc: &Services) {
    svc.tasks.abort(NAME);
}
