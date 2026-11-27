//! The Claude accounts service: the accounts config, the discovery of the accounts on this PC
//! and their docs in `_portfolio`. The usage poller, the sync and the switch arrive in later
//! pieces and hang their work on [`AccountsService`].

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex, OnceLock, PoisonError, RwLock};
use std::time::Duration;

use argus_accounts::desktop::ClaudeDesktop;
use argus_accounts::{
    Account, AccountsConfig, AuthProbe, BurnHistory, ClaudeAuthProbe, CliCredentials, Credentials,
    DesktopApp, HttpUsage, Poll, PollResult, ReadOnlyCredentials, Schedule, Usage, UsageSource, discover,
    find_app_data, find_cli_exe, poll_all, refresh_identities,
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
/// How long a login's expiry is kept before the CLI is asked again.
const LOGIN_EVERY_MS: i64 = 3_600_000;

/// A login expiry as the CLI prints it: an ISO text, or epoch milliseconds.
fn login_iso(value: &Value) -> Option<String> {
    match value {
        Value::String(text) if !text.is_empty() => Some(text.clone()),
        Value::Number(n) => n.as_i64().map(argus_core::clock::iso_from_ms),
        _ => None,
    }
}

/// How long Claude may rewrite its storage after a switch before the sidebar check reads it.
pub(crate) const SIDEBAR_SETTLE: Duration = Duration::from_secs(20);

/// Makes the desktop app that lives in an app data folder.
pub type DesktopFactory = Arc<dyn Fn(&Path) -> Arc<dyn DesktopApp> + Send + Sync>;

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
    /// Another desktop app than Claude Desktop, made per app data folder.
    pub desktop: Option<DesktopFactory>,
    /// Another wait before the sidebar check than [`SIDEBAR_SETTLE`].
    pub sidebar_settle: Option<Duration>,
    /// Another home folder of the user than the real one, for `.claude` (transcripts and the
    /// scheduled tasks).
    pub user_home: Option<PathBuf>,
}

impl Default for AccountsOptions {
    fn default() -> Self {
        AccountsOptions {
            enabled: true,
            timer: true,
            probe: None,
            usage: None,
            credentials: None,
            desktop: None,
            sidebar_settle: None,
            user_home: None,
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
    schedule: Mutex<Schedule>,
    probe_login: bool,
    logins: Mutex<HashMap<String, (i64, Option<String>)>>,
    latest: RwLock<HashMap<String, PollResult>>,
    pub(crate) log: Arc<Logger>,
    polled: watch::Sender<Arc<Vec<PollResult>>>,
    pub(crate) handovers: OnceLock<Arc<super::accounts_handover::Handovers>>,
    pub(crate) sync: super::accounts_sync::SyncState,
    pub(crate) rules: OnceLock<Arc<super::accounts_rules::RulesDesk>>,
    pub(crate) switch_log: super::accounts_switch_log::SwitchLog,
    pub(crate) refusals: super::accounts_limit::Refusals,
    desktop: DesktopFactory,
    sidebar_settle: Duration,
    user_home: Option<PathBuf>,
}

/// The outside world of the usage poll.
pub struct UsagePorts {
    /// The usage endpoint.
    pub source: Arc<dyn UsageSource>,
    /// The CLI logins.
    pub creds: Arc<dyn Credentials>,
    /// Where the poll reports what went wrong.
    pub log: Arc<Logger>,
    /// Whether the CLI may be asked when each login expires (never in a plain shadow run).
    pub probe_login: bool,
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
            schedule: Mutex::default(),
            probe_login: ports.probe_login,
            logins: Mutex::default(),
            latest: RwLock::default(),
            polled: watch::channel(Arc::new(Vec::new())).0,
            handovers: OnceLock::new(),
            sync: super::accounts_sync::SyncState::default(),
            rules: OnceLock::new(),
            switch_log: super::accounts_switch_log::SwitchLog::default(),
            refusals: super::accounts_limit::Refusals::default(),
            desktop: Arc::new(|app: &Path| -> Arc<dyn DesktopApp> {
                Arc::new(ClaudeDesktop::new(app))
            }),
            sidebar_settle: SIDEBAR_SETTLE,
            user_home: None,
        }
    }

