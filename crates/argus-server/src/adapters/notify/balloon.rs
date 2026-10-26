//! The tray's balloon tip, tried when a toast cannot be shown.

use argus_native::{Balloon, TrayFiles, read_live_tray, write_balloon};
use argus_notify::TrayFallback;

use crate::adapters::procs::alive;

/// Leaves a balloon request for the tray of this dashboard home, when a tray runs.
pub struct TrayBalloon {
    files: TrayFiles,
}

impl TrayBalloon {
    /// The balloon of the tray that holds the icon for `files`.
    #[must_use]
    pub fn new(files: TrayFiles) -> Self {
        TrayBalloon { files }
    }
}

impl TrayFallback for TrayBalloon {
    fn balloon(&self, title: &str, text: &str) -> bool {
        read_live_tray(&self.files, alive).is_some()
            && write_balloon(&self.files, &Balloon::new(title, text)).is_ok()
    }
}
