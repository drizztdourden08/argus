//! Changing the per-level settings from the owner's page.
// test-tag: critical

use crate::config::NotifyConfig;
use crate::level::{Level, PerLevel};
use crate::settings::{Settings, settings_of};

/// A change of the settings: any subset of levels, for the toast and for the sound.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct SettingsPatch {
    /// Switch the toast of a level on or off.
    pub toast: PerLevel<Option<bool>>,
    /// Switch the sound of a level on or off.
    pub sound: PerLevel<Option<bool>>,
}

/// The config after the patch: `info` follows `toastInfo`, the other levels `quietLevels`, and the
/// sound of every level is written out. Everything else stays as it was.
#[must_use]
pub fn apply(config: &NotifyConfig, patch: SettingsPatch) -> NotifyConfig {
    let before = settings_of(config);
    let mut next = config.clone();
    if let Some(on) = patch.toast.info {
        next.toast_info = on;
    }
    next.quiet_levels = quiet_after(config, patch.toast);
    next.sound = PerLevel::from_fn(|level| {
        Some(
            patch
                .sound
                .get(level)
                .unwrap_or_else(|| before.sound.get(level)),
        )
    });
    next
}

fn quiet_after(config: &NotifyConfig, toast: PerLevel<Option<bool>>) -> Vec<String> {
    Level::ALL
        .into_iter()
        .filter(|level| *level != Level::Info)
        .filter(|level| match toast.get(*level) {
            Some(on) => !on,
            None => config
                .quiet_levels
                .iter()
                .any(|name| name == level.as_str()),
        })
        .map(|level| level.as_str().to_owned())
        .collect()
}

/// The sentence of the activity line for a settings change.
#[must_use]
pub fn describe(settings: Settings) -> String {
    let parts: Vec<String> = Level::ALL
        .into_iter()
        .map(|level| {
            let name = level.as_str();
            match (settings.toast.get(level), settings.sound.get(level)) {
                (false, _) => format!("{name} no toast"),
                (true, true) => format!("{name} toast with sound"),
                (true, false) => format!("{name} toast silent"),
            }
        })
        .collect();
    format!(
        "notification settings changed from the page: {}",
        parts.join(", ")
    )
}

#[cfg(test)]
mod tests {
    use super::{SettingsPatch, apply, describe};
    use crate::config::NotifyConfig;
    use crate::level::{Level, PerLevel};
    use crate::settings::settings_of;

    #[test]
    fn a_quiet_ping_and_a_loud_info_are_written_to_the_config() {
        let patch = SettingsPatch {
            toast: PerLevel::from_fn(|level| Some(level != Level::Ping)),
            sound: PerLevel::from_fn(|_| Some(true)),
        };
        let next = apply(&NotifyConfig::default(), patch);
        assert_eq!(next.quiet_levels, ["ping"]);
        assert!(next.toast_info);
        assert_eq!(next.sound, PerLevel::from_fn(|_| Some(true)));
        assert_eq!(
            describe(settings_of(&next)),
            "notification settings changed from the page: info toast with sound, ping no toast, urgent toast with sound"
        );
    }

    #[test]
    fn switching_a_quiet_level_on_removes_it_and_an_empty_patch_changes_nothing_else() {
        let config = NotifyConfig {
            quiet_levels: vec!["ping".into()],
            ..NotifyConfig::default()
        };
        let on = SettingsPatch {
            toast: PerLevel {
                ping: Some(true),
                ..PerLevel::default()
            },
            ..SettingsPatch::default()
        };
        assert_eq!(apply(&config, on).quiet_levels, Vec::<String>::new());
        let same = apply(&config, SettingsPatch::default());
        assert_eq!(same.quiet_levels, ["ping"]);
        assert!(same.os && same.auto.handed);
    }
}
