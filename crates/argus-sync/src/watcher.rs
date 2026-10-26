//! Notices board changes under `COORD_HOME`. The file watcher is fast but can miss events, so a
//! 10 s stat poll backs it up and finds new projects. Each key is debounced: 120 ms after the
//! last event, never later than 1 s after the first, so a burst of CLI calls exports once.

use std::collections::HashMap;
use std::path::{Component, Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex, PoisonError};
use std::time::Duration;

use argus_core::ids::is_scope;
use notify::{RecommendedWatcher, RecursiveMode, Watcher as _};
use tokio::sync::mpsc::{UnboundedReceiver, UnboundedSender, unbounded_channel};
use tokio::task::JoinHandle;
use tokio::time::Instant;

use crate::state::State;

const POLL: Duration = Duration::from_secs(10);
const DEBOUNCE: Duration = Duration::from_millis(120);
const MAX_WAIT: Duration = Duration::from_secs(1);

/// A folder under `COORD_HOME` that can hold a board.
pub(crate) fn is_board_key(key: &str) -> bool {
    is_scope(key) && key != "dashboard"
}

fn key_of(root: &Path, path: &Path) -> Option<String> {
    let rel = path.strip_prefix(root).ok()?;
    let parts: Vec<&std::ffi::OsStr> = rel
        .components()
        .filter_map(|c| {
            if let Component::Normal(s) = c {
                Some(s)
            } else {
                None
            }
        })
        .collect();
    let [key, file] = parts.as_slice() else {
        return None;
    };
    let key = key.to_str()?;
    (file.to_str() == Some("board.json") && is_board_key(key)).then(|| key.to_owned())
}

/// The running watcher; stopping it ends its tasks.
pub(crate) struct Watcher {
    tasks: Vec<JoinHandle<()>>,
    fs: Arc<Mutex<Option<RecommendedWatcher>>>,
    ok: Arc<AtomicBool>,
}

impl Watcher {
    pub fn ok(&self) -> bool {
        self.ok.load(Ordering::SeqCst)
    }

    pub fn stop(self) {
        for t in &self.tasks {
            t.abort();
        }
        self.fs
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .take();
    }
}

struct Shared {
    root: PathBuf,
    state: Arc<State>,
    tx: UnboundedSender<String>,
    fs: Arc<Mutex<Option<RecommendedWatcher>>>,
    ok: Arc<AtomicBool>,
}

impl Shared {
    fn start_fs(&self) {
        let mut slot = self.fs.lock().unwrap_or_else(PoisonError::into_inner);
        if slot.is_some() {
            return;
        }
        let (root, tx, state, ok) = (
            self.root.clone(),
            self.tx.clone(),
            Arc::clone(&self.state),
            Arc::clone(&self.ok),
        );
        let made =
            notify::recommended_watcher(move |res: notify::Result<notify::Event>| match res {
                Ok(event) => {
                    for key in event.paths.iter().filter_map(|p| key_of(&root, p)) {
                        state.noticed();
                        let _ = tx.send(key);
                    }
                }
                Err(_) => ok.store(false, Ordering::SeqCst),
            });
        let Ok(mut w) = made else {
            return self.ok.store(false, Ordering::SeqCst);
        };
        let watching = w.watch(&self.root, RecursiveMode::Recursive).is_ok();
        self.ok.store(watching, Ordering::SeqCst);
        if watching {
            *slot = Some(w);
        }
    }

    fn signatures(&self) -> HashMap<String, (u128, u64)> {
        let Ok(dirs) = std::fs::read_dir(&self.root) else {
            return HashMap::new();
        };
        dirs.filter_map(Result::ok)
            .filter_map(|d| {
                let key = d.file_name().to_str()?.to_owned();
                if !is_board_key(&key) {
                    return None;
                }
                let meta = std::fs::metadata(d.path().join("board.json")).ok()?;
                let mtime = meta
                    .modified()
                    .ok()?
                    .duration_since(std::time::UNIX_EPOCH)
                    .ok()?
                    .as_nanos();
                Some((key, (mtime, meta.len())))
            })
            .collect()
    }
}

async fn poll_loop(shared: Arc<Shared>) {
    let mut seen = shared.signatures();
    loop {
        tokio::time::sleep(POLL).await;
        if !shared.ok.load(Ordering::SeqCst) {
            shared
                .fs
                .lock()
                .unwrap_or_else(PoisonError::into_inner)
                .take();
        }
        shared.start_fs();
        let now = shared.signatures();
        for (key, sig) in &now {
            if seen.get(key) != Some(sig) {
                shared.state.noticed();
                let _ = shared.tx.send(key.clone());
            }
        }
        seen = now;
    }
}

async fn debounce_loop(state: Arc<State>, mut rx: UnboundedReceiver<String>) {
    let mut pending: HashMap<String, (Instant, Instant)> = HashMap::new();
    loop {
        let next = pending.values().map(|(_, due)| *due).min();
        tokio::select! {
            got = rx.recv() => {
                let Some(key) = got else { return };
                let now = Instant::now();
                let first = pending.get(&key).map_or(now, |(f, _)| *f);
                pending.insert(key, (first, (now + DEBOUNCE).min(first + MAX_WAIT)));
            }
            () = sleep_until(next) => {
                let now = Instant::now();
                let due: Vec<String> = pending.iter().filter(|(_, (_, d))| *d <= now).map(|(k, _)| k.clone()).collect();
                for key in due {
                    pending.remove(&key);
                    state.export_key(&key);
                }
            }
        }
    }
}

async fn sleep_until(at: Option<Instant>) {
    match at {
        Some(t) => tokio::time::sleep_until(t).await,
        None => std::future::pending().await,
    }
}

/// Starts watching `state`'s coord home.
pub(crate) fn start(state: &Arc<State>) -> Watcher {
    let (tx, rx) = unbounded_channel();
    let fs = Arc::new(Mutex::new(None));
    let ok = Arc::new(AtomicBool::new(false));
    let shared = Arc::new(Shared {
        root: state.coord_home().to_path_buf(),
        state: Arc::clone(state),
        tx,
        fs: Arc::clone(&fs),
        ok: Arc::clone(&ok),
    });
    shared.start_fs();
    let tasks = vec![
        tokio::spawn(debounce_loop(Arc::clone(state), rx)),
        tokio::spawn(poll_loop(shared)),
    ];
    Watcher { tasks, fs, ok }
}
