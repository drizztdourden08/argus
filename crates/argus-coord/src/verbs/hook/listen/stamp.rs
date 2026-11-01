//! The listen stamp, `<dashboard home>/listen-<project>-<thread>.json`: the pid and start of
//! the latest listener of a thread, so a `PostToolUse` hook does not start a second one while a
//! fresh one lives. The stamp is an optimisation: without it every tool call starts a listener.
//!
//! `at` is what the freshness reads (Node reads `pid` and `at` only); `started` is when the
//! listener began, kept by [`keep`] so a listener is only carried past two minutes while it is
//! still inside its own deadline.

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

fn read(file: &Path) -> Option<Value> {
    serde_json::from_str(&fs::read_to_string(file).ok()?).ok()
}

/// The pid of a stamp when it names a live process that is not this one.
fn live_other(stamp: &Value) -> Option<u32> {
    stamp["pid"]
        .as_u64()
        .and_then(|p| u32::try_from(p).ok())
        .filter(|p| *p > 0 && *p != std::process::id() && pid_alive(*p))
}

/// Whether the stamp names another live process that started a listener in the last two
/// minutes.
pub fn fresh(file: &Path) -> bool {
    let Some(stamp) = read(file) else {
        return false;
    };
    let young = stamp["at"]
        .as_i64()
        .is_some_and(|at| now_ms() - at < FRESH_MS);
    young && live_other(&stamp).is_some()
}

fn save(file: &Path, stamp: &Value) {
    if let Some(dir) = file.parent() {
        fs::create_dir_all(dir).ok();
    }
    fs::write(file, stamp.to_string()).ok();
}

/// Writes this process as the thread's listener; a failure is ignored.
pub fn write(file: &Path) {
    let now = now_ms();
    let stamp = json!({ "pid": std::process::id(), "at": now, "started": now });
    save(file, &stamp);
}

/// Keeps a live listener fresh. A stamp that names another live process, is older than two
/// minutes and began less than `lives_ms` ago gets `at` set to now, so the `PostToolUse`
/// listener that follows skips instead of replacing a listener that is still waiting. A fresh
/// stamp, a dead pid, a listener past its own deadline or no stamp is left as it is. Returns
/// whether the stamp was refreshed.
pub fn keep(file: &Path, lives_ms: i64) -> bool {
    let Some(stamp) = read(file) else {
        return false;
    };
    let Some(pid) = live_other(&stamp) else {
        return false;
    };
    let now = now_ms();
    let Some(at) = stamp["at"].as_i64().filter(|at| now - at >= FRESH_MS) else {
        return false;
    };
    let started = stamp["started"].as_i64().unwrap_or(at);
    if now - started >= lives_ms {
        return false;
    }
    save(file, &json!({ "pid": pid, "at": now, "started": started }));
    true
}