    /// The same service with the seams of `options` that name another desktop or wait.
    #[must_use]
    pub(crate) fn with_seams(mut self, options: &AccountsOptions) -> Self {
        if let Some(desktop) = &options.desktop {
            self.desktop = Arc::clone(desktop);
        }
        self.sidebar_settle = options.sidebar_settle.unwrap_or(self.sidebar_settle);
        self.user_home.clone_from(&options.user_home);
        self
    }

    /// The desktop app of the app data folder `app`.
    pub(crate) fn desktop(&self, app: &Path) -> Arc<dyn DesktopApp> {
        (self.desktop)(app)
    }

    /// The `.claude` folder of the user: the seam's home, else the real profile.
    pub(crate) fn user_claude(&self) -> PathBuf {
        let home = self.user_home.clone().unwrap_or_else(|| {
            let real = std::env::var("USERPROFILE")
                .or_else(|_| std::env::var("HOME"))
                .unwrap_or_default();
            PathBuf::from(real)
        });
        home.join(".claude")
    }

    /// How long the sidebar check waits after a switch.
    pub(crate) fn sidebar_settle(&self) -> Duration {
        self.sidebar_settle
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
        let active = app.as_deref().and_then(last_known_account);
        let poll = Poll {
            source: &*self.source,
            creds: &*self.creds,
            app: app.as_deref(),
            config: &config,
            active: active.as_deref(),
        };
        let results = {
            let mut burn = self.burn.lock().unwrap_or_else(PoisonError::into_inner);
            let mut schedule = self.schedule.lock().unwrap_or_else(PoisonError::into_inner);
            poll_all(
                &config.accounts,
                &poll,
                (&mut burn, &mut schedule),
                now_ms(),
            )
        };
        let results = self.with_login_expiry(results, app.as_deref(), &config.accounts);
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

    /// How long the poll loop sleeps: until the first account is due again.
    fn next_poll_in(&self) -> std::time::Duration {
        self.schedule
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .wait(now_ms())
    }

    /// `results` with each login's expiry from `claude auth status`, asked at most once an hour.
    fn with_login_expiry(
        &self,
        mut results: Vec<PollResult>,
        app: Option<&Path>,
        accounts: &[Account],
    ) -> Vec<PollResult> {
        let exe = app.and_then(find_cli_exe).filter(|_| self.probe_login);
        let now = now_ms();
        let mut cache = self.logins.lock().unwrap_or_else(PoisonError::into_inner);
        for result in &mut results {
            let known = cache.get(&result.key).cloned();
            let fresh = known
                .as_ref()
                .is_some_and(|(at, _)| now - at < LOGIN_EVERY_MS);
            let expiry = match (&exe, accounts.iter().find(|a| a.key == result.key)) {
                (Some(exe), Some(account)) if !fresh => {
                    let status = self.probe.status(exe, Path::new(&account.cli_dir));
                    let value = status
                        .and_then(|s| s.login_expires_at)
                        .and_then(|v| login_iso(&v))
                        .or_else(|| known.and_then(|(_, v)| v));
                    cache.insert(result.key.clone(), (now, value.clone()));
                    value
                }
                _ => known.and_then(|(_, v)| v),
            };
            result.login_expires_at = expiry;
        }
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

/// The environment variable that lets a preview (never a shadow run) read live usage.
pub(crate) const PREVIEW_USAGE_ENV: &str = "ARGUS_PREVIEW_ACCOUNTS_USAGE";

/// Whether the usage endpoint may be asked: always outside shadow and preview; in a preview
/// only with the opt-in set to `1`; never in a plain shadow run.
pub(crate) fn live_usage_allowed(shadow: bool, preview: bool, opt_in: Option<&str>) -> bool {
    !shadow || (preview && opt_in == Some("1"))
}

/// A usage source that answers nothing, so the poll falls back to the Desktop log.
struct SilentUsage(Arc<dyn UsageSource>, bool);

impl UsageSource for SilentUsage {
    fn fetch(&self, access_token: &str) -> Result<Usage, String> {
        if self.1 {
            self.0.fetch(access_token)
        } else {
            Err("shadow or preview mode: no live usage read".to_owned())
        }
    }
}

/// Logins that are read and never renewed, whatever sits behind them.
struct NoRenewal(Arc<dyn Credentials>);

impl Credentials for NoRenewal {
    fn access_token(&self, dir: &Path) -> Result<String, String> {
        ReadOnlyCredentials.access_token(dir)
    }
    fn expires_at(&self, dir: &Path) -> Option<i64> {
        self.0.expires_at(dir)
    }
    fn refresh(&self, dir: &Path) -> Result<String, String> {
        ReadOnlyCredentials.refresh(dir)
    }
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
    let source: Arc<dyn UsageSource> = opts
        .accounts
        .usage
        .clone()
        .unwrap_or_else(|| Arc::new(HttpUsage));
    let opt_in = std::env::var(PREVIEW_USAGE_ENV).ok();
    let live = live_usage_allowed(svc.shadow, svc.preview, opt_in.as_deref());
    let creds: Arc<dyn Credentials> = if svc.shadow {
        Arc::new(NoRenewal(
            opts.accounts
                .credentials
                .clone()
                .unwrap_or_else(|| Arc::new(ReadOnlyCredentials)),
        ))
    } else {
        opts.accounts.credentials.clone().unwrap_or_else(|| {
            let app = find_app_data(config.app_data.as_deref());
            Arc::new(CliCredentials::new(app))
        })
    };
    let ports = UsagePorts {
        source: if svc.shadow {
            Arc::new(SilentUsage(source, live))
        } else {
            source
        },
        creds,
        log: Arc::clone(&svc.log),
        probe_login: live,
    };
    (opts.accounts.enabled && config.enabled).then(|| {
        let service = AccountsService::new(config, probe, docs, ports);
        Arc::new(service.with_seams(&opts.accounts))
    })
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
    let mut changed = false;
    if config.accounts.is_empty() {
        config = accounts.discover();
        changed = !config.accounts.is_empty();
    } else if refresh_identities(&mut config.accounts) {
        // Emails and plans come from each CLI login; old folder-name labels become the email.
        accounts.set_config(config.clone());
        changed = true;
    }
    {
        if changed && !svc.shadow {
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
    accounts.scan_drift_now();
    if !svc.shadow {
        super::accounts_switch::repair_interrupted(svc, accounts);
    }
}

pub(crate) fn start(svc: &Arc<Services>) {
    let Some(accounts) = svc.accounts.clone() else {
        return;
    };
    super::accounts_handover::attach(svc, &accounts);
    super::accounts_sync::attach(svc, &accounts);
    super::accounts_sync::start(svc, &accounts);
    super::accounts_rules::start(svc, &accounts);
    if !svc.shadow {
        super::accounts_resume::start(svc);
    }
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
        if accounts.poll_now().await.is_err() {
            return;
        }
        tokio::time::sleep(accounts.next_poll_in()).await;
    }
}

pub(crate) fn stop(svc: &Services) {
    svc.tasks.abort(NAME);
}

// test-tag: critical
#[cfg(test)]
mod tests {
    use super::live_usage_allowed;

    #[test]
    fn live_usage_needs_no_shadow_or_a_preview_opt_in() {
        assert!(live_usage_allowed(false, false, None));
        assert!(!live_usage_allowed(true, false, Some("1")));
        assert!(!live_usage_allowed(true, true, None));
        assert!(!live_usage_allowed(true, true, Some("0")));
        assert!(live_usage_allowed(true, true, Some("1")));
    }
}
