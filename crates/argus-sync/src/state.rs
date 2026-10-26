//! What the parts of the sync service share: the options, the last good board of each key, the
//! current cards and the watcher's health counters.

use std::collections::HashMap;
use std::path::Path;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::{Mutex, PoisonError, RwLock};

use argus_board::Board;
use argus_core::Config;
use serde_json::{Value, json};

use crate::SyncOptions;

/// Empties `dirty` and `versions`: nothing reads them any more, so every save by the server
/// clears what the CLI keeps adding.
pub(crate) fn clear_marks(b: &mut Board) {
    if let Some(o) = b.as_object_mut() {
        o.insert("dirty".into(), json!({}));
        o.insert("versions".into(), json!({}));
    }
}

/// The shared state.
pub(crate) struct State {
    pub opts: SyncOptions,
    pub boards: RwLock<HashMap<String, Board>>,
    pub cards: RwLock<Vec<Value>>,
    pub errors: AtomicU64,
    pub last_event_at: Mutex<Option<String>>,
    pub stopped: AtomicBool,
    pub portfolio_pending: AtomicBool,
}

impl State {
    pub fn new(opts: SyncOptions) -> Self {
        State {
            opts,
            boards: RwLock::new(HashMap::new()),
            cards: RwLock::new(Vec::new()),
            errors: AtomicU64::new(0),
            last_event_at: Mutex::new(None),
            stopped: AtomicBool::new(true),
            portfolio_pending: AtomicBool::new(false),
        }
    }

    pub fn coord_home(&self) -> &Path {
        &self.opts.paths.coord_home
    }

    pub fn stopped(&self) -> bool {
        self.stopped.load(Ordering::SeqCst)
    }

    pub fn config(&self) -> Config {
        self.opts
            .config
            .read()
            .unwrap_or_else(PoisonError::into_inner)
            .clone()
    }

    pub fn board(&self, key: &str) -> Option<Board> {
        self.boards
            .read()
            .unwrap_or_else(PoisonError::into_inner)
            .get(key)
            .cloned()
    }

    pub fn keep_board(&self, key: &str, board: Board) {
        self.boards
            .write()
            .unwrap_or_else(PoisonError::into_inner)
            .insert(key.to_owned(), board);
    }

    pub fn set_cards(&self, cards: Vec<Value>) {
        *self.cards.write().unwrap_or_else(PoisonError::into_inner) = cards;
    }

    pub fn noticed(&self) {
        *self
            .last_event_at
            .lock()
            .unwrap_or_else(PoisonError::into_inner) = Some(argus_core::clock::now_iso());
    }

    pub fn warn(&self, msg: &str, key: &str, err: &dyn std::fmt::Display) {
        self.opts.log.warn(
            msg,
            json!({ "key": key, "err": { "message": err.to_string() } }),
        );
    }
}
