//! The tray as `argus status` reports it: nothing off Windows, else whether the icon runs and
//! whether the owner closed the dashboard from it.

use argus_core::Paths;
use serde_json::Value;
#[cfg(windows)]
use serde_json::json;

/// The status facts of the tray, `null` off Windows.
#[cfg(windows)]
#[must_use]
pub fn info(paths: &Paths) -> Value {
    use argus_native::{Boot, TrayFiles, process_alive, tray_status};

    let status = tray_status(&TrayFiles::new(&paths.home), process_alive, Boot::current());
    json!({ "running": status.running, "pid": status.pid, "quit": status.quit })
}

/// The status facts of the tray, `null` off Windows.
#[cfg(not(windows))]
#[must_use]
pub fn info(_paths: &Paths) -> Value {
    Value::Null
}

/// The line of the text status for the tray facts.
#[must_use]
pub fn line(info: &Value) -> String {
    if info["running"] == true {
        return format!("  tray: running (pid {})", info["pid"]);
    }
    if info["quit"] == true {
        return "  tray: not running (the dashboard was closed from the tray; `argus tray start` reopens it)".to_owned();
    }
    "  tray: not running (`argus tray start` shows the icon)".to_owned()
}
