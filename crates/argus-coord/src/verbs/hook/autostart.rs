//! The dashboard is up whenever a session is: the session-start and prompt hooks (and a
//! listener whose stream broke) start it when no live server holds its pid file, hidden and
//! detached, without waiting. It stays down when the owner chose Quit in the tray during this
//! boot (`quit.flag`), a start begun in the last minute is not repeated (`start.lock`),
//! `COORD_DASHBOARD_NO_AUTOSTART` turns it off, and a session pointed at another home
//! (`COORD_HOME` or `DASHBOARD_HOME`) never starts the owner's server unless
//! `COORD_DASHBOARD_AUTOSTART=1`.
//!
//! What is started, and how (the tray on Windows, the Node bin or the Argus binary of a shadow
//! home), is decided by [`crate::client::launch`].

use std::fs;
use std::path::Path;
use std::time::{Duration, SystemTime};

use argus_core::Paths;
use argus_core::clock::now_iso;
use serde_json::Value;

use super::alive::{pid_alive, quit_this_boot};
use crate::cli::Env;
use crate::client::launch::{plan, spawn};

const LOCK_FRESH: Duration = Duration::from_secs(60);

fn read_json(file: &Path) -> Option<Value> {
    serde_json::from_str(&fs::read_to_string(file).ok()?).ok()
}

fn server_alive(home: &Path) -> bool {
    read_json(&home.join("server.pid"))
        .and_then(|p| p["pid"].as_u64())
        .and_then(|pid| u32::try_from(pid).ok())
        .is_some_and(|pid| pid > 0 && pid_alive(pid))
}

fn lock_fresh(lock: &Path) -> bool {
    fs::metadata(lock)
        .and_then(|m| m.modified())
        .is_ok_and(|at| SystemTime::now().duration_since(at).unwrap_or_default() < LOCK_FRESH)
}

fn held_back(env: &Env, home: &Path) -> bool {
    let other_home = env.get("COORD_HOME").is_some() || env.get("DASHBOARD_HOME").is_some();
    env.get("COORD_DASHBOARD_NO_AUTOSTART").is_some()
        || (other_home && env.get("COORD_DASHBOARD_AUTOSTART") != Some("1"))
        || server_alive(home)
        || quit_this_boot(home)
        || lock_fresh(&home.join("start.lock"))
}

/// Starts the dashboard when it is down and may be started. Never fails.
pub fn ensure_dashboard(env: &Env, paths: &Paths) {
    if held_back(env, &paths.home) {
        return;
    }
    let Some(plan) = plan(env, paths) else {
        return;
    };
    if fs::write(paths.home.join("start.lock"), format!("{}\n", now_iso())).is_ok() {
        spawn(env, &plan);
    }
}
