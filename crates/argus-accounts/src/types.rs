//! The shapes the config and the docs share. All serialize as camelCase.

use serde::de::{self, Deserializer};
use serde::ser::Serializer;
use serde::{Deserialize, Serialize};
use serde_json::Value;

/// The short name of an account, also the name of its CLI folder (`hotmail`).
pub type AccountKey = String;

/// One Desktop scope: `claude-code-sessions/<account>/<org>`.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Scope {
    /// The account uuid.
    pub account: String,
    /// The organization uuid.
    pub org: String,
}

impl Scope {
    /// The scope as the Desktop app names it, `<account>/<org>`.
    #[must_use]
    pub fn name(&self) -> String {
        format!("{}/{}", self.account, self.org)
    }
}

/// When the owner wants a handover started for an account.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub enum HandoverMode {
    /// Never.
    #[default]
    Off,
    /// At a fixed percent of the limit.
    At(f64),
    /// At a point Argus works out from the burn rate.
    Auto,
}

impl Serialize for HandoverMode {
    fn serialize<S: Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        match self {
            HandoverMode::Off => s.serialize_str("off"),
            HandoverMode::Auto => s.serialize_str("auto"),
            HandoverMode::At(p) => s.serialize_f64(*p),
        }
    }
}

impl<'de> Deserialize<'de> for HandoverMode {
    fn deserialize<D: Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        match Value::deserialize(d)? {
            Value::String(s) if s == "off" => Ok(HandoverMode::Off),
            Value::String(s) if s == "auto" => Ok(HandoverMode::Auto),
            Value::Number(n) => n
                .as_f64()
                .map(HandoverMode::At)
                .ok_or_else(|| de::Error::custom("handover is not a number")),
            other => Err(de::Error::custom(format!(
                "handover must be \"off\", \"auto\" or a percent, got {other}"
            ))),
        }
    }
}

/// The two switching rules of an account.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct Rules {
    /// Switch away by itself when the account is nearly spent.
    pub auto_switch: bool,
    /// The percent at which the switch countdown starts.
    pub switch_at: f64,
    /// The handover rule.
    pub handover: HandoverMode,
    /// Copy of the account's `can_receive`, so the doc carries it; the account's is the truth.
    pub can_receive: bool,
}

impl Default for Rules {
    fn default() -> Self {
        Rules {
            auto_switch: false,
            switch_at: 98.0,
            handover: HandoverMode::Off,
            can_receive: true,
        }
    }
}

/// One Claude account of the owner.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Account {
    /// The short name.
    pub key: AccountKey,
    /// The name shown on screen.
    pub label: String,
    /// The color of its chips, a `#rrggbb` string.
    pub color: String,
    /// The Desktop account uuid, when the Desktop app has seen this login.
    #[serde(default)]
    pub desktop_account: Option<String>,
    /// The Desktop organization uuid, the one the CLI login is in.
    #[serde(default)]
    pub desktop_org: Option<String>,
    /// The folder `CLAUDE_CONFIG_DIR` points at for this account.
    pub cli_dir: String,
    /// A switch may land here.
    #[serde(default = "yes")]
    pub can_receive: bool,
    /// The switching rules.
    #[serde(default)]
    pub rules: Rules,
}

fn yes() -> bool {
    true
}

/// What `claude auth status --json` says about a CLI login.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CliStatus {
    /// Whether the folder holds a valid login.
    #[serde(default)]
    pub logged_in: bool,
    /// The organization uuid.
    #[serde(default)]
    pub org_id: Option<String>,
    /// The organization name.
    #[serde(default)]
    pub org_name: Option<String>,
    /// The plan, as the CLI names it.
    #[serde(default)]
    pub subscription_type: Option<String>,
    /// When the login expires.
    #[serde(default)]
    pub login_expires_at: Option<Value>,
}

/// How near a limit is to its end.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Severity {
    /// Getting near.
    Warning,
    /// Nearly or fully spent.
    Critical,
    /// Plenty left; also what an unknown word reads as.
    #[default]
    #[serde(other)]
    Normal,
}

/// One limit of an account: the weekly total, a per-model weekly, the five-hour window.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UsageLimit {
    /// The endpoint's kind (`weekly_all`, `weekly_scoped`, ...); unknown kinds are kept.
    pub kind: String,
    /// The endpoint's group.
    #[serde(default)]
    pub group: Option<String>,
    /// Percent used, 0 to 100.
    pub percent: f64,
    /// How near the end.
    #[serde(default)]
    pub severity: Severity,
    /// When it resets, ISO 8601.
    #[serde(default)]
    pub resets_at: Option<String>,
    /// The model a scoped limit is for.
    #[serde(default)]
    pub model: Option<String>,
    /// Whether this limit is the one currently binding.
    #[serde(default)]
    pub is_active: bool,
}

/// The last usage read of an account.
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Usage {
    /// Every limit the endpoint listed.
    pub limits: Vec<UsageLimit>,
    /// When it was read, ISO 8601.
    #[serde(default)]
    pub read_at: Option<String>,
    /// Why the last read failed, when it did.
    #[serde(default)]
    pub error: Option<String>,
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::{HandoverMode, Rules, Severity, UsageLimit};

    #[test]
    fn handover_reads_the_three_spellings() {
        let read = |v| serde_json::from_value::<HandoverMode>(v).unwrap();
        assert_eq!(read(json!("off")), HandoverMode::Off);
        assert_eq!(read(json!("auto")), HandoverMode::Auto);
        assert_eq!(read(json!(95.5)), HandoverMode::At(95.5));
        assert!(serde_json::from_value::<HandoverMode>(json!("soon")).is_err());
    }

    #[test]
    fn handover_writes_back_as_read() {
        assert_eq!(
            serde_json::to_value(HandoverMode::Auto).unwrap(),
            json!("auto")
        );
        assert_eq!(
            serde_json::to_value(HandoverMode::At(90.0)).unwrap(),
            json!(90.0)
        );
    }

    #[test]
    fn rules_default_when_missing() {
        let r: Rules = serde_json::from_value(json!({ "autoSwitch": true })).unwrap();
        assert!(r.auto_switch);
        assert!((r.switch_at - 98.0).abs() < f64::EPSILON);
        assert_eq!(r.handover, HandoverMode::Off);
    }

    #[test]
    fn unknown_severity_reads_as_normal() {
        let l: UsageLimit =
            serde_json::from_value(json!({ "kind": "x", "percent": 3.0, "severity": "odd" }))
                .unwrap();
        assert_eq!(l.severity, Severity::Normal);
    }
}
