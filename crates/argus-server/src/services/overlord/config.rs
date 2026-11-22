//! `config.overlord`: the parts the spend meter reads, with the plan's defaults.

use std::path::PathBuf;

use argus_overlord::mode::Mode;
use serde_json::Value;

use crate::Services;

const DEFAULT_BUDGET_USD: f64 = 100.0;
const DEFAULT_WARN_AT: f64 = 0.8;
const DEFAULT_MODEL: &str = "sonnet";
const DEFAULT_RESTART_MAX: u32 = 3;
const DEFAULT_APPROVAL_MIN: u64 = 30;

/// The monthly budget and the share of it that warns.
#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) struct Budget {
    pub budget_usd: f64,
    pub warn_at: f64,
}

fn share(setting: &Value, name: &str, default: f64, ok: impl Fn(f64) -> bool) -> f64 {
    setting[name]
        .as_f64()
        .filter(|v| v.is_finite() && ok(*v))
        .unwrap_or(default)
}

/// `budgetUsd` (default 100, not below 0) and `warnAt` (default 0.8, between 0 and 1).
pub(crate) fn budget(svc: &Services) -> Budget {
    let config = svc.config();
    let setting = config.extra.get("overlord").unwrap_or(&Value::Null);
    Budget {
        budget_usd: share(setting, "budgetUsd", DEFAULT_BUDGET_USD, |v| v >= 0.0),
        warn_at: share(setting, "warnAt", DEFAULT_WARN_AT, |v| {
            (0.0..=1.0).contains(&v)
        }),
    }
}

fn setting(svc: &Services) -> Value {
    svc.config()
        .extra
        .get("overlord")
        .cloned()
        .unwrap_or(Value::Null)
}

/// The `mode` as saved; a missing or unknown value is `live` (I-196).
pub(crate) fn mode(svc: &Services) -> Mode {
    Mode::of_config(setting(svc)["mode"].as_str())
}

/// True when `mode` is `live`, which a missing mode is too.
pub(crate) fn is_live(svc: &Services) -> bool {
    mode(svc) == Mode::Live
}

/// True when `mode` is `shadow`: a read-only child that mirrors the desktop Overlord's wakes.
pub(crate) fn is_shadow(svc: &Services) -> bool {
    mode(svc) == Mode::Shadow
}

/// `autostart` (default true): with mode `live` or `shadow`, Argus starts the child at boot.
pub(crate) fn autostart(svc: &Services) -> bool {
    setting(svc)["autostart"].as_bool().unwrap_or(true)
}

/// What the live child is started with.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct Live {
    /// The Overlord's folder; the dashboard home when none is set.
    pub cwd: Option<PathBuf>,
    pub model: String,
    /// The `claude` to run; the one the Desktop app carries when none is set.
    pub claude_exe: Option<PathBuf>,
    pub restart_max: u32,
    pub approval_timeout_min: u64,
}

fn text(setting: &Value, name: &str) -> Option<String> {
    setting[name]
        .as_str()
        .map(str::trim)
        .filter(|v| !v.is_empty())
        .map(str::to_owned)
}

/// `cwd`, `model` (default `sonnet`), `claudeExe`, `restartMax` (default 3) and
/// `approvalTimeoutMin` (default 30).
pub(crate) fn live(svc: &Services) -> Live {
    let setting = setting(svc);
    Live {
        cwd: text(&setting, "cwd").map(PathBuf::from),
        model: text(&setting, "model").unwrap_or_else(|| DEFAULT_MODEL.to_owned()),
        claude_exe: text(&setting, "claudeExe").map(PathBuf::from),
        restart_max: setting["restartMax"]
            .as_u64()
            .and_then(|v| u32::try_from(v).ok())
            .unwrap_or(DEFAULT_RESTART_MAX),
        approval_timeout_min: setting["approvalTimeoutMin"]
            .as_u64()
            .filter(|v| *v > 0)
            .unwrap_or(DEFAULT_APPROVAL_MIN),
    }
}
