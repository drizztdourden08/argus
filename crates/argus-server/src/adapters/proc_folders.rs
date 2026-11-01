//! Every folder of a project, for the process monitor's scope rule (a thread's own kill and the
//! reaper): the root, the folders of the worktree rules, the worktrees the scan found, the
//! `git worktree list` of the root (kept a few minutes) and the ci root.

use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::{Arc, Mutex, PoisonError};
use std::time::{Duration, Instant};

use argus_core::Coll;
use argus_core::remote::ci_root_of;
use argus_procs::{ProjectFolders, ProjectRoot};
use argus_store::Db;
use argus_worktrees::{GitExec, SystemExec};
use serde_json::Value;

use crate::boards::BoardReader;

const GIT_TIMEOUT: Duration = Duration::from_secs(10);
const GIT_KEEP: Duration = Duration::from_secs(5 * 60);

type Kept = HashMap<String, (Instant, Vec<String>)>;

/// Reads the folders of the projects from the boards, the worktree docs, git and `remote.json`.
pub struct FolderReader {
    boards: BoardReader,
    db: Arc<Db>,
    coord_home: PathBuf,
    git: Mutex<Kept>,
}

/// The non-empty `path` of each object of a list.
fn paths_in(list: &Value) -> Vec<String> {
    let all = list.as_array().into_iter().flatten();
    all.filter_map(|v| v["path"].as_str())
        .filter(|p| !p.trim().is_empty())
        .map(str::to_owned)
        .collect()
}

/// The folders `git worktree list --porcelain` prints.
fn listed(stdout: &str) -> Vec<String> {
    stdout
        .lines()
        .filter_map(|line| line.strip_prefix("worktree "))
        .map(|path| path.trim().to_owned())
        .filter(|path| !path.is_empty())
        .collect()
}

impl FolderReader {
    /// A reader over the boards, the doc store and the coord home.
    #[must_use]
    pub fn new(boards: BoardReader, db: Arc<Db>, coord_home: PathBuf) -> Self {
        FolderReader {
            boards,
            db,
            coord_home,
            git: Mutex::new(HashMap::new()),
        }
    }

    /// The worktrees git knows for the root, read again after a few minutes. Blocking: git runs.
    fn git_worktrees(&self, root: &str) -> Vec<String> {
        if root.trim().is_empty() {
            return Vec::new();
        }
        let mut kept = self.git.lock().unwrap_or_else(PoisonError::into_inner);
        let fresh = kept.get(root).filter(|(at, _)| at.elapsed() < GIT_KEEP);
        if let Some((_, list)) = fresh {
            return list.clone();
        }
        let args = ["worktree", "list", "--porcelain"].map(str::to_owned);
        let out = SystemExec::default().exec(root, &args, GIT_TIMEOUT);
        let list = if out.ok() {
            listed(&out.stdout)
        } else {
            Vec::new()
        };
        kept.insert(root.to_owned(), (Instant::now(), list.clone()));
        list
    }

    fn folders_of(&self, key: &str, root: &str) -> Vec<String> {
        let mut out = vec![root.to_owned()];
        if let Some(board) = self.boards.board(key) {
            out.extend(paths_in(&board["worktreeRules"]));
        }
        if let Ok(rows) = self.db.docs().list(key, Coll::Worktrees) {
            let paths = rows.iter().filter_map(|r| r.data["path"].as_str());
            out.extend(paths.map(str::to_owned));
        }
        out.extend(self.git_worktrees(root));
        out.extend(ci_root_of(&self.coord_home, key));
        out.sort();
        out.dedup();
        out
    }

    /// The folders of each project of these roots, one entry per key.
    #[must_use]
    pub fn of(&self, roots: Vec<ProjectRoot>) -> Vec<ProjectFolders> {
        let mut projects: Vec<ProjectFolders> = Vec::new();
        for root in roots {
            let folders = self.folders_of(&root.key, &root.root);
            match projects.iter_mut().find(|p| p.key == root.key) {
                Some(known) => known.folders.extend(folders),
                None => projects.push(ProjectFolders {
                    key: root.key,
                    folders,
                }),
            }
        }
        projects
    }
}

// test-tag: critical
#[cfg(test)]
mod tests {
    //! K1: the porcelain list and the rule paths parse to their folders.
    use super::*;
    use serde_json::json;

    #[test]
    fn the_porcelain_list_and_the_rules_give_their_folders() {
        let porcelain = "worktree X:/argus\nHEAD abc\nbranch refs/heads/main\n\nworktree X:/argus/.claude/worktrees/a\nHEAD def\ndetached\n";
        assert_eq!(
            listed(porcelain),
            ["X:/argus", "X:/argus/.claude/worktrees/a"]
        );
        let rules =
            json!([{ "path": "X:/argus/.claude/worktrees" }, { "path": "  " }, { "id": "r" }]);
        assert_eq!(paths_in(&rules), ["X:/argus/.claude/worktrees"]);
        assert_eq!(paths_in(&Value::Null), Vec::<String>::new());
    }
}
