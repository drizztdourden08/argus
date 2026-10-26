//! `argus mcp [--project <key>]` (DESIGN 7.1): the stdio MCP server Claude spawns. It answers
//! `initialize` and `tools/list` itself, forwards everything else to the running server's
//! `/mcp`, and brings the server up when it is down, unless the owner quit it from the tray.
//! The protocol is `argus-mcp`; this module is the machine it runs on.

mod endpoint;
mod env;
mod launch;
mod options;

use argus_core::Paths;
use argus_mcp::run_bridge;
use tokio::io::{BufReader, stdin, stdout};

pub use env::{Clock, DashboardEnv, Launch};
pub use options::options;

/// Runs the bridge on stdin and stdout until stdin ends.
///
/// # Errors
///
/// Stdin could not be read.
pub async fn run(paths: Paths, project: Option<String>) -> Result<u8, String> {
    let var = |name: &str| std::env::var(name).ok();
    let env = DashboardEnv::new(paths);
    run_bridge(
        env,
        options(project, &var),
        BufReader::new(stdin()),
        stdout(),
    )
    .await
    .map_err(|e| e.to_string())?;
    Ok(0)
}
