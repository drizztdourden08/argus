//! The settings of the stall watch (`stall.*` of config.json).

use serde_json::Value;

use crate::config::section_ms;
use crate::stall_rules::Limits;

const MIN_MS: i64 = 60_000;

/// The settings of the stall watch. A value that is not a positive number keeps its default.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StallConfig {
    /// Is the watch on (`stall.enabled`)? When off nothing is raised, nudged or written.
    pub enabled: bool,
    /// How often the evaluator runs (`stall.tickMs`).
    pub tick_ms: i64,
    /// Rule 2: an unread wake is sent again after this long (`stall.nudgeMs`).
    pub nudge_ms: i64,
    /// Rule 2: the Overlord hears of an unread wake after this long (`stall.unreadMs`).
    pub unread_ms: i64,
    /// Rule 2: the owner hears of an unread wake after this long (`stall.ownerMs`).
    pub owner_ms: i64,
    /// Rule 3: a dependency open with its item still `todo` (`stall.unstartedMs`).
    pub unstarted_ms: i64,
    /// Rule 4: a lead idle with unblocked work (`stall.idleMs`).
    pub idle_ms: i64,
    /// Rule 5: an item `doing` with nothing happening (`stall.silentMs`).
    pub silent_ms: i64,
    /// The Overlord hears of an open stall again this often (`stall.resendMs`).
    pub resend_ms: i64,
    /// How far back rule 2 reads wake rows (`stall.horizonMs`).
    pub horizon_ms: i64,
    /// A process quieter than this many minutes is not work (`stall.procIdleMin`); the server reads it.
    pub proc_idle_min: i64,
}

impl Default for StallConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            tick_ms: MIN_MS,
            nudge_ms: 5 * MIN_MS,
            unread_ms: 10 * MIN_MS,
            owner_ms: 30 * MIN_MS,
            unstarted_ms: 30 * MIN_MS,
            idle_ms: 10 * MIN_MS,
            silent_ms: 60 * MIN_MS,
            resend_ms: 30 * MIN_MS,
            horizon_ms: 24 * 60 * MIN_MS,
            proc_idle_min: 10,
        }
    }
}

impl StallConfig {
    /// The settings from the `stall` section of config.json.
    #[must_use]
    pub fn from_json(section: &Value) -> Self {
        let d = Self::default();
        let ms = |key: &str, default: i64| section_ms(section, key, default);
        Self {
            enabled: section
                .get("enabled")
                .and_then(Value::as_bool)
                .unwrap_or(d.enabled),
            tick_ms: ms("tickMs", d.tick_ms),
            nudge_ms: ms("nudgeMs", d.nudge_ms),
            unread_ms: ms("unreadMs", d.unread_ms),
            owner_ms: ms("ownerMs", d.owner_ms),
            unstarted_ms: ms("unstartedMs", d.unstarted_ms),
            idle_ms: ms("idleMs", d.idle_ms),
            silent_ms: ms("silentMs", d.silent_ms),
            resend_ms: ms("resendMs", d.resend_ms),
            horizon_ms: ms("horizonMs", d.horizon_ms),
            proc_idle_min: ms("procIdleMin", d.proc_idle_min),
        }
    }

    /// The thresholds of rules 3, 4 and 5.
    #[must_use]
    pub fn limits(&self) -> Limits {
        Limits {
            unstarted_ms: self.unstarted_ms,
            idle_ms: self.idle_ms,
            silent_ms: self.silent_ms,
        }
    }
}
