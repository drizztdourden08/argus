//! The boards as the wake logic reads them: [`BoardView`]s parsed once per export and kept, so
//! the many reads of a registry pass cost no parse. Also what the wake logic asks the live view
//! of sub-agents.

use std::collections::HashMap;
use std::sync::{Arc, PoisonError, RwLock};

use argus_agents::AgentWatcher;
use argus_board::Board;
use argus_wake::board::BoardView;
use argus_wake::source::{AgentLive, BoardSource};
use serde::Deserialize;

use crate::boards::BoardReader;

/// The project boards, as [`BoardView`]s.
pub struct WakeBoards {
    reader: BoardReader,
    kept: RwLock<HashMap<String, Arc<BoardView>>>,
}

/// A board as the wake logic sees it.
#[must_use]
pub fn view_of(board: &Board) -> Arc<BoardView> {
    Arc::new(BoardView::deserialize(board).unwrap_or_default())
}

impl WakeBoards {
    /// Boards read through `reader`, with a kept copy per key.
    #[must_use]
    pub fn new(reader: BoardReader) -> Self {
        WakeBoards {
            reader,
            kept: RwLock::new(HashMap::new()),
        }
    }

    /// Reads the board of a key again, as just exported.
    pub fn refresh(&self, key: &str) {
        let Some(board) = self.reader.board(key) else {
            return;
        };
        self.kept
            .write()
            .unwrap_or_else(PoisonError::into_inner)
            .insert(key.to_owned(), view_of(&board));
    }
}

impl BoardSource for WakeBoards {
    fn scopes(&self) -> Vec<String> {
        self.reader
            .projects()
            .into_iter()
            .map(|p| p.key)
            .filter(|key| self.board(key).is_some())
            .collect()
    }

    fn board(&self, scope: &str) -> Option<Arc<BoardView>> {
        let kept = self.kept.read().unwrap_or_else(PoisonError::into_inner);
        if let Some(view) = kept.get(scope) {
            return Some(Arc::clone(view));
        }
        drop(kept);
        let view = view_of(&self.reader.board(scope)?);
        self.kept
            .write()
            .unwrap_or_else(PoisonError::into_inner)
            .insert(scope.to_owned(), Arc::clone(&view));
        Some(view)
    }
}

/// The live view of sub-agents, when there is one.
pub struct WatcherLive {
    watcher: Option<Arc<AgentWatcher>>,
}

impl WatcherLive {
    /// Asks `watcher` (nothing, when it is off).
    #[must_use]
    pub fn new(watcher: Option<Arc<AgentWatcher>>) -> Self {
        WatcherLive { watcher }
    }
}

impl AgentLive for WatcherLive {
    fn last_activity_at(&self, scope: &str, agent: &str) -> Option<String> {
        self.watcher.as_ref()?.live(scope, agent)?.last_activity_at
    }
}
