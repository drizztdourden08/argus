//! Where the server answers, read from `config.json` on every call so a server that restarts on
//! another port is found again.

use std::fs;
use std::path::Path;

use argus_mcp::Endpoint;
use serde_json::Value;

/// The loopback endpoint `config.json` names, or none when the file is missing, unreadable or
/// has no port. The token is carried only when the config asks for it.
#[must_use]
pub fn read(config: &Path) -> Option<Endpoint> {
    let text = fs::read_to_string(config).ok()?;
    let config: Value = serde_json::from_str(&text).ok()?;
    let port = u16::try_from(config["port"].as_u64()?)
        .ok()
        .filter(|p| *p != 0)?;
    let require = config["requireToken"] == true;
    Some(Endpoint::loopback(port, config["token"].as_str(), require))
}
