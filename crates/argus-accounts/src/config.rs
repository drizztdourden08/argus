//! The `accounts` key of `config.json`. Every field has a default, so a missing or half-written
//! key still gives a working config.

use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::types::Account;

/// Where an archived chat is kept when the accounts disagree.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ArchiveRule {
    /// The newest record of the chat decides.
    #[default]
    Newest,
    /// Archived on any account means archived on all.
    Anywhere,
}

/// How much of a limit one agent of each kind is assumed to use per run.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct EstimateDefaults {
    /// An Opus agent.
    pub opus: f64,
    /// A Sonnet agent.
    pub sonnet: f64,
    /// A Haiku agent.
    pub haiku: f64,
    /// A lead thread.
    pub lead: f64,
    /// The factor for a long context.
    pub long_context_factor: f64,
    /// The margin added on top.
    pub safety: f64,
}

impl Default for EstimateDefaults {
    fn default() -> Self {
        EstimateDefaults {
            opus: 0.3,
            sonnet: 0.1,
            haiku: 0.03,
            lead: 0.4,
            long_context_factor: 1.5,
            safety: 0.5,
        }
    }
}

/// The accounts settings.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct AccountsConfig {
    /// Run the service.
    pub enabled: bool,
    /// The Desktop app's data folder; `None` finds it.
    pub app_data: Option<String>,
    /// The folder holding one CLI config folder per account.
    pub cli_root: String,
    /// Minutes between usage reads.
    pub poll_minutes: u64,
    /// Seconds between usage reads once an account is past `fast_at`.
    pub fast_poll_seconds: u64,
    /// The percent from which the fast poll runs.
    pub fast_at: f64,
    /// The percent from which the owner is warned.
    pub warn_at: f64,
    /// The percent from which a switch is proposed.
    pub propose_at: f64,
    /// Which archive state wins in a sync.
    pub archive_rule: ArchiveRule,
    /// Handover estimate defaults.
    pub estimate_defaults: EstimateDefaults,
    /// Calibration factor of the estimates.
    pub k: f64,
    /// The Claude package version a switch by hand last confirmed; automatic switches need the
    /// installed version to be this one.
    pub verified_app_version: Option<String>,
    /// End Claude Desktop by PID when it stays in the tray after a polite close. Off by default:
    /// Desktop may lose its last few seconds of unsaved sidebar changes.
    pub force_quit: bool,
    /// The known accounts; empty until discovery fills it.
    pub accounts: Vec<Account>,
}

impl Default for AccountsConfig {
    fn default() -> Self {
        AccountsConfig {
            enabled: true,
            app_data: None,
            cli_root: default_cli_root(),
            poll_minutes: 5,
            fast_poll_seconds: 60,
            fast_at: 80.0,
            warn_at: 80.0,
            propose_at: 95.0,
            archive_rule: ArchiveRule::default(),
            estimate_defaults: EstimateDefaults::default(),
            k: 1.0,
            verified_app_version: None,
            force_quit: false,
            accounts: Vec::new(),
        }
    }
}

fn default_cli_root() -> String {
    let home = std::env::var("USERPROFILE")
        .or_else(|_| std::env::var("HOME"))
        .unwrap_or_default();
    let root = std::path::Path::new(&home).join(".claude-accounts");
    root.display().to_string().replace('\\', "/")
}

impl AccountsConfig {
    /// The config of a `config.json` value; anything unreadable gives the defaults.
    #[must_use]
    pub fn from_json(value: &Value) -> Self {
        serde_json::from_value::<AccountsConfig>(value.clone())
            .unwrap_or_default()
            .synced()
    }

    /// Copies each account's `can_receive` into its rules, where the docs read it.
    fn synced(mut self) -> Self {
        for account in &mut self.accounts {
            account.rules.can_receive = account.can_receive;
        }
        self
    }

    /// The value to save under `accounts`.
    #[must_use]
    pub fn to_json(&self) -> Value {
        serde_json::to_value(self).unwrap_or(Value::Null)
    }

    /// This config with the keys of `patch` replaced, for a settings update. A patch that makes
    /// the config unreadable is refused.
    ///
    /// # Errors
    /// When the result is not a valid accounts config.
    pub fn patched(&self, patch: &Value) -> Result<Self, String> {
        let mut merged = self.to_json();
        let (Some(base), Some(over)) = (merged.as_object_mut(), patch.as_object()) else {
            return Err("send an object of settings".to_owned());
        };
        for (key, value) in over {
            base.insert(key.clone(), value.clone());
        }
        serde_json::from_value::<AccountsConfig>(merged)
            .map(Self::synced)
            .map_err(|e| e.to_string())
    }
}

// test-tag: critical
#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::{AccountsConfig, ArchiveRule};
    use crate::types::HandoverMode;

    #[test]
    fn empty_value_gives_defaults() {
        let c = AccountsConfig::from_json(&json!(null));
        assert!(c.enabled);
        assert_eq!(c.poll_minutes, 5);
        assert!(c.accounts.is_empty());
    }

    #[test]
    fn reads_the_documented_shape() {
        let c = AccountsConfig::from_json(&json!({
            "pollMinutes": 2, "archiveRule": "anywhere",
            "accounts": [{
                "key": "hotmail", "label": "Hotmail", "color": "#6f9cf0",
                "cliDir": "C:/x/hotmail", "canReceive": false,
                "rules": { "autoSwitch": true, "switchAt": 97, "handover": "auto" }
            }]
        }));
        assert_eq!(c.poll_minutes, 2);
        assert_eq!(c.archive_rule, ArchiveRule::Anywhere);
        let a = &c.accounts[0];
        assert!(a.rules.auto_switch && !a.rules.can_receive);
        assert_eq!(a.rules.handover, HandoverMode::Auto);
    }

    #[test]
    fn patch_replaces_keys_and_refuses_bad_ones() {
        let base = AccountsConfig::default();
        let next = base.patched(&json!({ "pollMinutes": 9 })).unwrap();
        assert_eq!(next.poll_minutes, 9);
        assert!(base.patched(&json!({ "pollMinutes": "soon" })).is_err());
        assert!(base.patched(&json!(3)).is_err());
    }

    #[test]
    fn force_quit_is_off_until_asked_for() {
        let base = AccountsConfig::default();
        assert!(!base.force_quit);
        let on = base.patched(&json!({ "forceQuit": true })).unwrap();
        assert!(on.force_quit);
        assert_eq!(on.to_json()["forceQuit"], json!(true));
    }

    #[test]
    fn a_full_percent_rule_saves_and_reads_back() {
        let account = json!({
            "key": "hotmail", "label": "Hotmail", "color": "#6f9cf0", "cliDir": "C:/x/hotmail",
            "rules": { "autoSwitch": true, "switchAt": 100, "handover": 99 }
        });
        let next = AccountsConfig::default()
            .patched(&json!({ "accounts": [account] }))
            .unwrap();
        let back = AccountsConfig::from_json(&next.to_json());
        let rules = &back.accounts[0].rules;
        assert!((rules.switch_at - 100.0).abs() < f64::EPSILON);
        assert_eq!(rules.handover, HandoverMode::At(99.0));
        assert_eq!(back.to_json()["accounts"][0]["rules"]["switchAt"], json!(100.0));
    }

    #[test]
    fn round_trips() {
        let c = AccountsConfig::default();
        assert_eq!(AccountsConfig::from_json(&c.to_json()), c);
    }
}
