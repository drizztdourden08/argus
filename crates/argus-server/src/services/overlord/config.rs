//! `config.overlord`: the parts the spend meter reads, with the plan's defaults.

use std::path::PathBuf;

use argus_overlord::mode::Mode;
use argus_overlord::session::choices::{
    DEFAULT_EFFORT, DEFAULT_PERMISSION_MODE, is_effort, is_permission_mode,
};
use serde_json::Value;

use crate::Services;

const DEFAULT_WARN_AT: f64 = 0.8;
const DEFAULT_MODEL: &str = "sonnet";
const DEFAULT_RESTART_MAX: u32 = 3;
const DEFAULT_APPROVAL_MIN: u64 = 30;

/// The monthly budget, when one is set, and the share of it that warns.
#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) struct Budget {
    /// `None` is no cap, the default.
    pub budget_usd: Option<f64>,
    pub warn_at: f64,
}

fn share(setting: &Value, name: &str, default: f64, ok: impl Fn(f64) -> bool) -> f64 {
    setting[name]
        .as_f64()
        .filter(|v| v.is_finite() && ok(*v))
        .unwrap_or(default)
}

/// `budgetUsd` (default none; a missing, null or 0 value is no cap) and `warnAt` (default 0.8, between 0 and 1).
pub(crate) fn budget(svc: &Services) -> Budget {
    let config = svc.config();
    let setting = config.extra.get("overlord").unwrap_or(&Value::Null);
    Budget {
        budget_usd: setting["budgetUsd"]
            .as_f64()
            .filter(|v| v.is_finite() && *v > 0.0),
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
    /// The Overlord's folder; the root of the Overlord's board when none is set, else the dashboard home.
    pub cwd: Option<PathBuf>,
    pub model: String,
    /// The `--effort` level.
    pub effort: String,
    /// The `--permission-mode`.
    pub permission_mode: String,
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

/// The program path: `exePath`, else the `claudeExe` an older config wrote.
fn exe_text(setting: &Value) -> Option<String> {
    text(setting, "exePath").or_else(|| text(setting, "claudeExe"))
}

/// A text field that is one of the choices `ok` takes, else `default`.
fn picked(setting: &Value, name: &str, ok: fn(&str) -> bool, default: &str) -> String {
    text(setting, name)
        .filter(|v| ok(v))
        .unwrap_or_else(|| default.to_owned())
}

/// `cwd`, `model` (default `sonnet`), `effort` (default `high`), `permissionMode` (default
/// `bypassPermissions`), `exePath` (the old `claudeExe` is read too), `restartMax` (default 3) and `approvalTimeoutMin`
/// (default 30). An effort or a mode that is not one of the choices is the default.
pub(crate) fn live(svc: &Services) -> Live {
    let setting = setting(svc);
    Live {
        cwd: text(&setting, "cwd").map(PathBuf::from),
        model: text(&setting, "model").unwrap_or_else(|| DEFAULT_MODEL.to_owned()),
        effort: picked(&setting, "effort", is_effort, DEFAULT_EFFORT),
        permission_mode: picked(
            &setting,
            "permissionMode",
            is_permission_mode,
            DEFAULT_PERMISSION_MODE,
        ),
        claude_exe: exe_text(&setting).map(PathBuf::from),
        restart_max: restart_max_of(&setting).unwrap_or(DEFAULT_RESTART_MAX),
        approval_timeout_min: approval_of(&setting).unwrap_or(DEFAULT_APPROVAL_MIN),
    }
}

fn restart_max_of(setting: &Value) -> Option<u32> {
    setting["restartMax"]
        .as_u64()
        .and_then(|v| u32::try_from(v).ok())
}

fn approval_of(setting: &Value) -> Option<u64> {
    setting["approvalTimeoutMin"].as_u64().filter(|v| *v > 0)
}

/// A setting of the page that `config.json` can hold a value for.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Field {
    ExePath,
    Budget,
    RestartMax,
    ApprovalTimeout,
}

/// Which of the page's settings `config.json` holds a usable value for.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct Saved(Vec<Field>);

impl Saved {
    pub(crate) fn has(&self, field: Field) -> bool {
        self.0.contains(&field)
    }
}

/// What `config.json` sets, so the page can say where each value in effect came from.
pub(crate) fn saved(svc: &Services) -> Saved {
    let setting = setting(svc);
    let held = [
        (Field::ExePath, exe_text(&setting).is_some()),
        (Field::Budget, budget(svc).budget_usd.is_some()),
        (Field::RestartMax, restart_max_of(&setting).is_some()),
        (Field::ApprovalTimeout, approval_of(&setting).is_some()),
    ];
    Saved(
        held.into_iter()
            .filter_map(|(field, set)| set.then_some(field))
            .collect(),
    )
}
