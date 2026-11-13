//! The worktree service's storage: a project's rules, keeps and answers live on its board
//! file, the "already told" and "made by a removal" memories in the key-value table. Shadow
//! mode writes neither.

use std::path::PathBuf;
use std::sync::Arc;

use argus_board::inbox::queue;
use argus_board::store::with_board;
use argus_core::StorageError;
use argus_launchers::{ListFilter, RunRow, RunService};
use argus_store::Db;
use argus_worktrees::{BoardFacts, Link, WorktreeStore};
use serde_json::json;

use super::auto::{AutoRepos, AutoSetting};
use super::facts::facts_of;
use crate::adapters::runs::RunHost;
use crate::boards::BoardReader;

/// The runs queued or running, at most this many.
const RUNS_SEEN: i64 = 500;

/// The boards and the key-value table as the worktree service sees them.
pub struct BoardWorktrees {
    boards: BoardReader,
    db: Arc<Db>,
    coord_home: PathBuf,
    shadow: bool,
    runs: Option<Arc<RunService<RunHost>>>,
    auto: AutoRepos,
}

impl BoardWorktrees {
    /// The storage over the boards under `coord_home` and the database.
    #[must_use]
    pub fn new(boards: BoardReader, db: Arc<Db>, coord_home: PathBuf, shadow: bool) -> Self {
        BoardWorktrees {
            boards,
            db,
            coord_home,
            shadow,
            runs: None,
            auto: AutoRepos::new(AutoSetting::default()),
        }
    }

    /// The same storage, with the settings of the automatic repo rules.
    #[must_use]
    pub fn with_auto(mut self, setting: AutoSetting) -> Self {
        self.auto = AutoRepos::new(setting);
        self
    }

    /// The same storage, with the run engine the hygiene asks which runs work in a worktree.
    #[must_use]
    pub fn with_runs(mut self, runs: Option<Arc<RunService<RunHost>>>) -> Self {
        self.runs = runs;
        self
    }
}

fn run_link(run: RunRow) -> Link {
    Link {
        what: format!("run {} ({})", run.id, run.name),
        path: Some(run.cwd),
        ..Link::default()
    }
}

impl WorktreeStore for BoardWorktrees {
    fn overlord_board(&self) -> String {
        self.boards.overlord().unwrap_or_default()
    }

    fn board(&self, project: &str) -> Option<BoardFacts> {
        let mut facts = facts_of(&self.boards.board(project)?);
        facts.rules.extend(self.auto.of(&self.boards, project));
        Some(facts)
    }

    fn projects(&self) -> Vec<String> {
        self.boards
            .projects()
            .into_iter()
            .filter(|p| p.has_board)
            .map(|p| p.key)
            .collect()
    }

    fn kv_get(&self, key: &str) -> Option<String> {
        self.db.kv_get(key).ok().flatten()
    }

    fn kv_set(&self, key: &str, value: &str) -> Result<(), StorageError> {
        if self.shadow {
            return Ok(());
        }
        self.db
            .kv_set(key, value)
            .map_err(|e| StorageError(e.to_string()))
    }

    fn drop_kept(
        &self,
        project: &str,
        keep_ids: &[String],
        answer_ids: &[String],
    ) -> Result<(), StorageError> {
        if self.shadow {
            return Ok(());
        }
        with_board(&self.coord_home, project, false, |board| {
            for id in keep_ids {
                board["worktreeKeeps"].as_object_mut().map(|m| m.remove(id));
            }
            for id in answer_ids {
                board["worktreeAnswers"]
                    .as_object_mut()
                    .map(|m| m.remove(id));
            }
            board["dirty"] = json!({});
            board["versions"] = json!({});
            Ok(())
        })
        .map_err(|e| StorageError(e.to_string()))
    }

    fn post_inbox(&self, project: &str, thread: &str, text: &str) -> Result<bool, StorageError> {
        if self.shadow {
            return Ok(false);
        }
        with_board(&self.coord_home, project, false, |board| {
            Ok(queue(board, thread, "Argus", "note", text, None))
        })
        .map_err(|e| StorageError(e.to_string()))
    }

    fn live_runs(&self) -> Result<Vec<Link>, String> {
        let Some(runs) = &self.runs else {
            return Ok(Vec::new());
        };
        let filter = ListFilter {
            limit: Some(RUNS_SEEN),
            ..ListFilter::default()
        }
        .with_statuses("running, queued");
        Ok(runs.list_all(&filter).into_iter().map(run_link).collect())
    }
}
