//! The `notify` key of `config.json`, read on every send.
// test-tag: critical

use serde::{Deserialize, Serialize};

use crate::level::PerLevel;

/// The pings about the board's own news.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct BoardTopics {
    /// A build handed to the owner.
    pub handed: bool,
    /// A new open question by a thread.
    pub ask: bool,
}

impl Default for BoardTopics {
    fn default() -> Self {
        Self {
            handed: true,
            ask: true,
        }
    }
}

/// The pings about work waiting for the owner.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct OwnerTopics {
    /// An item or a to-do assigned to the owner by a thread (`notify.auto.assigned`).
    pub assigned: bool,
    /// A bug the owner reported is ready to verify (`notify.auto.verify`).
    pub verify: bool,
    /// A test plan is ready to run (`notify.auto.run`).
    pub run: bool,
}

impl Default for OwnerTopics {
    fn default() -> Self {
        Self {
            assigned: true,
            verify: true,
            run: true,
        }
    }
}

/// Which board events ping the owner by themselves. The two groups read and write as one flat
/// object: `handed`, `ask`, `assigned`, `verify`, `run`.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct AutoConfig {
    /// The pings about the board's news.
    #[serde(flatten)]
    pub board: BoardTopics,
    /// The pings about work waiting for the owner.
    #[serde(flatten)]
    pub owner: OwnerTopics,
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
        assert!(
            config.os && config.auto.board.handed && config.auto.board.ask && !config.toast_info
        );
        assert!(config.auto.owner.assigned && config.auto.owner.verify && config.auto.owner.run);
        assert_eq!(config.quiet_levels, ["ping"]);
        assert_eq!(config.sound.urgent, Some(false));
        assert_eq!(config.sound.ping, None);
    }
}
