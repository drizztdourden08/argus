//! Starting the tray from another process: the checks `argus tray start` makes before it spawns
//! the tray, and the wait for the tray to show up.

use std::fs;
use std::io;
use std::time::{Duration, Instant};

use super::paths::TrayFiles;
use super::quit_flag;
use super::record::{TrayRecord, read_live};

/// What `argus tray start` does next.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Launch {
    /// A live tray holds the icon already; start nothing.
    Already {
        /// The process id of that tray.
        pid: u32,
    },
    /// Spawn the tray.
    Proceed,
}

/// Prepares a start. Starting the tray is how the owner reopens a dashboard they quit, so
/// `quit.flag` is cleared first. A live `tray.json` means no second tray; otherwise a leftover
/// `tray.stop` is removed so the new tray does not close itself.
///
/// # Errors
///
/// `quit.flag` or `tray.stop` exists and cannot be removed.
pub fn prepare(files: &TrayFiles, alive: impl Fn(u32) -> bool) -> io::Result<Launch> {
    quit_flag::clear(files)?;
    if let Some(live) = read_live(files, alive) {
        return Ok(Launch::Already { pid: live.pid });
    }
    match fs::remove_file(files.stop_file()) {
        Err(error) if error.kind() != io::ErrorKind::NotFound => Err(error),
        _ => Ok(Launch::Proceed),
    }
}

/// Waits for the `tray.json` of a live tray, calling `pause` between looks. `None` when `timeout`
/// passes first.
pub fn wait_for_tray(
    files: &TrayFiles,
    alive: impl Fn(u32) -> bool,
    timeout: Duration,
    mut pause: impl FnMut(),
) -> Option<TrayRecord> {
    let until = Instant::now() + timeout;
    loop {
        if let Some(record) = read_live(files, &alive) {
            return Some(record);
        }
        if Instant::now() >= until {
            return None;
        }
        pause();
    }
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use super::{Launch, prepare, wait_for_tray};
    use crate::tray::paths::TrayFiles;
    use crate::tray::{quit_flag, record};

    #[test]
    fn a_live_tray_means_no_second_start_and_a_dead_one_does_not_count() {
        let dir = tempfile::tempdir().unwrap();
        let files = TrayFiles::new(dir.path());
        record::write(&files, 777).unwrap();
        assert_eq!(
            prepare(&files, |pid| pid == 777).unwrap(),
            Launch::Already { pid: 777 }
        );
        assert_eq!(prepare(&files, |_| false).unwrap(), Launch::Proceed);
    }

    #[test]
    fn starting_clears_quit_flag_and_a_leftover_stop_request() {
        let dir = tempfile::tempdir().unwrap();
        let files = TrayFiles::new(dir.path());
        quit_flag::write(&files).unwrap();
        std::fs::write(files.stop_file(), "x").unwrap();
        assert_eq!(prepare(&files, |_| false).unwrap(), Launch::Proceed);
        assert!(!files.quit_flag().exists());
        assert!(!files.stop_file().exists());
    }

    #[test]
    fn a_live_tray_still_clears_quit_flag() {
        let dir = tempfile::tempdir().unwrap();
        let files = TrayFiles::new(dir.path());
        record::write(&files, 9).unwrap();
        quit_flag::write(&files).unwrap();
        prepare(&files, |_| true).unwrap();
        assert!(!files.quit_flag().exists());
    }

    #[test]
    fn waiting_gives_up_when_tray_json_never_appears() {
        let dir = tempfile::tempdir().unwrap();
        let files = TrayFiles::new(dir.path());
        let mut pauses = 0;
        let found = wait_for_tray(
            &files,
            |_| true,
            Duration::from_millis(20),
            || {
                pauses += 1;
                std::thread::sleep(Duration::from_millis(10));
            },
        );
        assert!(found.is_none());
        assert!(pauses >= 1);
    }

    #[test]
    fn waiting_returns_the_record_that_appears() {
        let dir = tempfile::tempdir().unwrap();
        let files = TrayFiles::new(dir.path());
        let found = wait_for_tray(
            &files,
            |pid| pid == 4242,
            Duration::from_secs(5),
            || {
                record::write(&files, 4242).unwrap();
            },
        );
        assert_eq!(found.unwrap().pid, 4242);
    }
}
