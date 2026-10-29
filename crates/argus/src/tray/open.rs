//! What the menu opens besides a page (which `argus_native::open_url` opens): the folder of the
//! logs.

use std::io;
use std::path::Path;

use argus::detach;

/// Opens a folder in the file explorer.
///
/// # Errors
///
/// The explorer could not be started.
pub fn folder(path: &Path) -> io::Result<()> {
    detach::windowed("explorer.exe", &[path.as_os_str()])
}
