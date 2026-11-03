//! The settings of the reminders.

use serde_json::Value;

use crate::config::section_ms;

/// How often the Overlord hears again about what is still open, when config.json does not say.
pub const DEFAULT_RESEND_MS: i64 = 15 * 60_000;

/// How long a fault is open before the owner hears of it, when config.json does not say.
pub const DEFAULT_OWNER_MS: i64 = 60 * 60_000;

/// The settings of the reminders (`reminders.*` of config.json).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReminderConfig {
    /// The Overlord hears the coalesced line again this often (`reminders.resendMs`).
    pub resend_ms: i64,
    /// The owner hears of a fault open this long (`reminders.ownerMs`).
    pub owner_ms: i64,
}

impl Default for ReminderConfig {
    fn default() -> Self {
        Self {
            resend_ms: DEFAULT_RESEND_MS,
            owner_ms: DEFAULT_OWNER_MS,
        }
    }
}

impl ReminderConfig {
    /// The settings from the `reminders` section of config.json; a value that is not a positive number
    /// keeps its default.
    #[must_use]
    pub fn from_json(section: &Value) -> Self {
        let d = Self::default();
        Self {
            resend_ms: section_ms(section, "resendMs", d.resend_ms),
            owner_ms: section_ms(section, "ownerMs", d.owner_ms),
        }
    }
}
