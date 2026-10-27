//! What the CLI knows about the dashboard server from its files: the port (the server's own
//! `server.pid` first, then `config.json`, then the default), the token and the install folder.

use std::fs;
use std::path::Path;

use argus_core::Paths;
use serde_json::Value;

use crate::cli::env::Env;

/// The port of the Node dashboard.
pub const DEFAULT_PORT: u16 = 47615;

/// The server as `config.json` and `server.pid` describe it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DashboardConfig {
    /// The port the server really listens on.
    pub port: u16,
    /// The bearer token, only when `config.json` says `requireToken`: a loopback caller needs
    /// no credential otherwise.
    pub token: Option<String>,
    /// The folder of the dashboard install, when `config.json` names it.
    pub install_dir: Option<String>,
}

pub(crate) fn read_json(file: &Path) -> Option<Value> {
    serde_json::from_str(
        fs::read_to_string(file)
            .ok()?
            .trim_start_matches('\u{feff}'),
    )
    .ok()
}

/// A positive integer port (`Number.isInteger(v) && v > 0`).
#[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
fn valid_port(v: &Value) -> Option<u16> {
    let n = v
        .as_f64()
        .filter(|n| n.fract() == 0.0 && *n > 0.0 && *n <= f64::from(u16::MAX))?;
    Some(n as u16)
}

fn text_of(v: &Value) -> Option<String> {
    v.as_str().filter(|s| !s.is_empty()).map(str::to_owned)
}

/// The dashboard config: `None` when `config.json` is missing or unreadable.
#[must_use]
pub fn dashboard_config(paths: &Paths) -> Option<DashboardConfig> {
    let config = read_json(&paths.config)?;
    let pid = read_json(&paths.pid_file);
    let port = pid
        .as_ref()
        .and_then(|p| valid_port(&p["port"]))
        .or_else(|| valid_port(&config["port"]))
        .unwrap_or(DEFAULT_PORT);
    let token = (config["requireToken"] == true)
        .then(|| text_of(&config["token"]))
        .flatten();
    Some(DashboardConfig {
        port,
        token,
        install_dir: text_of(&config["installDir"]),
    })
}

/// The port the hooks use: the running server's `server.pid`, else `config.json`, else the
/// default; the files are optional.
#[must_use]
pub fn server_port(paths: &Paths) -> u16 {
    read_json(&paths.pid_file)
        .and_then(|p| valid_port(&p["port"]))
        .or_else(|| read_json(&paths.config).and_then(|c| valid_port(&c["port"])))
        .unwrap_or(DEFAULT_PORT)
}

/// `Number(v)` for a port that is a number or a numeric text, else `None`.
fn number_port(v: &Value) -> Option<u16> {
    let n = match v {
        Value::Number(n) => n.as_f64()?,
        Value::String(s) => s.trim().parse::<f64>().ok()?,
        _ => return None,
    };
    valid_port(&Value::from(n))
}

/// The port the board verbs print in urls and shortcuts: `DASHBOARD_PORT`, overridden by the
/// port of `config.json` when it ran once, else the default.
#[must_use]
pub fn board_port(env: &Env, paths: &Paths) -> u16 {
    let from_env = env
        .get("DASHBOARD_PORT")
        .and_then(|p| number_port(&Value::from(p)))
        .unwrap_or(DEFAULT_PORT);
    read_json(&paths.config)
        .and_then(|c| number_port(&c["port"]))
        .unwrap_or(from_env)
}
