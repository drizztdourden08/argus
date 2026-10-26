//! The two machine facts the bridge needs that a test replaces: how to start the server, and
//! when this boot began.

use std::path::Path;

use argus_native::Boot;

/// Starts `argus serve` for the dashboard home, hidden and apart from the bridge, without
/// waiting for it. A second server of the same home refuses to start, so asking twice is safe.
///
/// # Errors
///
/// The program could not be started, or this is not Windows.
#[cfg(windows)]
pub fn serve_hidden(home: &Path) -> Result<(), String> {
    use std::ffi::OsStr;

    let args = [
        OsStr::new("serve"),
        OsStr::new("--data-dir"),
        home.as_os_str(),
    ];
    crate::detach::argus(&args).map_err(|e| format!("the server could not be started: {e}"))
}

/// Starts `argus serve` for the dashboard home.
///
/// # Errors
///
/// Always: starting the server in the background is built for Windows only.
#[cfg(not(windows))]
pub fn serve_hidden(_home: &Path) -> Result<(), String> {
    Err("the server can only be started in the background on Windows; run `argus serve`".to_owned())
}

/// The clock readings that place this machine's boot, when they are known.
#[cfg(windows)]
#[must_use]
#[allow(clippy::unnecessary_wraps)]
pub fn boot_now() -> Option<Boot> {
    Some(Boot::current())
}

/// The clock readings that place this machine's boot, when they are known.
#[cfg(not(windows))]
#[must_use]
pub fn boot_now() -> Option<Boot> {
    None
}
