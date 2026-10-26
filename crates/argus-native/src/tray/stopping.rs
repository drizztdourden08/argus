//! `argus tray stop`: close the icon only. The tray sees `tray.stop` within a second and exits;
//! one that ignores it is killed. The server is never touched.

use std::fs;
use std::io;
use std::time::{Duration, Instant};

use super::paths::TrayFiles;
use super::record::read_live;
use super::timestamp::iso_utc;

/// How long a tray gets to close itself before it is killed.
pub const GRACE: Duration = Duration::from_secs(6);

/// What a stop did.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Stopped {
    /// No live tray held the icon.
    NotRunning,
    /// A tray was closed.
    Closed {
        /// The process id of that tray.
        pid: u32,
        /// True when it ignored the request and was killed.
        forced: bool,
    },
}

/// Asks the running tray to close by writing `tray.stop`.
///
/// # Errors
///
/// The file cannot be written.
pub fn request(files: &TrayFiles) -> io::Result<()> {
    let at = iso_utc(std::time::SystemTime::now());
    fs::write(files.stop_file(), format!("{at}\n"))
}

/// Closes the tray: asks through `tray.stop`, calls `pause` while the tray is alive and within
/// `grace`, then `kill` when it is still alive. Removes `tray.stop` and `tray.json` after.
///
/// # Errors
///
/// `tray.stop` cannot be written.
pub fn stop_tray(
    files: &TrayFiles,
    alive: impl Fn(u32) -> bool,
    mut kill: impl FnMut(u32),
    mut pause: impl FnMut(),
    grace: Duration,
) -> io::Result<Stopped> {
    let Some(live) = read_live(files, &alive) else {
        let _stale = fs::remove_file(files.tray_json());
        return Ok(Stopped::NotRunning);
    };
    request(files)?;
    let until = Instant::now() + grace;
    while Instant::now() < until && alive(live.pid) {
        pause();
    }
    let forced = alive(live.pid);
    if forced {
        kill(live.pid);
        pause();
    }
    let _gone = fs::remove_file(files.stop_file());
    let _gone = fs::remove_file(files.tray_json());
    Ok(Stopped::Closed {
        pid: live.pid,
        forced,
    })
}

#[cfg(test)]
mod tests {
    use std::cell::Cell;
    use std::time::Duration;

    use super::{Stopped, stop_tray};
    use crate::tray::paths::TrayFiles;
    use crate::tray::record;

    #[test]
    fn a_tray_that_leaves_when_asked_is_not_killed() {
        let dir = tempfile::tempdir().unwrap();
        let files = TrayFiles::new(dir.path());
        record::write(&files, 31337).unwrap();
        let gone = Cell::new(false);
        let mut killed = false;
        let stopped = stop_tray(
            &files,
            |pid| pid == 31337 && !gone.get(),
            |_| killed = true,
            || {
                assert!(
                    files.stop_file().exists(),
                    "the request is on disk while it waits"
                );
                gone.set(true);
            },
            Duration::from_secs(5),
        )
        .unwrap();
        assert_eq!(
            stopped,
            Stopped::Closed {
                pid: 31337,
                forced: false
            }
        );
        assert!(!killed);
        assert!(!files.stop_file().exists());
        assert!(!files.tray_json().exists());
    }

    #[test]
    fn a_tray_that_ignores_the_request_is_killed() {
        let dir = tempfile::tempdir().unwrap();
        let files = TrayFiles::new(dir.path());
        record::write(&files, 31338).unwrap();
        let dead = Cell::new(false);
        let mut kills = Vec::new();
        let stopped = stop_tray(
            &files,
            |_| !dead.get(),
            |pid| {
                kills.push(pid);
                dead.set(true);
            },
            || {},
            Duration::from_millis(1),
        )
        .unwrap();
        assert_eq!(
            stopped,
            Stopped::Closed {
                pid: 31338,
                forced: true
            }
        );
        assert_eq!(kills, [31338]);
    }

    #[test]
    fn no_tray_is_a_no_op_that_clears_a_stale_record() {
        let dir = tempfile::tempdir().unwrap();
        let files = TrayFiles::new(dir.path());
        record::write(&files, 5).unwrap();
        let stopped = stop_tray(&files, |_| false, |_| {}, || {}, Duration::ZERO).unwrap();
        assert_eq!(stopped, Stopped::NotRunning);
        assert!(!files.tray_json().exists());
    }
}
