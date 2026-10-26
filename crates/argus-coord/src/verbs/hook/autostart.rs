//! The dashboard is up whenever a session is: the session-start and prompt hooks (and a
//! listener whose stream broke) start it when no live server holds its pid file, hidden and
//! detached, without waiting. It stays down when the owner chose Quit in the tray during this
//! boot (`quit.flag`), a start begun in the last minute is not repeated (`start.lock`),
//! `COORD_DASHBOARD_NO_AUTOSTART` turns it off, and a session pointed at another home
//! (`COORD_HOME` or `DASHBOARD_HOME`) never starts the owner's server unless
//! `COORD_DASHBOARD_AUTOSTART=1`.
//!
//! While the Node dashboard serves, the start is the Node one: `node
//! <installDir>/bin/dashboard.mjs tray start` on Windows, `... start` elsewhere.

use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::time::{Duration, SystemTime};

use argus_core::Paths;
use argus_core::clock::now_iso;
use serde_json::Value;

use super::alive::{pid_alive, quit_this_boot};
use crate::cli::Env;
use crate::client::dashboard_config;

const LOCK_FRESH: Duration = Duration::from_secs(60);

/// `DETACHED_PROCESS | CREATE_NO_WINDOW`: no console, and not tied to this process.
#[cfg(windows)]
const HIDDEN: u32 = 0x0800_0008;

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

/// The install the CLI config of this machine names (`aiConfigDir/dashboard`).
fn installed(env: &Env) -> Option<PathBuf> {
    let cli = read_json(&env.home_dir().join(".rotp-cli").join("config.json"))?;
    Some(Path::new(cli["aiConfigDir"].as_str()?).join("dashboard"))
}

/// `bin/dashboard.mjs` of the install: `COORD_DASHBOARD_INSTALL`, else `config.json`'s
/// `installDir`, else the install of the CLI config; `None` when it is not there.
fn dashboard_bin(env: &Env, paths: &Paths) -> Option<PathBuf> {
    let install = env
        .get("COORD_DASHBOARD_INSTALL")
        .map(PathBuf::from)
        .or_else(|| dashboard_config(paths)?.install_dir.map(PathBuf::from))
        .or_else(|| installed(env))?;
    Some(install.join("bin").join("dashboard.mjs")).filter(|bin| bin.exists())
}

/// `node <bin> tray start` (`start` off Windows), with no window and not tied to this process.
fn start(env: &Env, bin: &Path) {
    argus_native::keep_stdio_private();
    let verbs: &[&str] = if cfg!(windows) {
        &["tray", "start"]
    } else {
        &["start"]
    };
    let mut command = Command::new("node");
    command.arg(bin).args(verbs).envs(env.iter());
    command
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null());
    #[cfg(windows)]
    std::os::windows::process::CommandExt::creation_flags(&mut command, HIDDEN);
    command.spawn().ok();
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
    let Some(bin) = dashboard_bin(env, paths) else {
        return;
    };
    if fs::write(paths.home.join("start.lock"), format!("{}\n", now_iso())).is_ok() {
        start(env, &bin);
    }
}
