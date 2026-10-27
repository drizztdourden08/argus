//! The two machine facts the bridge needs that a test replaces: how to start the dashboard, and
//! when this boot began.

use std::ffi::OsString;
use std::path::Path;

use argus_native::Boot;

/// The command line that brings the dashboard up for the bridge: `argus tray start` of the
/// home, as the Node bridge runs `dashboard tray start`, so the tray icon is there whenever the
/// server runs. A shadow home adds `--shadow`.
#[must_use]
pub fn start_plan(home: &Path) -> Vec<OsString> {
    argus_core::launch::tray_start(home)
}

/// Starts the tray (and with it the server) for the dashboard home, hidden and apart from the
/// bridge, without waiting for it. A second start of the same home finds the first and does
/// nothing, so asking twice is safe.
///
/// # Errors
///
/// The program could not be started, or this is not Windows.
#[cfg(windows)]
pub fn start_hidden(home: &Path) -> Result<(), String> {
    crate::detach::argus(&start_plan(home))
        .map_err(|e| format!("the dashboard could not be started: {e}"))
}

/// Starts the dashboard for the dashboard home.
///
/// # Errors
///
/// Always: starting the dashboard in the background is built for Windows only.
#[cfg(not(windows))]
pub fn start_hidden(_home: &Path) -> Result<(), String> {
    Err(
        "the dashboard can only be started in the background on Windows; run `argus serve`"
            .to_owned(),
    )
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

#[cfg(test)]
mod tests {
    use std::path::Path;

    use super::start_plan;

    fn words(home: &Path) -> Vec<String> {
        start_plan(home)
            .iter()
            .map(|a| a.to_string_lossy().into_owned())
            .collect()
    }

    #[test]
    fn the_bridge_starts_the_tray_of_its_home_and_a_shadow_home_says_so() {
        let dir = tempfile::tempdir().unwrap();
        let home = dir.path().join("h");
        std::fs::create_dir_all(&home).unwrap();
        let h = home.to_string_lossy().into_owned();
        assert_eq!(words(&home), ["tray", "start", "--data-dir", h.as_str()]);
        std::fs::write(home.join("config.json"), r#"{"shadow":true}"#).unwrap();
        assert_eq!(
            words(&home),
            ["tray", "start", "--data-dir", h.as_str(), "--shadow"]
        );
    }
}
