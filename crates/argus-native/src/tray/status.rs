//! `argus tray status`: whether the tray runs and whether the owner closed the dashboard from it.

use super::paths::TrayFiles;
use super::quit_flag::{self, Boot};
use super::record::read_live;

/// The state of the tray as `argus tray status` reports it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TrayStatus {
    /// True when a live process holds the icon.
    pub running: bool,
    /// Its process id.
    pub pid: Option<u32>,
    /// When it started, as ISO text.
    pub started_at: Option<String>,
    /// True when `quit.flag` says the owner quit from the tray during this boot.
    pub quit: bool,
    /// The tooltip the live icon shows, when the tray wrote it.
    pub tooltip: Option<String>,
}

/// Reads the state of the tray.
#[must_use]
pub fn status(files: &TrayFiles, alive: impl Fn(u32) -> bool, boot: Boot) -> TrayStatus {
    let live = read_live(files, alive);
    TrayStatus {
        running: live.is_some(),
        pid: live.as_ref().map(|record| record.pid),
        started_at: live.as_ref().and_then(|record| record.started_at.clone()),
        tooltip: live.and_then(|record| record.tooltip),
        quit: quit_flag::is_active(files, boot),
    }
}

/// The one line `argus tray status` prints.
#[must_use]
pub fn status_text(status: &TrayStatus) -> String {
    match (status.pid, status.running, status.quit) {
        (Some(pid), true, _) => format!(
            "tray running (pid {pid}, since {})",
            status.started_at.as_deref().unwrap_or("?")
        ),
        (_, _, true) => {
            "tray not running (Argus was closed from the tray; `argus tray start` reopens it)"
                .to_owned()
        }
        _ => "tray not running".to_owned(),
    }
}

/// The second line of `argus tray status`: the icon is registered and what its tooltip says.
/// None when no live tray wrote a tooltip.
#[must_use]
pub fn icon_line(status: &TrayStatus) -> Option<String> {
    let tooltip = status.tooltip.as_deref().filter(|_| status.running)?;
    Some(format!("icon registered: \"{tooltip}\""))
}

#[cfg(test)]
mod tests {
    use super::{icon_line, status, status_text};
    use crate::tray::paths::TrayFiles;
    use crate::tray::quit_flag::{self, Boot};
    use crate::tray::record;

    const BOOT: Boot = Boot {
        now_ms: 0,
        uptime_ms: 0,
    };

    #[test]
    fn no_tray_no_flag_is_plainly_not_running() {
        let dir = tempfile::tempdir().unwrap();
        let files = TrayFiles::new(dir.path());
        let state = status(&files, |_| false, BOOT);
        assert!(!state.running && !state.quit && state.pid.is_none());
        assert_eq!(status_text(&state), "tray not running");
    }

    #[test]
    fn a_quit_flag_is_reported_and_explained() {
        let dir = tempfile::tempdir().unwrap();
        let files = TrayFiles::new(dir.path());
        quit_flag::write(&files).unwrap();
        let state = status(&files, |_| false, BOOT);
        assert!(state.quit);
        assert!(status_text(&state).contains("closed from the tray"));
    }

    #[test]
    fn a_live_tray_reports_its_pid_and_start() {
        let dir = tempfile::tempdir().unwrap();
        let files = TrayFiles::new(dir.path());
        record::write(&files, 12).unwrap();
        let state = status(&files, |pid| pid == 12, BOOT);
        assert!(state.running);
        assert_eq!(state.pid, Some(12));
        assert!(status_text(&state).starts_with("tray running (pid 12, since 20"));
        assert_eq!(icon_line(&state), None, "no tooltip written yet");
        record::set_tooltip(&files, 12, "Argus (shadow): stopped").unwrap();
        let state = status(&files, |pid| pid == 12, BOOT);
        assert_eq!(
            icon_line(&state).as_deref(),
            Some("icon registered: \"Argus (shadow): stopped\"")
        );
    }
}
