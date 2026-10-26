//! How `argus mcp` behaves, from its flag and the two knobs the tests of the Node bridge used:
//! `COORD_DASHBOARD_NO_BOOT_LAUNCH` keeps it from starting the server when it starts, and
//! `COORD_DASHBOARD_START_WAIT_MS` shortens the wait for a start.

use std::time::Duration;

use argus_mcp::BridgeOptions;

const NO_BOOT_LAUNCH: &str = "COORD_DASHBOARD_NO_BOOT_LAUNCH";
const START_WAIT_MS: &str = "COORD_DASHBOARD_START_WAIT_MS";

/// The options for `--project <project>` and the environment `var` reads.
#[must_use]
pub fn options(project: Option<String>, var: &dyn Fn(&str) -> Option<String>) -> BridgeOptions {
    let mut opts = BridgeOptions {
        project,
        ..BridgeOptions::default()
    };
    opts.boot_launch = var(NO_BOOT_LAUNCH).is_none_or(|v| v.is_empty());
    let wait = var(START_WAIT_MS).and_then(|v| v.trim().parse::<u64>().ok());
    if let Some(ms) = wait.filter(|ms| *ms > 0) {
        opts.start_wait = Duration::from_millis(ms);
    }
    opts
}
