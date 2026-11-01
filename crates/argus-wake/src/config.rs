//! Reading the wake settings of config.json the way the Node dashboard does.

use std::time::Duration;

use serde_json::Value;

use crate::agent_wakes::AgentWakeConfig;
use crate::item_wakes::ItemWakeConfig;
use crate::registry::{
    DEFAULT_ALIVE_MS, DEFAULT_BOARD_GRACE_MS, DEFAULT_NO_LISTENER_MS, DEFAULT_REACTION_MS,
    DEFAULT_TICK_MS, RegistryConfig,
};
use crate::sync_wakes::SyncWakeConfig;
use crate::wake_policy::WakePolicy;

/// The coalescing window when config.json does not say.
pub const DEFAULT_WINDOW_MS: i64 = 2000;

/// The restart grace when config.json does not say.
pub const DEFAULT_RESTART_GRACE_MS: i64 = 90_000;

fn number(value: Option<&Value>) -> Option<f64> {
    let n = match value? {
        Value::Number(n) => n.as_f64()?,
        Value::String(s) if !s.trim().is_empty() => s.trim().parse().ok()?,
        _ => return None,
    };
    n.is_finite().then_some(n)
}

fn whole_ms(n: f64) -> Option<i64> {
    let secs = Duration::try_from_secs_f64(n.abs() / 1000.0).ok()?;
    let ms = i64::try_from(secs.as_millis()).ok()?;
    Some(if n < 0.0 { -ms } else { ms })
}

/// A positive number of milliseconds, else the default.
#[must_use]
pub fn positive_ms(value: Option<&Value>, default: i64) -> i64 {
    number(value)
        .filter(|n| *n > 0.0)
        .and_then(whole_ms)
        .unwrap_or(default)
}

/// A positive number of milliseconds under `key` of a config section, else the default.
#[must_use]
pub fn section_ms(section: &Value, key: &str, default: i64) -> i64 {
    positive_ms(section.get(key), default)
}

/// A number of milliseconds of at least `min`, else the default.
#[must_use]
pub fn ms_at_least(value: Option<&Value>, default: i64, min: i64) -> i64 {
    number(value)
        .and_then(whole_ms)
        .filter(|ms| *ms >= min)
        .unwrap_or(default)
}

/// Every setting of the wake service.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WakeConfig {
    /// The coalescing window (`wake.windowMs`); 0 closes at once.
    pub window_ms: i64,
    /// The registry timeouts (`wake.*`).
    pub registry: RegistryConfig,
    /// The agent reminders (`agents.*`).
    pub agents: AgentWakeConfig,
    /// The Kanban reminders (`items.*`).
    pub items: ItemWakeConfig,
    /// The remote sync wake (`remoteSync.*`).
    pub syncs: SyncWakeConfig,
    /// How often one thread may be woken (`wakePolicy.*`).
    pub policy: WakePolicy,
}

impl Default for WakeConfig {
    fn default() -> Self {
        Self::from_json(&Value::Null)
    }
}

impl WakeConfig {
    /// The settings of a parsed config.json (anything missing or invalid keeps its default).
    #[must_use]
    pub fn from_json(config: &Value) -> Self {
        let wake = config.get("wake").unwrap_or(&Value::Null);
        let ms = |key: &str, default: i64, min: i64| ms_at_least(wake.get(key), default, min);
        let registry = RegistryConfig {
            reaction_ms: ms("reactionMs", DEFAULT_REACTION_MS, 1),
            no_listener_ms: ms("noListenerMs", DEFAULT_NO_LISTENER_MS, 1),
            tick_ms: ms("tickMs", DEFAULT_TICK_MS, 20),
            board_grace_ms: ms("boardGraceMs", DEFAULT_BOARD_GRACE_MS, 1),
            restart_grace_ms: ms("restartGraceMs", DEFAULT_RESTART_GRACE_MS, 0),
            deaf_grace_ms: ms("deafGraceMs", DEFAULT_REACTION_MS, 1),
            alive_ms: ms("aliveMs", DEFAULT_ALIVE_MS, 1),
        };
        Self {
            window_ms: ms("windowMs", DEFAULT_WINDOW_MS, 0),
            registry,
            agents: AgentWakeConfig::from_json(config.get("agents").unwrap_or(&Value::Null)),
            items: ItemWakeConfig::from_json(config.get("items").unwrap_or(&Value::Null)),
            syncs: SyncWakeConfig::from_json(config.get("remoteSync").unwrap_or(&Value::Null)),
            policy: WakePolicy::from_json(config.get("wakePolicy").unwrap_or(&Value::Null)),
        }
    }
}
