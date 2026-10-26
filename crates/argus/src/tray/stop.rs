//! `argus tray stop`: closes the icon only; the server keeps running.

use std::time::Duration;

use argus_core::Paths;
use argus_native::{STOP_GRACE, Stopped, TrayFiles, process_alive, stop_tray};

use argus::detach;

const STEP: Duration = Duration::from_millis(100);

/// Asks the tray to close, and ends one that does not within the grace.
///
/// # Errors
///
/// The request could not be written.
pub fn stop(paths: &Paths) -> Result<u8, String> {
    let files = TrayFiles::new(&paths.home);
    let pause = || std::thread::sleep(STEP);
    let stopped = stop_tray(&files, process_alive, detach::kill, pause, STOP_GRACE)
        .map_err(|e| format!("the tray could not be asked to close: {e}"))?;
    match stopped {
        Stopped::NotRunning => println!("tray is not running"),
        Stopped::Closed { pid, forced } => {
            let how = if forced { ", forced" } else { "" };
            println!("tray stopped (pid {pid}{how}); the server keeps running");
        }
    }
    Ok(0)
}
