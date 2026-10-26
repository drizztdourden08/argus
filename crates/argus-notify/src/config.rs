//! The `notify` key of `config.json`, read on every send.

use serde::{Deserialize, Serialize};

use crate::level::PerLevel;

/// Which board events ping the owner by themselves.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct AutoConfig {
    /// A build handed to the owner.
    pub handed: bool,
    /// A new open question by a thread.
    pub ask: bool,
}

impl Default for AutoConfig {
    fn default() -> Self {
        Self {
            handed: true,
            ask: true,
        }
    }
}

/// `notify: { os, auto, quietLevels, sound, toastInfo }`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct NotifyConfig {
    /// False records every notification without any toast.
    pub os: bool,
    /// The automatic pings.
    pub auto: AutoConfig,
    /// Levels that are recorded and never toasted.
    pub quiet_levels: Vec<String>,
    /// Sound per level; a level left out keeps its default.
    pub sound: PerLevel<Option<bool>>,
    /// True gives `info` a toast too.
    pub toast_info: bool,
}

impl Default for NotifyConfig {
    fn default() -> Self {
        Self {
            os: true,
            auto: AutoConfig::default(),
            quiet_levels: Vec::new(),
            sound: PerLevel::default(),
            toast_info: false,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::NotifyConfig;

    #[test]
    fn a_partial_config_keeps_the_defaults_of_what_it_leaves_out() {
        let config: NotifyConfig =
            serde_json::from_str(r#"{"quietLevels":["ping"],"sound":{"urgent":false}}"#)
                .expect("parses");
        assert!(config.os && config.auto.handed && config.auto.ask && !config.toast_info);
        assert_eq!(config.quiet_levels, ["ping"]);
        assert_eq!(config.sound.urgent, Some(false));
        assert_eq!(config.sound.ping, None);
    }
}
