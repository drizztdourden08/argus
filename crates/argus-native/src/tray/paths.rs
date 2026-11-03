//! The files the tray leaves in the dashboard home, readable without the server and the tray.
// test-tag: critical

use std::path::{Path, PathBuf};

const FNV_OFFSET: u64 = 0xcbf2_9ce4_8422_2325;
const FNV_PRIME: u64 = 0x0100_0000_01b3;

/// The tray files of one dashboard home.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TrayFiles {
    home: PathBuf,
}

impl TrayFiles {
    /// The files of the dashboard home `home`.
    #[must_use]
    pub fn new(home: impl Into<PathBuf>) -> Self {
        Self { home: home.into() }
    }

    /// The dashboard home itself.
    #[must_use]
    pub fn home(&self) -> &Path {
        &self.home
    }

    /// `tray.json`: who holds the icon.
    #[must_use]
    pub fn tray_json(&self) -> PathBuf {
        self.home.join("tray.json")
    }

    /// `quit.flag`: the owner chose Quit in the tray menu.
    #[must_use]
    pub fn quit_flag(&self) -> PathBuf {
        self.home.join("quit.flag")
    }

    /// `tray.stop`: a request to close the icon only.
    #[must_use]
    pub fn stop_file(&self) -> PathBuf {
        self.home.join("tray.stop")
    }

    /// `tray.quit`: a request that does what the Quit item does.
    #[must_use]
    pub fn quit_request(&self) -> PathBuf {
        self.home.join("tray.quit")
    }

    /// `tray-notify.json`: a notification the server could not show as a toast.
    #[must_use]
    pub fn balloon_file(&self) -> PathBuf {
        self.home.join("tray-notify.json")
    }

    /// The folder of the logs, `logs`.
    #[must_use]
    pub fn log_dir(&self) -> PathBuf {
        self.home.join("logs")
    }

    /// `logs/tray.log`.
    #[must_use]
    pub fn log_file(&self) -> PathBuf {
        self.log_dir().join("tray.log")
    }

    /// The name of the single-instance mutex: one per dashboard home, whatever the letter case of
    /// its path.
    #[must_use]
    pub fn instance_name(&self) -> String {
        let lower = self.home.to_string_lossy().to_lowercase();
        let hash = lower.bytes().fold(FNV_OFFSET, |h, b| {
            (h ^ u64::from(b)).wrapping_mul(FNV_PRIME)
        });
        format!(r"Local\argus-tray-{hash:016x}")
    }
}

#[cfg(test)]
mod tests {
    use super::TrayFiles;

    #[test]
    fn the_files_sit_in_the_home_under_the_names_of_the_protocol() {
        let files = TrayFiles::new("/h");
        let names: Vec<String> = [
            files.tray_json(),
            files.quit_flag(),
            files.stop_file(),
            files.quit_request(),
            files.balloon_file(),
            files.log_file(),
        ]
        .iter()
        .map(|p| {
            let rest = p.strip_prefix("/h").unwrap();
            rest.to_string_lossy().replace('\\', "/")
        })
        .collect();
        assert_eq!(
            names,
            [
                "tray.json",
                "quit.flag",
                "tray.stop",
                "tray.quit",
                "tray-notify.json",
                "logs/tray.log"
            ]
        );
    }

    #[test]
    fn the_mutex_name_is_per_home_and_ignores_letter_case() {
        let a = TrayFiles::new(r"C:\Users\Me\.coord\dashboard").instance_name();
        let b = TrayFiles::new(r"c:\users\me\.coord\dashboard").instance_name();
        let c = TrayFiles::new(r"C:\Users\Me\.coord\other").instance_name();
        assert!(a.starts_with(r"Local\argus-tray-") && a.len() == 17 + 16);
        assert_eq!(a, b);
        assert_ne!(a, c);
    }
}
