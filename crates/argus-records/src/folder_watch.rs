//! A watch on a record folder: any change under it calls `on_change` once the burst is over
//! (120 ms quiet, never more than 1 s after the first event). The file watcher can miss events,
//! so a quiet folder is also looked at again every 30 s.

use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::sync::mpsc::{RecvTimeoutError, Sender, channel};
use std::thread::JoinHandle;
use std::time::{Duration, Instant};

use notify::{RecommendedWatcher, RecursiveMode, Watcher as _};

const DEBOUNCE: Duration = Duration::from_millis(120);
const MAX_WAIT: Duration = Duration::from_secs(1);
const SAFETY: Duration = Duration::from_secs(30);

/// What the debounce thread is told.
enum Msg {
    Event,
    Stop,
}

/// The running watch; [`FolderWatch::stop`] ends it.
pub struct FolderWatch {
    root: PathBuf,
    watcher: Option<RecommendedWatcher>,
    tx: Sender<Msg>,
    thread: Option<JoinHandle<()>>,
}

fn open(root: &Path, tx: Sender<Msg>) -> Option<RecommendedWatcher> {
    let mut watcher = notify::recommended_watcher(move |res: notify::Result<notify::Event>| {
        if res.is_ok() {
            let _ = tx.send(Msg::Event);
        }
    })
    .ok()?;
    watcher.watch(root, RecursiveMode::Recursive).ok()?;
    Some(watcher)
}

/// Waits for the end of a burst; `false` when the watch was stopped meanwhile.
fn settle(rx: &std::sync::mpsc::Receiver<Msg>) -> bool {
    let first = Instant::now();
    loop {
        let left = MAX_WAIT.saturating_sub(first.elapsed()).min(DEBOUNCE);
        match rx.recv_timeout(left) {
            Ok(Msg::Event) if first.elapsed() < MAX_WAIT => {}
            Ok(Msg::Event) | Err(RecvTimeoutError::Timeout) => return true,
            Ok(Msg::Stop) | Err(RecvTimeoutError::Disconnected) => return false,
        }
    }
}

fn pump(rx: &std::sync::mpsc::Receiver<Msg>, on_change: &(dyn Fn() + Send + Sync)) {
    loop {
        match rx.recv_timeout(SAFETY) {
            Ok(Msg::Event) => {
                if !settle(rx) {
                    return;
                }
                on_change();
            }
            Err(RecvTimeoutError::Timeout) => on_change(),
            Ok(Msg::Stop) | Err(RecvTimeoutError::Disconnected) => return,
        }
    }
}

impl FolderWatch {
    /// Starts watching `root` (created when missing). `on_change` runs on the watch's own thread.
    #[must_use]
    pub fn start(root: &Path, on_change: Arc<dyn Fn() + Send + Sync>) -> FolderWatch {
        let _ = std::fs::create_dir_all(root);
        let (tx, rx) = channel();
        let watcher = open(root, tx.clone());
        let thread = std::thread::spawn(move || pump(&rx, on_change.as_ref()));
        FolderWatch {
            root: root.to_path_buf(),
            watcher,
            tx,
            thread: Some(thread),
        }
    }

    /// Ends the thread and lets the file watcher go of the folder before returning: on Windows
    /// the watcher stops in its own thread after the drop, and a folder removed while that is
    /// under way can deadlock the process. The unwatch is queued, and the configure round trip
    /// after it answers only once the unwatch is done.
    pub fn stop(mut self) {
        let _ = self.tx.send(Msg::Stop);
        if let Some(mut watcher) = self.watcher.take() {
            let _ = watcher.unwatch(&self.root);
            let _ = watcher.configure(notify::Config::default());
        }
        if let Some(thread) = self.thread.take() {
            let _ = thread.join();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicUsize, Ordering};

    #[test]
    fn a_file_appearing_calls_back_once_after_the_burst() {
        let dir = tempfile::tempdir().expect("temp dir");
        let calls = Arc::new(AtomicUsize::new(0));
        let seen = Arc::clone(&calls);
        let watch = FolderWatch::start(
            dir.path(),
            Arc::new(move || {
                seen.fetch_add(1, Ordering::SeqCst);
            }),
        );
        for i in 0..5 {
            std::fs::write(dir.path().join(format!("f{i}")), "x").expect("write");
        }
        let until = Instant::now() + Duration::from_secs(10);
        while calls.load(Ordering::SeqCst) == 0 && Instant::now() < until {
            std::thread::sleep(Duration::from_millis(20));
        }
        watch.stop();
        let n = calls.load(Ordering::SeqCst);
        assert!((1..=2).contains(&n), "{n} calls for one burst");
    }
}
