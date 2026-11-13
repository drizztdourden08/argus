//! Step 1: the migration never stops or starts the server, it refuses to run while it is up.

use std::fs;

use serde_json::Value;

use crate::error::{Result, stop};
use crate::options::Options;

fn server_pid(opts: &Options) -> Option<u32> {
    let text = fs::read_to_string(opts.dashboard().join("server.pid")).ok()?;
    let value: Value = serde_json::from_str(&text).ok()?;
    u32::try_from(value["pid"].as_u64()?).ok()
}

/// Refuses when `<home>/dashboard/server.pid` names a live process.
pub fn server_down(opts: &Options) -> Result<()> {
    match server_pid(opts) {
        Some(pid) if (opts.pid_alive)(pid) => stop(format!(
            "the Argus server is running (pid {pid}): stop it (argus stop), run the migration, then start it again"
        )),
        _ => Ok(()),
    }
}
