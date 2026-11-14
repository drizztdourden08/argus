//! `config.json` of the dashboard home: every Node key with its default, unknown keys kept.
//! A missing file is created with a fresh token; `installDir` is kept when the file has one (the
//! Node bin folder the coord CLI starts) and set to the binary's folder otherwise; `exe` is
//! rewritten on every load; `DASHBOARD_PORT` overrides the port of the loaded config without
//! being saved.

use std::collections::BTreeMap;
use std::fs;
use std::path::Path;

use serde::{Deserialize, Serialize};
use serde_json::{Map, Value, json};

use crate::Paths;

/// The default port of the Node server.
pub const DEFAULT_PORT: u16 = 47615;
/// The only host the server listens on.
pub const HOST: &str = "127.0.0.1";

/// A config that cannot be read or is refused.
#[derive(Debug, thiserror::Error)]
pub enum ConfigError {
    /// The file is not JSON, or a value has the wrong shape.
    #[error("cannot read {file}: {message}")]
    Unreadable {
        /// The file.
        file: String,
        /// Why.
        message: String,
    },
    /// A value is refused (host, port, `DASHBOARD_PORT`).
    #[error("{0}")]
    Refused(String),
    /// The file could not be written.
    #[error("cannot write config: {0}")]
    Io(#[from] std::io::Error),
}

/// `runs` limits.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RunsConfig {
    /// Runs at once.
    pub max_concurrent: u64,
    /// Log cap per run.
    pub log_cap_bytes: u64,
    /// Kept head of a log.
    pub log_head_bytes: u64,
    /// Runs kept per def.
    pub keep_per_def: u64,
    /// Days runs are kept.
    pub keep_days: u64,
    /// Grace before a forced stop.
    pub stop_grace_ms: u64,
    /// Keys Argus does not know.
    #[serde(flatten)]
    pub extra: Map<String, Value>,
}

/// `mcp` permissions.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct McpConfig {
    /// `"*"` or a list of tool names.
    pub allow: Value,
    /// Denied tools.
    pub deny: Vec<String>,
    /// Read tools only.
    pub read_only: bool,
    /// Keys Argus does not know.
    #[serde(flatten)]
    pub extra: Map<String, Value>,
}

/// `attach` limits.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AttachConfig {
    /// Largest image.
    pub max_bytes: u64,
    /// Extra folders a proof path may come from.
    pub extra_roots: Vec<String>,
    /// Keys Argus does not know.
    #[serde(flatten)]
    pub extra: Map<String, Value>,
}

/// The loaded config.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Config {
    /// Always 1.
    pub version: u64,
    /// Always `127.0.0.1`.
    pub host: String,
    /// The port.
    pub port: u16,
    /// 64 hex characters.
    pub token: String,
    /// The optional token hardening.
    pub require_token: bool,
    /// The folder of the running binary.
    pub install_dir: String,
    /// Project roots overriding the registry.
    pub project_roots: BTreeMap<String, String>,
    /// Run limits.
    pub runs: RunsConfig,
    /// MCP permissions.
    pub mcp: McpConfig,
    /// Attachment limits.
    pub attach: AttachConfig,
    /// The absolute path of the Argus binary (an Argus key).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub exe: Option<String>,
    /// Every other key, kept as it is (`agents`, `notify` and unknown ones).
    #[serde(flatten)]
    pub extra: Map<String, Value>,
}

impl Config {
    /// Whether cross-board dependencies are on (`deps.enabled`, true unless set to false).
    #[must_use]
    pub fn deps_enabled(&self) -> bool {
        self.extra
            .get("deps")
            .and_then(|d| d.get("enabled"))
            .and_then(Value::as_bool)
            .unwrap_or(true)
    }
}

fn defaults() -> Value {
    json!({
        "version": 1,
        "host": HOST,
        "port": DEFAULT_PORT,
        "token": "",
        "requireToken": false,
        "installDir": install_dir(),
        "projectRoots": {},
        "runs": { "maxConcurrent": 4, "logCapBytes": 2_097_152, "logHeadBytes": 262_144, "keepPerDef": 30, "keepDays": 14, "stopGraceMs": 5000 },
        "mcp": { "allow": "*", "deny": [], "readOnly": false },
        "attach": { "maxBytes": 10_485_760, "extraRoots": [] },
        "agents": { "stallMs": 180_000, "reapMs": 1_200_000 },
        "deps": { "enabled": true },
        "notify": { "os": true, "auto": { "handed": true, "ask": true }, "quietLevels": [], "sound": { "info": false, "ping": false, "urgent": true }, "toastInfo": false },
    })
}

