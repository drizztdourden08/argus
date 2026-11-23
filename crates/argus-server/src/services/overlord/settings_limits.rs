//! The settings of the hosted child that name a folder, a program or a limit (I-207): the working
//! folder, the `claude` program, the monthly budget, the restart cap and the approval wait. Each is
//! shown with its value in effect and where that came from, and each can be set or reset.

use std::path::Path;

use serde_json::{Value, json};

use super::config::{self, Field, Saved};
use super::live::{find_exe, overlord_root};
use crate::Services;

const MAX_BUDGET_USD: f64 = 10_000.0;
const MAX_RESTARTS: u64 = 20;
const MAX_APPROVAL_MIN: u64 = 1440;

/// What a request asks of one field.
#[derive(Debug, Clone, PartialEq)]
pub(crate) enum Edit<T> {
    Keep,
    /// An empty value: the field goes back to its default.
    Reset,
    Set(T),
}

/// The changes to the folder, program and limits.
#[derive(Debug, Clone, PartialEq)]
pub(crate) struct Limits {
    pub cwd: Edit<String>,
    pub claude_exe: Edit<String>,
    pub budget_usd: Edit<f64>,
    pub restart_max: Edit<u64>,
    pub approval_timeout_min: Edit<u64>,
}

fn text_edit(request: &Value, name: &str) -> Result<Edit<String>, String> {
    match request.get(name) {
        None | Some(Value::Null) => Ok(Edit::Keep),
        Some(Value::String(v)) if v.trim().is_empty() => Ok(Edit::Reset),
        Some(Value::String(v)) => Ok(Edit::Set(v.trim().to_owned())),
        Some(_) => Err(format!("{name} must be text")),
    }
}

fn number_edit<T>(
    request: &Value,
    name: &str,
    read: impl Fn(&Value) -> Option<T>,
) -> Result<Edit<T>, String> {
    match request.get(name) {
        None | Some(Value::Null) => Ok(Edit::Keep),
        Some(Value::String(v)) if v.trim().is_empty() => Ok(Edit::Reset),
        Some(v) => read(v)
            .map(Edit::Set)
            .ok_or_else(|| format!("{name} must be a number")),
    }
}

fn existing_folder(v: &str) -> Result<Value, &'static str> {
    let path = Path::new(v);
    (path.is_absolute() && path.is_dir())
        .then(|| json!(v))
        .ok_or("cwd must be the full path of an existing folder")
}

fn existing_file(v: &str) -> Result<Value, &'static str> {
    let path = Path::new(v);
    (path.is_absolute() && path.is_file())
        .then(|| json!(v))
        .ok_or("exePath must be the full path of an existing file")
}

fn sane_budget(v: f64) -> Result<Value, &'static str> {
    (v.is_finite() && v > 0.0 && v <= MAX_BUDGET_USD)
        .then(|| json!(v))
        .ok_or("budgetUsd must be above 0 and at most 10000")
}

fn sane_restarts(v: u64) -> Result<Value, &'static str> {
    (v <= MAX_RESTARTS)
        .then(|| json!(v))
        .ok_or("restartMax must be a whole number from 0 to 20")
}

fn sane_wait(v: u64) -> Result<Value, &'static str> {
    (1..=MAX_APPROVAL_MIN)
        .contains(&v)
        .then(|| json!(v))
        .ok_or("approvalTimeoutMin must be a whole number from 1 to 1440")
}

fn put<T>(
    out: &mut Vec<(String, Value)>,
    name: &str,
    edit: &Edit<T>,
    check: impl Fn(&T) -> Result<Value, &'static str>,
) -> Result<(), String> {
    match edit {
        Edit::Keep => {}
        Edit::Reset => out.push((name.to_owned(), Value::Null)),
        Edit::Set(v) => out.push((name.to_owned(), check(v)?)),
    }
    Ok(())
}

impl Limits {
    /// Reads the fields of a request; an empty text is a reset and a missing value no change.
    ///
    /// # Errors
    /// A sentence naming the field of the wrong type.
    pub(crate) fn parse(request: &Value) -> Result<Self, String> {
        Ok(Self {
            cwd: text_edit(request, "cwd")?,
            claude_exe: text_edit(request, "exePath")?,
            budget_usd: number_edit(request, "budgetUsd", Value::as_f64)?,
            restart_max: number_edit(request, "restartMax", Value::as_u64)?,
            approval_timeout_min: number_edit(request, "approvalTimeoutMin", Value::as_u64)?,
        })
    }

    /// Checks every field and names the `config.overlord` entries to write; a reset is a null.
    ///
    /// # Errors
    /// A sentence naming the first field out of range or missing on disk.
    pub(crate) fn fields(&self) -> Result<Vec<(String, Value)>, String> {
        let mut out = Vec::new();
        put(&mut out, "cwd", &self.cwd, |v| existing_folder(v))?;
        put(&mut out, "exePath", &self.claude_exe, |v| existing_file(v))?;
        if self.claude_exe != Edit::Keep {
            out.push(("claudeExe".to_owned(), Value::Null));
        }
        put(&mut out, "budgetUsd", &self.budget_usd, |v| sane_budget(*v))?;
        put(&mut out, "restartMax", &self.restart_max, |v| {
            sane_restarts(*v)
        })?;
        put(
            &mut out,
            "approvalTimeoutMin",
            &self.approval_timeout_min,
            |v| sane_wait(*v),
        )?;
        Ok(out)
    }
}

fn entry(value: &Value, source: &str) -> Value {
    json!({ "value": value, "source": source })
}

fn pick(set: bool) -> &'static str {
    if set { "set" } else { "default" }
}

/// Each setting with its value in effect and its source: `set` (`config.json`), `board` (the root
/// of the Overlord's board) or `default` (the dashboard home, the Desktop app's `claude`, no cap,
/// 3 restarts, 30 minutes).
pub(crate) fn body(svc: &Services) -> Value {
    let live = config::live(svc);
    let saved: Saved = config::saved(svc);
    let (folder, folder_source) = match (live.cwd, overlord_root(svc)) {
        (Some(path), _) => (path, "set"),
        (None, Some(path)) => (path, "board"),
        (None, None) => (svc.paths.home.clone(), "default"),
    };
    let exe = live.claude_exe.unwrap_or_else(find_exe);
    json!({
        "cwd": entry(&json!(folder.to_string_lossy()), folder_source),
        "exePath": entry(&json!(exe.to_string_lossy()), pick(saved.has(Field::ExePath))),
        "budgetUsd": entry(
            &json!(config::budget(svc).budget_usd),
            pick(saved.has(Field::Budget)),
        ),
        "restartMax": entry(&json!(live.restart_max), pick(saved.has(Field::RestartMax))),
        "approvalTimeoutMin": entry(
            &json!(live.approval_timeout_min),
            pick(saved.has(Field::ApprovalTimeout)),
        ),
    })
}
