//! `argus tray status`: whether the icon runs, and whether the owner closed the dashboard from it.

use argus_core::Paths;
use argus_native::{Boot, TrayFiles, process_alive, status_text, tray_status};

/// Prints the one line and answers 0 for a running tray, 1 for none.
pub fn status(paths: &Paths) -> u8 {
    let files = TrayFiles::new(&paths.home);
    let state = tray_status(&files, process_alive, Boot::current());
    println!("{}", status_text(&state));
    u8::from(!state.running)
}
