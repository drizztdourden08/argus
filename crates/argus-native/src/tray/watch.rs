//! What other processes ask of the running tray through files: close the icon (`tray.stop`), do
//! what Quit does (`tray.quit`), show a balloon (`tray-notify.json`). The tray looks every tick.

use std::fs;

use super::balloon::{self, Balloon};
use super::paths::TrayFiles;

/// One request found on disk.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Request {
    /// Close the icon only; the server keeps running.
    Stop,
    /// Do what the Quit menu item does.
    Quit,
    /// Show a balloon tip.
    Balloon(Balloon),
}

/// Takes the requests that wait. A stop request wins over everything else.
#[must_use]
pub fn poll(files: &TrayFiles) -> Vec<Request> {
    if fs::remove_file(files.stop_file()).is_ok() {
        return vec![Request::Stop];
    }
    let quit = fs::remove_file(files.quit_request()).is_ok();
    let balloon = balloon::take(files);
    quit.then_some(Request::Quit)
        .into_iter()
        .chain(balloon.map(Request::Balloon))
        .collect()
}

/// Removes the requests a previous tray left behind.
pub fn clear_requests(files: &TrayFiles) {
    let _stop = fs::remove_file(files.stop_file());
    let _quit = fs::remove_file(files.quit_request());
}

#[cfg(test)]
mod tests {
    use tempfile::TempDir;

    use super::{Request, clear_requests, poll};
    use crate::tray::balloon::{self, Balloon};
    use crate::tray::paths::TrayFiles;

    fn with_stop_and_quit() -> (TempDir, TrayFiles) {
        let dir = tempfile::tempdir().unwrap();
        let files = TrayFiles::new(dir.path());
        std::fs::write(files.stop_file(), "x").unwrap();
        std::fs::write(files.quit_request(), "quit").unwrap();
        (dir, files)
    }

    #[test]
    fn nothing_waiting_is_nothing_to_do() {
        let dir = tempfile::tempdir().unwrap();
        assert_eq!(poll(&TrayFiles::new(dir.path())), Vec::new());
    }

    #[test]
    fn a_stop_request_wins_and_is_consumed() {
        let (_dir, files) = with_stop_and_quit();
        assert_eq!(poll(&files), [Request::Stop]);
        assert!(!files.stop_file().exists());
        assert_eq!(
            poll(&files),
            [Request::Quit],
            "the quit request waits for the next look"
        );
    }

    #[test]
    fn a_quit_request_and_a_balloon_arrive_together_and_are_consumed() {
        let dir = tempfile::tempdir().unwrap();
        let files = TrayFiles::new(dir.path());
        std::fs::write(files.quit_request(), "quit").unwrap();
        balloon::write(&files, &Balloon::new("T", "x")).unwrap();
        assert_eq!(
            poll(&files),
            [Request::Quit, Request::Balloon(Balloon::new("T", "x"))]
        );
        assert_eq!(poll(&files), Vec::new());
    }

    #[test]
    fn a_new_tray_clears_the_requests_of_an_old_one() {
        let (_dir, files) = with_stop_and_quit();
        clear_requests(&files);
        assert_eq!(poll(&files), Vec::new());
    }
}
