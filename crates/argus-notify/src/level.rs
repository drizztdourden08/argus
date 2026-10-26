//! The three levels of a notification and the per-level view the settings use.

use serde::{Deserialize, Serialize};

/// How much a notification matters.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Level {
    /// Recorded only, unless the owner switched info toasts on.
    Info,
    /// The default: recorded and shown as a toast.
    #[default]
    Ping,
    /// Recorded and shown as a reminder toast that stays until it is dismissed.
    Urgent,
}

impl Level {
    /// Every level, quietest first.
    pub const ALL: [Self; 3] = [Self::Info, Self::Ping, Self::Urgent];

    /// The lowercase name used in records, config and routes.
    #[must_use]
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Info => "info",
            Self::Ping => "ping",
            Self::Urgent => "urgent",
        }
    }

    /// The level with this name, if it is one.
    #[must_use]
    pub fn parse(name: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|level| level.as_str() == name)
    }

    /// The level with this name; an absent or unknown name is a `ping`.
    #[must_use]
    pub fn or_ping(name: Option<&str>) -> Self {
        name.and_then(Self::parse).unwrap_or_default()
    }

    pub(crate) fn native(self) -> argus_native::Level {
        match self {
            Self::Info => argus_native::Level::Info,
            Self::Ping => argus_native::Level::Ping,
            Self::Urgent => argus_native::Level::Urgent,
        }
    }
}

/// One value for each level.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct PerLevel<T> {
    /// The value for `info`.
    pub info: T,
    /// The value for `ping`.
    pub ping: T,
    /// The value for `urgent`.
    pub urgent: T,
}

impl<T: Copy> PerLevel<T> {
    /// Builds the value of each level with `make`.
    pub fn from_fn(make: impl Fn(Level) -> T) -> Self {
        Self {
            info: make(Level::Info),
            ping: make(Level::Ping),
            urgent: make(Level::Urgent),
        }
    }

    /// The value for `level`.
    #[must_use]
    pub fn get(&self, level: Level) -> T {
        match level {
            Level::Info => self.info,
            Level::Ping => self.ping,
            Level::Urgent => self.urgent,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{Level, PerLevel};

    #[test]
    fn an_unknown_level_is_a_ping() {
        assert_eq!(Level::or_ping(Some("nonsense")), Level::Ping);
        assert_eq!(Level::or_ping(None), Level::Ping);
        assert_eq!(Level::or_ping(Some("urgent")), Level::Urgent);
    }

    #[test]
    fn a_value_is_built_and_read_per_level() {
        let lengths = PerLevel::from_fn(|level| level.as_str().len());
        assert_eq!(
            (lengths.get(Level::Info), lengths.get(Level::Urgent)),
            (4, 6)
        );
    }
}
