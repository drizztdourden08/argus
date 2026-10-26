//! The boards as the sub-agent services see them: the projects, the agents and threads of the
//! exported copy, and a write under the board lock on the project's queue.

use std::path::PathBuf;

use argus_agents::record::{Record, id_of};
use argus_agents::{AgentsError, Board, BoardStore};
use argus_board::store::{LockWait, with_board_async};
use argus_sync::ScopeQueue;
use serde_json::{Map, Value};
use tokio::runtime::Handle;

use crate::boards::BoardReader;

/// The boards of the sync service, written with the lock the CLI uses.
pub struct AgentBoards {
    boards: BoardReader,
    home: PathBuf,
    queue: ScopeQueue,
    runtime: Handle,
}

impl AgentBoards {
    /// Boards read through `boards` and written under `home`. A write blocks the calling
    /// thread, which must not belong to the runtime's workers: the services run on blocking
    /// threads.
    #[must_use]
    pub fn new(boards: BoardReader, home: PathBuf, queue: ScopeQueue, runtime: Handle) -> Self {
        AgentBoards {
            boards,
            home,
            queue,
            runtime,
        }
    }
}

fn records(root: &Value, name: &str) -> Vec<Record> {
    root.get(name)
        .and_then(Value::as_object)
        .map(|m| m.values().filter_map(|v| v.as_object().cloned()).collect())
        .unwrap_or_default()
}

fn board_of(root: &Value) -> Board {
    Board {
        agents: records(root, "agents"),
        threads: records(root, "threads"),
    }
}

/// Puts the records that changed back into their collection, by id.
fn put_back(root: &mut Value, name: &str, before: &[Record], after: &[Record]) {
    let Some(map) = root.as_object_mut() else {
        return;
    };
    let entry = map
        .entry(name.to_owned())
        .or_insert_with(|| Value::Object(Map::new()));
    let Some(target) = entry.as_object_mut() else {
        return;
    };
    for doc in after {
        let unchanged = before.iter().any(|b| b == doc);
        if !unchanged {
            target.insert(id_of(doc).to_owned(), Value::Object(doc.clone()));
        }
    }
}

impl BoardStore for AgentBoards {
    fn projects(&self) -> Vec<String> {
        self.boards.projects().into_iter().map(|p| p.key).collect()
    }

    fn snapshot(&self, key: &str) -> Option<Board> {
        self.boards.board(key).map(|b| board_of(&b))
    }

    fn write(&self, key: &str, change: &mut dyn FnMut(&mut Board)) -> Result<(), AgentsError> {
        let job = with_board_async(&self.home, key, LockWait::default(), |root| {
            let before = board_of(root);
            let mut board = before.clone();
            change(&mut board);
            put_back(root, "agents", &before.agents, &board.agents);
            put_back(root, "threads", &before.threads, &board.threads);
            Ok(())
        });
        self.runtime
            .block_on(self.queue.run(key, job))
            .map_err(|e| AgentsError::Board(key.to_owned(), e.to_string()))
    }
}
