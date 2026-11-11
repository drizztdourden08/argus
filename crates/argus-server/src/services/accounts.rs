//! The Claude accounts service: the accounts config, the discovery of the accounts on this PC
//! and their docs in `_portfolio`. The usage poller, the sync and the switch arrive in later
//! pieces and hang their work on [`AccountsService`].

use std::path::Path;
use std::sync::{Arc, PoisonError, RwLock};

use argus_accounts::{Account, AccountsConfig, AuthProbe, ClaudeAuthProbe, discover, find_app_data};
use argus_core::ConfigError;
use argus_core::config::save_config;
use serde_json::json;

use crate::adapters::accounts::AccountsDocs;
use crate::{ServeOptions, Services};

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
}

impl Default for AccountsOptions {
    fn default() -> Self {
        AccountsOptions {
            enabled: true,
            timer: true,
            probe: None,
        }
    }
}

/// The accounts and their settings.
pub struct AccountsService {
    config: RwLock<AccountsConfig>,
    probe: Arc<dyn AuthProbe>,
    docs: AccountsDocs,
}

impl AccountsService {
    /// A service over a config, a probe and the docs.
    #[must_use]
    pub fn new(config: AccountsConfig, probe: Arc<dyn AuthProbe>, docs: AccountsDocs) -> Self {
        AccountsService {
            config: RwLock::new(config),
            probe,
            docs,
        }
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
    (opts.accounts.enabled && config.enabled)
        .then(|| Arc::new(AccountsService::new(config, probe, docs)))
}

fn load(svc: &Services, accounts: &AccountsService) {
    let mut config = accounts.config();
    if config.accounts.is_empty() {
        config = accounts.discover();
        if !config.accounts.is_empty() && !svc.shadow {
            if let Err(e) = save(svc, &config) {
                svc.log
                    .error("accounts config not saved", json!({ "error": e.to_string() }));
            }
        }
    }
    if let Err(e) = accounts.publish(&config.accounts) {
        svc.log
            .error("accounts docs not written", json!({ "error": e }));
    }
}

pub(crate) fn start(svc: &Arc<Services>) {
    let Some(accounts) = svc.accounts.clone() else {
        return;
    };
    let owned = Arc::clone(svc);
    let task = tokio::task::spawn_blocking(move || load(&owned, &accounts));
    svc.tasks.keep(NAME, task);
}

pub(crate) fn stop(svc: &Services) {
    svc.tasks.abort(NAME);
}
