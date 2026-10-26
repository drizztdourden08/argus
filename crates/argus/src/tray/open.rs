//! What the menu opens: a page of the dashboard in the browser, the folder of the logs.

use std::ffi::OsStr;
use std::io;
use std::path::Path;

use argus::detach;

/// Opens an address in the default browser, with no window of its own.
///
/// # Errors
///
/// The opener could not be started.
pub fn url(address: &str) -> io::Result<()> {
    let args = ["url.dll,FileProtocolHandler", address].map(OsStr::new);
    detach::hidden("rundll32", &args)
}

/// Opens a folder in the file explorer.
///
/// # Errors
///
/// The explorer could not be started.
pub fn folder(path: &Path) -> io::Result<()> {
    detach::windowed("explorer.exe", &[path.as_os_str()])
}