/// The folder of the running binary: what `installDir` names.
#[must_use]
pub fn install_dir() -> String {
    std::env::current_exe()
        .ok()
        .and_then(|p| p.parent().map(Path::to_path_buf))
        .map(|p| p.display().to_string())
        .unwrap_or_default()
}

/// A fresh token: 32 random bytes as 64 hex characters.
#[must_use]
pub fn new_token() -> String {
    let mut buf = [0u8; 32];
    getrandom::fill(&mut buf).unwrap_or_default();
    hex(&buf)
}

/// Lowercase hex of some bytes.
#[must_use]
pub fn hex(bytes: &[u8]) -> String {
    const DIGITS: &[u8; 16] = b"0123456789abcdef";
    let mut out = String::with_capacity(bytes.len() * 2);
    for b in bytes {
        out.push(char::from(DIGITS[usize::from(b >> 4)]));
        out.push(char::from(DIGITS[usize::from(b & 15)]));
    }
    out
}

fn js_type(v: &Value) -> &'static str {
    match v {
        Value::Bool(_) => "boolean",
        Value::Number(_) => "number",
        Value::String(_) => "string",
        Value::Null | Value::Array(_) | Value::Object(_) => "object",
    }
}

fn wrong(v: Option<&Value>, default: &Value) -> bool {
    v.is_none_or(|v| v.is_null() || js_type(v) != js_type(default))
}

fn merge_sub(base: &Map<String, Value>, file: &Map<String, Value>) -> Map<String, Value> {
    let mut out = base.clone();
    for (k, v) in file {
        out.insert(k.clone(), v.clone());
    }
    for (k, v) in base {
        if !v.is_object() && wrong(out.get(k), v) {
            out.insert(k.clone(), v.clone());
        }
    }
    out
}

/// Defaults under the file's values, one level deep for the sub-objects, unknown keys kept.
fn merge(base: &Map<String, Value>, file: &Map<String, Value>) -> Map<String, Value> {
    let mut out = file.clone();
    for (k, v) in base {
        match (v, file.get(k)) {
            (Value::Object(b), Some(Value::Object(f))) => {
                out.insert(k.clone(), Value::Object(merge_sub(b, f)));
            }
            _ if wrong(out.get(k), v) => {
                out.insert(k.clone(), v.clone());
            }
            _ => {}
        }
    }
    out
}

fn read_file(file: &Path) -> Result<Option<Value>, ConfigError> {
    match fs::read_to_string(file) {
        Ok(text) => serde_json::from_str(&text)
            .map(Some)
            .map_err(|e| ConfigError::Unreadable {
                file: file.display().to_string(),
                message: e.to_string(),
            }),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(None),
        Err(e) => Err(ConfigError::Unreadable {
            file: file.display().to_string(),
            message: e.to_string(),
        }),
    }
}

/// `DASHBOARD_PORT` as Node reads it: absent or empty is no override.
///
/// # Errors
/// When the value is not an integer in 0..65535.
pub fn parse_env_port(raw: Option<&str>) -> Result<Option<u16>, ConfigError> {
    match raw {
        None | Some("") => Ok(None),
        Some(r) => r.parse::<u16>().map(Some).map_err(|_| {
            ConfigError::Refused(format!("DASHBOARD_PORT must be 0..65535, got \"{r}\""))
        }),
    }
}

/// `DASHBOARD_PORT` from the environment.
///
/// # Errors
/// When it is set to something that is not a port.
pub fn env_port() -> Result<Option<u16>, ConfigError> {
    parse_env_port(std::env::var("DASHBOARD_PORT").ok().as_deref())
}

