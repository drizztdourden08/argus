//! The client of a dashboard that may be down: it answers at once when the server is healthy,
//! else it starts the server (hidden, detached) and waits up to 8 seconds
//! (`COORD_DASHBOARD_WAIT_MS`) for it. A server that will not come up is
//! [`ClientError::Down`], the error the `run` verbs turn into exit 3.
//!
//! The server is started as the Node CLI does while the Node dashboard serves: `node
//! <installDir>/bin/dashboard.mjs start`. After the switch-over only this module changes.

use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::thread::sleep;
use std::time::{Duration, Instant};

use argus_core::Paths;
use serde_json::Value;

use super::config::dashboard_config;
use super::health::healthy;
use super::http::{Client, ClientError};
use crate::cli::env::Env;

const WAIT_DEFAULT_MS: u64 = 8000;
const POLL: Duration = Duration::from_millis(200);

fn bin_of(install_dir: &Path) -> PathBuf {
    install_dir.join("bin").join("dashboard.mjs")
}

/// Where `bin/dashboard.mjs` lives when `config.json` does not say yet (the first ever start):
/// `COORD_DASHBOARD_INSTALL`, else the install the CLI config names (`aiConfigDir/dashboard`).
fn install_dir_guess(env: &Env) -> Option<PathBuf> {
    if let Some(dir) = env.get("COORD_DASHBOARD_INSTALL") {
        return Some(PathBuf::from(dir));
    }
    let file = env.home_dir().join(".rotp-cli").join("config.json");
    let cli: Value = serde_json::from_str(&fs::read_to_string(file).ok()?).ok()?;
    let dir = Path::new(cli["aiConfigDir"].as_str()?).join("dashboard");
    bin_of(&dir).exists().then_some(dir)
}

#[cfg(windows)]
fn detach(command: &mut Command) {
    use std::os::windows::process::CommandExt;
    const DETACHED_PROCESS: u32 = 0x0000_0008;
    const CREATE_NO_WINDOW: u32 = 0x0800_0000;
    command.creation_flags(DETACHED_PROCESS | CREATE_NO_WINDOW);
}

#[cfg(not(windows))]
fn detach(_command: &mut Command) {}

/// `node <bin> start`, hidden and detached so it outlives this call.
fn start_server(env: &Env, bin: &Path) {
    let mut command = Command::new("node");
    command
        .arg(bin)
        .arg("start")
        .envs(env.iter())
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null());
    detach(&mut command);
    command.spawn().ok();
}

fn ready(paths: &Paths) -> Option<Client> {
    let client = Client::new(&dashboard_config(paths)?);
    healthy(&client).then_some(client)
}

fn wait_ms(env: &Env) -> u64 {
    env.get("COORD_DASHBOARD_WAIT_MS")
        .and_then(|v| v.trim().parse::<u64>().ok())
        .filter(|ms| *ms > 0)
        .unwrap_or(WAIT_DEFAULT_MS)
}

/// A client of the running dashboard. With `autostart`, a server that does not answer is
/// started and waited for.
///
/// # Errors
/// [`ClientError::Down`] when it is not running and may not be started (`autostart` off, no
/// install found) or does not come up in time.
pub fn dashboard_client(env: &Env, paths: &Paths, autostart: bool) -> Result<Client, ClientError> {
    if let Some(client) = ready(paths) {
        return Ok(client);
    }
    if !autostart {
        return Err(ClientError::down(None));
    }
    let install = env
        .get("COORD_DASHBOARD_INSTALL")
        .map(PathBuf::from)
        .or_else(|| {
            dashboard_config(paths)
                .and_then(|c| c.install_dir)
                .map(PathBuf::from)
        })
        .or_else(|| install_dir_guess(env));
    let Some(bin) = install.map(|d| bin_of(&d)).filter(|b| b.exists()) else {
        return Err(ClientError::down(Some("no dashboard install found")));
    };
    start_server(env, &bin);
    let deadline = Instant::now() + Duration::from_millis(wait_ms(env));
    while Instant::now() < deadline {
        sleep(POLL);
        if let Some(client) = ready(paths) {
            return Ok(client);
        }
    }
    Err(ClientError::down(None))
}
