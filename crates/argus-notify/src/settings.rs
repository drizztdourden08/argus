//! The per-level view of the config: which levels toast and which toasts play a sound.
// test-tag: critical

use serde::{Deserialize, Serialize};

use crate::config::NotifyConfig;
use crate::level::{Level, PerLevel};

/// The sound each level has until the owner says otherwise.
pub const SOUND_DEFAULTS: PerLevel<bool> = PerLevel {
    info: false,
    ping: false,
    urgent: true,
};

/// The settings in effect, as the Notifications tab shows them.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Settings {
    /// False when notifications are off altogether.
    pub os: bool,
    /// Whether each level shows a toast.
    pub toast: PerLevel<bool>,
    /// Whether the toast of each level plays the system sound.
    pub sound: PerLevel<bool>,
}

/// The settings a config gives: `info` toasts only with `toastInfo`, a quiet level never toasts.
#[must_use]
pub fn settings_of(config: &NotifyConfig) -> Settings {
    let quiet = |level: Level| {
        config
            .quiet_levels
            .iter()
            .any(|name| name == level.as_str())
    };
    Settings {
        os: config.os,
        toast: PerLevel::from_fn(|level| {
            (level != Level::Info || config.toast_info) && !quiet(level)
        }),
        sound: PerLevel::from_fn(|level| {
            config
                .sound
                .get(level)
                .unwrap_or_else(|| SOUND_DEFAULTS.get(level))
        }),
    }
}

#[cfg(test)]
mod tests {
    use super::settings_of;
    use crate::config::NotifyConfig;
    use crate::level::Level;

    #[test]
    fn a_quiet_level_wins_over_toast_info() {
        let config = NotifyConfig {
            toast_info: true,
            quiet_levels: vec!["info".into()],
            ..NotifyConfig::default()
        };
        assert!(!settings_of(&config).toast.get(Level::Info));
    }
}
