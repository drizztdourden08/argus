//! The monitor's settings: config.json `procs`.

use crate::num::to_millis;
use serde_json::Value;

pub const DEFAULT_SAMPLE_MS: u64 = 30_000;
pub const DEFAULT_REPEAT_MS: u64 = 12 * 3_600_000;
pub const DEFAULT_START_DELAY_MS: u64 = 3000;
const MIN_SAMPLE_MS: f64 = 1000.0;

/// How often to sample, how long an alert waits before it repeats, and the delay of the first sample.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ProcsConfig {
    pub sample_ms: u64,
    pub repeat_ms: u64,
    pub start_delay_ms: u64,
}

impl Default for ProcsConfig {
    fn default() -> Self {
        Self {
            sample_ms: DEFAULT_SAMPLE_MS,
            repeat_ms: DEFAULT_REPEAT_MS,
            start_delay_ms: DEFAULT_START_DELAY_MS,
        }
    }
}

fn setting(config: &Value, key: &str, default: u64, min: f64) -> u64 {
    let number = match config.get(key) {
        Some(Value::Number(n)) => n.as_f64(),
        Some(Value::String(s)) if !s.is_empty() => s.trim().parse().ok(),
        _ => None,
    };
    number
        .filter(|n| n.is_finite() && *n >= min)
        .map_or(default, to_millis)
}

impl ProcsConfig {
    /// Reads `sampleMs`, `repeatMs` and `startDelayMs`; a missing or bad value keeps its default.
    #[must_use]
    pub fn from_json(config: &Value) -> Self {
        Self {
            sample_ms: setting(config, "sampleMs", DEFAULT_SAMPLE_MS, MIN_SAMPLE_MS),
            repeat_ms: setting(config, "repeatMs", DEFAULT_REPEAT_MS, 1.0),
            start_delay_ms: setting(config, "startDelayMs", DEFAULT_START_DELAY_MS, 0.0),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn bad_values_keep_the_defaults() {
        let cfg = ProcsConfig::from_json(
            &json!({"sampleMs": 500, "repeatMs": "3600000", "startDelayMs": 0}),
        );
        assert_eq!(cfg.sample_ms, DEFAULT_SAMPLE_MS);
        assert_eq!(cfg.repeat_ms, 3_600_000);
        assert_eq!(cfg.start_delay_ms, 0);
        assert_eq!(ProcsConfig::from_json(&Value::Null), ProcsConfig::default());
    }
}
