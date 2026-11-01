//! `quit.flag`: written only by the tray's Quit, so a process that finds the server down knows the
//! owner closed it on purpose. A flag older than the current boot is from a previous session.
// test-tag: critical

use std::fs;
use std::io;
use std::time::SystemTime;

use argus_core::fs::write_atomic;

use super::paths::TrayFiles;
use super::timestamp::{epoch_ms, iso_utc};

/// The slack that covers clock rounding between the file system and the uptime counter.
pub const BOOT_SLACK_MS: u64 = 5000;

/// The clock readings that place the boot of the machine in time.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Boot {
    /// Milliseconds since the epoch.
    pub now_ms: u64,
    /// Milliseconds the machine has been up.
    pub uptime_ms: u64,
}

impl Boot {
    /// The boot time, in milliseconds since the epoch.
    #[must_use]
    pub const fn time_ms(self) -> u64 {
        self.now_ms.saturating_sub(self.uptime_ms)
    }
}

/// True when the owner quit the dashboard from the tray during this boot.
#[must_use]
pub fn is_active(files: &TrayFiles, boot: Boot) -> bool {
    let written = fs::metadata(files.quit_flag()).and_then(|meta| meta.modified());
    written.is_ok_and(|time| epoch_ms(time) >= boot.time_ms().saturating_sub(BOOT_SLACK_MS))
}

/// Writes `quit.flag` as `{"at":"<iso time>","pid":<pid>}`.
///
/// # Errors
///
/// The home cannot be created or the file cannot be written.
pub fn write(files: &TrayFiles) -> io::Result<()> {
    let at = iso_utc(SystemTime::now());
    let pid = std::process::id();
    write_atomic(
        &files.quit_flag(),
        format!("{{\"at\":\"{at}\",\"pid\":{pid}}}\n"),
    )
}

/// Removes `quit.flag`; a missing file is fine.
///
/// # Errors
///
/// The file exists and cannot be removed.
pub fn clear(files: &TrayFiles) -> io::Result<()> {
    match fs::remove_file(files.quit_flag()) {
        Err(error) if error.kind() != io::ErrorKind::NotFound => Err(error),
        _ => Ok(()),
    }
}

#[cfg(test)]
mod tests {
    use std::fs::File;
    use std::time::{Duration, UNIX_EPOCH};

    use super::{Boot, clear, is_active, write};
    use crate::tray::paths::TrayFiles;

    const NOON: u64 = 1_792_670_400_000;

    fn set_written(files: &TrayFiles, ms: u64) {
        let file = File::options().write(true).open(files.quit_flag()).unwrap();
        file.set_modified(UNIX_EPOCH + Duration::from_millis(ms))
            .unwrap();
    }

    #[test]
    fn a_flag_written_since_boot_counts_and_one_from_before_it_is_ignored() {
        let dir = tempfile::tempdir().unwrap();
        let files = TrayFiles::new(dir.path());
        let boot = Boot {
            now_ms: NOON,
            uptime_ms: 3_600_000,
        };
        assert_eq!(boot.time_ms(), NOON - 3_600_000);
        assert!(!is_active(&files, boot), "no file");
        write(&files).unwrap();
        set_written(&files, NOON - 1_800_000);
        assert!(is_active(&files, boot), "written after the boot");
        set_written(&files, NOON - 3_660_000);
        assert!(!is_active(&files, boot), "written before the boot");
        set_written(&files, NOON - 3_603_000);
        assert!(is_active(&files, boot), "a few seconds of clock slack");
    }

    #[test]
    fn the_flag_holds_the_time_and_the_pid_and_clears_twice() {
        let dir = tempfile::tempdir().unwrap();
        let files = TrayFiles::new(dir.path().join("fresh"));
        write(&files).unwrap();
        let text = std::fs::read_to_string(files.quit_flag()).unwrap();
        let json: serde_json::Value = serde_json::from_str(&text).unwrap();
        assert_eq!(json["pid"], std::process::id());
        assert!(json["at"].as_str().unwrap().ends_with('Z'));
        clear(&files).unwrap();
        assert!(!files.quit_flag().exists());
        clear(&files).unwrap();
    }
}
