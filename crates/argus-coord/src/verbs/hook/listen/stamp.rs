//! The listen stamp, `<dashboard home>/listen-<project>-<thread>.json`: the pid and start of
//! the latest listener of a thread, so a `PostToolUse` hook does not start a second one while a
//! fresh one lives. The stamp is an optimisation: without it every tool call starts a listener.

use std::fs;
use std::path::{Path, PathBuf};

use argus_core::clock::now_ms;
use serde_json::{Value, json};

use super::super::alive::pid_alive;

/// A listener started within this is fresh.
const FRESH_MS: i64 = 120_000;

/// The stamp file of a thread; characters a file name should not hold become `_`.
pub fn file(home: &Path, project: &str, thread: &str) -> PathBuf {
    let mut name = String::new();
    for c in format!("listen-{project}-{thread}.json").chars() {
        if c.is_ascii_alphanumeric() || matches!(c, '.' | '_' | '-') {
            name.push(c);
        } else {
            name.extend(std::iter::repeat_n('_', c.len_utf16()));
        }
    }
    home.join(name)
}

/// Whether the stamp names another live process that started a listener in the last two
/// minutes.
pub fn fresh(file: &Path) -> bool {
    let Some(stamp) = fs::read_to_string(file)
        .ok()
        .and_then(|t| serde_json::from_str::<Value>(&t).ok())
    else {
        return false;
    };
    let pid = stamp["pid"]
        .as_u64()
        .and_then(|p| u32::try_from(p).ok())
        .filter(|p| *p > 0 && *p != std::process::id());
    let young = stamp["at"]
        .as_i64()
        .is_some_and(|at| now_ms() - at < FRESH_MS);
    pid.is_some_and(pid_alive) && young
}

/// Writes this process as the thread's listener; a failure is ignored.
pub fn write(file: &Path) {
    if let Some(dir) = file.parent() {
        fs::create_dir_all(dir).ok();
    }
    let stamp = json!({ "pid": std::process::id(), "at": now_ms() });
    fs::write(file, stamp.to_string()).ok();
}
