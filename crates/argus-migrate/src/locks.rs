//! The board locks: every `<folder>/board.lock` is taken for the whole run and refreshed every
//! 5 s, so the Node dashboard and an old binary wait (a lock older than 15 s counts as a crash).

use std::fs::{self, OpenOptions};
use std::io;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread::{self, JoinHandle, sleep};
use std::time::{Duration, Instant, SystemTime};

use argus_core::ids::PORTFOLIO;

use crate::error::{Result, stop};

const WAIT: Duration = Duration::from_secs(20);
const STALE: Duration = Duration::from_secs(15);
const REFRESH: Duration = Duration::from_secs(5);
const SLICE: Duration = Duration::from_millis(100);

/// The locks of a run; dropping it stops the refresh and removes them.
pub struct Held {
    all: Arc<Vec<PathBuf>>,
    stop: Arc<AtomicBool>,
    refresher: Option<JoinHandle<()>>,
}

fn is_stale(path: &Path) -> bool {
    fs::metadata(path)
        .and_then(|m| m.modified())
        .map_or(true, |t| {
            SystemTime::now().duration_since(t).unwrap_or_default() > STALE
        })
}

fn acquire(path: &Path) -> Result<()> {
    let start = Instant::now();
    loop {
        match OpenOptions::new().write(true).create_new(true).open(path) {
            Ok(_) => return Ok(()),
            Err(e)
                if matches!(
                    e.kind(),
                    io::ErrorKind::AlreadyExists | io::ErrorKind::PermissionDenied
                ) =>
            {
                if is_stale(path) {
                    fs::remove_file(path).ok();
                } else if start.elapsed() > WAIT {
                    return stop(format!("{} stays locked by a live writer", path.display()));
                } else {
                    sleep(Duration::from_millis(50));
                }
            }
            Err(e) => return Err(e.into()),
        }
    }
}

fn touch(path: &Path) {
    let now = SystemTime::now();
    if let Ok(file) = OpenOptions::new().write(true).open(path) {
        file.set_modified(now).ok();
    }
}

fn refresh_loop(all: &Arc<Vec<PathBuf>>, halt: &Arc<AtomicBool>) {
    let mut waited = Duration::ZERO;
    while !halt.load(Ordering::Relaxed) {
        sleep(SLICE);
        waited += SLICE;
        if waited >= REFRESH {
            waited = Duration::ZERO;
            all.iter().for_each(|p| touch(p));
        }
    }
}

fn lock_file(home: &Path, folder: &str) -> PathBuf {
    home.join(folder).join("board.lock")
}

/// Takes the lock of every board in `pairs` (the folder named by the first or by the second
/// name, whichever exists) and of `_portfolio`.
pub fn take(home: &Path, pairs: &[(String, String)]) -> Result<Held> {
    let mut names: Vec<(String, String)> = pairs.to_vec();
    names.push((PORTFOLIO.to_owned(), PORTFOLIO.to_owned()));
    let mut got: Vec<PathBuf> = Vec::new();
    let mut all: Vec<PathBuf> = Vec::new();
    for (first, second) in &names {
        all.push(lock_file(home, first));
        all.push(lock_file(home, second));
        let folder = if home.join(first).is_dir() { first } else { second };
        if !home.join(folder).is_dir() {
            continue;
        }
        let file = lock_file(home, folder);
        if let Err(e) = acquire(&file) {
            got.iter().for_each(|p| {
                fs::remove_file(p).ok();
            });
            return Err(e);
        }
        got.push(file);
    }
    let all = Arc::new(all);
    let halt = Arc::new(AtomicBool::new(false));
    let (a, h) = (Arc::clone(&all), Arc::clone(&halt));
    let refresher = Some(thread::spawn(move || refresh_loop(&a, &h)));
    Ok(Held {
        all,
        stop: halt,
        refresher,
    })
}

impl Drop for Held {
    fn drop(&mut self) {
        self.stop.store(true, Ordering::Relaxed);
        if let Some(handle) = self.refresher.take() {
            handle.join().ok();
        }
        self.all.iter().for_each(|p| {
            fs::remove_file(p).ok();
        });
    }
}
