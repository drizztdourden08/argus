//! The `notify` key of the live config, read on every send and written to `config.json` when
//! the owner changes the settings.

use std::sync::{Arc, PoisonError, RwLock};

use argus_core::config::save_extra_key;
use argus_core::{Config, Paths, StorageError};
use argus_notify::{ConfigStore, NotifyConfig};
use serde_json::{Map, Value, to_value};

const KEY: &str = "notify";

/// The notification settings inside the live config.
pub struct LiveConfig {
    config: Arc<RwLock<Config>>,
    paths: Paths,
    env_port: Option<u16>,
}

impl LiveConfig {
    /// The settings of a live config; `env_port` is the port `config.json` must not take.
    #[must_use]
    pub fn new(config: Arc<RwLock<Config>>, paths: Paths, env_port: Option<u16>) -> Self {
        LiveConfig {
            config,
            paths,
            env_port,
        }
    }
}

/// The key as it was, with every key of `next` written over it, so a key this build does not
/// know stays.
fn merged(before: Option<&Value>, next: &NotifyConfig) -> Result<Value, StorageError> {
    let fresh = to_value(next).map_err(|e| StorageError(e.to_string()))?;
    let mut out: Map<String, Value> = before
        .and_then(Value::as_object)
        .cloned()
        .unwrap_or_default();
    if let Value::Object(set) = fresh {
        out.extend(set.into_iter().filter(|(_, v)| !v.is_null()));
    }
    Ok(Value::Object(out))
}

impl ConfigStore for LiveConfig {
    fn load(&self) -> NotifyConfig {
        let config = self.config.read().unwrap_or_else(PoisonError::into_inner);
        config
            .extra
            .get(KEY)
            .and_then(|v| serde_json::from_value(v.clone()).ok())
            .unwrap_or_default()
    }

    fn save(&self, config: &NotifyConfig) -> Result<(), StorageError> {
        let mut live = self.config.write().unwrap_or_else(PoisonError::into_inner);
        let before = live.extra.get(KEY).cloned();
        let value = merged(before.as_ref(), config)?;
        save_extra_key(&self.paths, &mut live, self.env_port, KEY, value)
    }
}
