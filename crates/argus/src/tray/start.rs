//! `argus tray start`: the tray in the background, and with it the server.

use std::ffi::OsStr;
use std::time::Duration;

use argus_core::Paths;
use argus_native::{Launch, TrayFiles, prepare_launch, process_alive, wait_for_tray};

use super::detach;

const WAIT: Duration = Duration::from_secs(10);
const STEP: Duration = Duration::from_millis(150);

/// Starts the tray unless one runs. Starting it is how the owner reopens a dashboard they quit,
/// so `quit.flag` is cleared first.
///
/// # Errors
///
/// The flag could not be cleared, the tray could not be started or did not show up in time.
pub fn start(paths: &Paths) -> Result<u8, String> {
    let files = TrayFiles::new(&paths.home);
    let launch = prepare_launch(&files, process_alive).map_err(|e| e.to_string())?;
    if let Launch::Already { pid } = launch {
        println!("tray already running (pid {pid})");
        return Ok(0);
    }
    let home = paths.home.as_os_str();
    detach::argus(&[
        OsStr::new("tray"),
        OsStr::new("run"),
        OsStr::new("--data-dir"),
        home,
    ])
    .map_err(|e| format!("the tray could not be started: {e}"))?;
    let pause = || std::thread::sleep(STEP);
    match wait_for_tray(&files, process_alive, WAIT, pause) {
        Some(tray) => {
            println!("tray started (pid {})", tray.pid);
            Ok(0)
        }
        None => Err("the tray did not start in time".to_owned()),
    }
}
