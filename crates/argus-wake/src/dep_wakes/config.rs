//! The settings of the dependency wakes.

use serde_json::Value;

use crate::config::section_ms;

/// The settings of the dependency wakes (`deps.*` of config.json).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DepWakeConfig {
    /// Are dependencies on (`deps.enabled`)? When off no wake is made.
    pub enabled: bool,
    /// How often the owner calls `tick`.
    pub tick_ms: i64,
}

impl Default for DepWakeConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            tick_ms: 30_000,
        }
    }
}

impl DepWakeConfig {
    /// The settings from the `deps` section of config.json; a value that is not a positive number keeps its default.
    #[must_use]
    pub fn from_json(section: &Value) -> Self {
        let d = Self::default();
        Self {
            enabled: section
                .get("enabled")
                .and_then(Value::as_bool)
                .unwrap_or(d.enabled),
            tick_ms: section_ms(section, "tickMs", d.tick_ms),
        }
    }
}