fn check(c: &Map<String, Value>) -> Result<(), ConfigError> {
    if c.get("host").and_then(Value::as_str) != Some(HOST) {
        let got = c.get("host").map_or_else(String::new, crate::js::string);
        return Err(ConfigError::Refused(format!(
            "config.json host must be {HOST} (got \"{got}\"); the dashboard only listens on loopback"
        )));
    }
    let port = c.get("port").and_then(Value::as_u64);
    if port.is_none_or(|p| p > 65535) {
        let got = c.get("port").map_or_else(String::new, crate::js::string);
        return Err(ConfigError::Refused(format!(
            "config.json port is not a valid port: {got}"
        )));
    }
    Ok(())
}

fn valid_token(t: Option<&Value>) -> bool {
    t.and_then(Value::as_str).is_some_and(|t| {
        t.len() == 64
            && t.bytes()
                .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))
    })
}

fn typed(file: &Path, c: Map<String, Value>) -> Result<Config, ConfigError> {
    serde_json::from_value(Value::Object(c)).map_err(|e| ConfigError::Unreadable {
        file: file.display().to_string(),
        message: e.to_string(),
    })
}

/// Loads the config, creating it (and the token) when missing, keeping a given `installDir`
/// and rewriting `exe`. `env_port` (from `DASHBOARD_PORT`) overrides the returned port only.
///
/// # Errors
/// A file that is not JSON, a refused host or port, or a file that cannot be written.
pub fn load_config(paths: &Paths, env_port: Option<u16>) -> Result<Config, ConfigError> {
    let file = read_file(&paths.config)?
        .and_then(|v| v.as_object().cloned())
        .unwrap_or_default();
    let base = defaults().as_object().cloned().unwrap_or_default();
    let mut c = merge(&base, &file);
    check(&c)?;
    if !valid_token(c.get("token")) {
        c.insert("token".into(), json!(new_token()));
    }
    if let Ok(exe) = std::env::current_exe() {
        c.insert("exe".into(), json!(exe.display().to_string()));
    }
    if Value::Object(c.clone()) != Value::Object(file) {
        crate::fs::write_atomic(&paths.config, pretty(&Value::Object(c.clone())))?;
    }
    let mut config = typed(&paths.config, c)?;
    if let Some(p) = env_port {
        config.port = p;
    }
    Ok(config)
}

/// Saves the config. With a `DASHBOARD_PORT` override the file keeps its own port.
///
/// # Errors
/// When the file cannot be written.
pub fn save_config(
    paths: &Paths,
    config: &Config,
    env_port: Option<u16>,
) -> Result<(), ConfigError> {
    let mut out = config.clone();
    if env_port.is_some() {
        let disk = read_file(&paths.config)?;
        if let Some(p) = disk
            .as_ref()
            .and_then(|d| d.get("port"))
            .and_then(Value::as_u64)
        {
            out.port = u16::try_from(p).unwrap_or(out.port);
        }
    }
    let value = serde_json::to_value(&out).map_err(|e| ConfigError::Refused(e.to_string()))?;
    crate::fs::write_atomic(&paths.config, pretty(&value))?;
    Ok(())
}

/// Saves one `config.extra` key. The in-memory key is put back as it was when the file cannot be written.
///
/// # Errors
/// A [`StorageError`](crate::StorageError) when `config.json` cannot be saved.
pub fn save_extra_key(
    paths: &Paths,
    live: &mut Config,
    env_port: Option<u16>,
    key: &str,
    value: Value,
) -> Result<(), crate::StorageError> {
    let before = live.extra.insert(key.to_owned(), value);
    if let Err(error) = save_config(paths, live, env_port) {
        match before {
            Some(old) => live.extra.insert(key.to_owned(), old),
            None => live.extra.remove(key),
        };
        return Err(crate::StorageError(error.to_string()));
    }
    Ok(())
}

/// The file as it is, without creating anything (for `status` and `stop`).
#[must_use]
pub fn peek_config(paths: &Paths, env_port: Option<u16>) -> Option<Value> {
    let mut c = read_file(&paths.config).ok()??;
    if let (Some(p), Some(obj)) = (env_port, c.as_object_mut()) {
        obj.insert("port".into(), json!(p));
    }
    Some(c)
}

/// `JSON.stringify(v, null, 2)` plus a newline.
#[must_use]
pub fn pretty(v: &Value) -> String {
    let mut s = serde_json::to_string_pretty(v).unwrap_or_default();
    s.push('\n');
    s
}
