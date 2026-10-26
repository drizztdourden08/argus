//! What the tray process says goes to `logs/tray.log`; it has no console.

use std::fs::{self, OpenOptions};
use std::io::Write;

use argus_core::clock::now_iso;
use argus_native::TrayFiles;

/// Adds a line. A log that cannot be written is not worth stopping the tray for.
pub fn line(files: &TrayFiles, text: &str) {
    let _made = fs::create_dir_all(files.log_dir());
    let file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(files.log_file());
    if let Ok(mut file) = file {
        let _written = writeln!(file, "{} {text}", now_iso());
    }
}
