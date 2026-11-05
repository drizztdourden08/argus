//! The OS switch and the five automatic pings, as the app settings dialog shows and changes them.
// test-tag: critical

use serde::Serialize;

use crate::config::{AutoConfig, NotifyConfig};

/// The OS switch and which board events ping the owner by themselves.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub struct AutoSettings {
    /// False when notifications are off altogether (`notify.os`).
    pub os: bool,
    /// The five automatic pings: `handed`, `ask`, `assigned`, `verify`, `run`.
    pub auto: AutoConfig,
}

/// A change: any subset of the switches. Absent is left as it was.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct AutoPatch {
    /// The OS switch.
    pub os: Option<bool>,
    /// A build handed to the owner.
    pub handed: Option<bool>,
    /// A new open question.
    pub ask: Option<bool>,
    /// Work assigned to the owner.
    pub assigned: Option<bool>,
    /// A bug ready to verify.
    pub verify: Option<bool>,
    /// A test plan ready to run.
    pub run: Option<bool>,
}

/// The switches a config gives.
#[must_use]
pub fn auto_of(config: &NotifyConfig) -> AutoSettings {
    AutoSettings {
        os: config.os,
        auto: config.auto,
    }
}

/// The config after the patch; toast and sound settings stay as they were.
#[must_use]
pub fn apply_auto(config: &NotifyConfig, patch: AutoPatch) -> NotifyConfig {
    let mut next = config.clone();
    let set = |slot: &mut bool, on: Option<bool>| *slot = on.unwrap_or(*slot);
    set(&mut next.os, patch.os);
    set(&mut next.auto.board.handed, patch.handed);
    set(&mut next.auto.board.ask, patch.ask);
    set(&mut next.auto.owner.assigned, patch.assigned);
    set(&mut next.auto.owner.verify, patch.verify);
    set(&mut next.auto.owner.run, patch.run);
    next
}

/// The sentence of the activity line for a change.
#[must_use]
pub fn describe_auto(settings: AutoSettings) -> String {
    let word = |on: bool| if on { "on" } else { "off" };
    let auto = settings.auto;
    format!(
        "automatic notifications changed from the page: os {}, handed {}, ask {}, assigned {}, verify {}, run {}",
        word(settings.os),
        word(auto.board.handed),
        word(auto.board.ask),
        word(auto.owner.assigned),
        word(auto.owner.verify),
        word(auto.owner.run),
    )
}

#[cfg(test)]
mod tests {
    use super::{AutoPatch, apply_auto, auto_of};
    use crate::config::NotifyConfig;

    #[test]
    fn a_partial_patch_changes_only_what_it_sends() {
        let config = NotifyConfig::default();
        let patch = AutoPatch {
            ask: Some(false),
            ..AutoPatch::default()
        };
        let next = apply_auto(&config, patch);
        let now = auto_of(&next);
        assert!(!now.auto.board.ask && now.auto.board.handed && now.os);
        assert_eq!(next.sound, config.sound);
        assert_eq!(next.quiet_levels, config.quiet_levels);
    }
}
