//! The check run store: `<root>/<project>/<runId>.json`, one file per run, written by Brock's
//! local CI runner. Each run gets a number `C-n` per project in the order it was first seen;
//! the numbers are kept by an [`IdBook`] (the dashboard database) so they survive a restart.

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex, PoisonError, RwLock};

use argus_core::ids::is_key;
use serde_json::Value;

use crate::fields::text;
use crate::safe_path::{children, segment_ok};
use crate::stamp::StampCache;

/// Hands out the number of a run: the same number every time it is asked for the same run, a
/// new one for a run it has not seen. `None` when the number cannot be kept.
pub trait IdBook: Send + Sync {
    /// The `n` of `C-n` for a run of a project.
    fn number(&self, project: &str, run_id: &str) -> Option<u32>;
}

/// One run in the index.
#[derive(Debug, Clone, PartialEq)]
pub struct CheckRun {
    /// The `n` of `C-n`.
    pub n: u32,
    /// The project key (the folder).
    pub project: String,
    /// The file name without `.json`.
    pub run_id: String,
    /// The record as Brock wrote it.
    pub record: Value,
}

impl CheckRun {
    /// `C-n`.
    #[must_use]
    pub fn id(&self) -> String {
        format!("C-{}", self.n)
    }
}

/// The runs on disk, indexed.
pub struct CheckStore {
    root: PathBuf,
    ids: Arc<dyn IdBook>,
    index: RwLock<Arc<Vec<CheckRun>>>,
    cache: Mutex<StampCache<Value>>,
    numbers: Mutex<HashMap<(String, String), u32>>,
}

type Found = (String, String, Value);

fn read_record(path: &Path) -> Option<Value> {
    let record: Value = serde_json::from_slice(&std::fs::read(path).ok()?).ok()?;
    (record.is_object() && record["kind"].as_str().is_none_or(|k| k == "check-run"))
        .then_some(record)
}

fn run_id_of(path: &Path) -> Option<String> {
    let name = path.file_name()?.to_str()?;
    let id = name.strip_suffix(".json")?;
    segment_ok(name).then(|| id.to_owned())
}

fn entries(dir: &Path) -> Vec<(String, PathBuf)> {
    children(dir, |t| !t.is_symlink())
}

impl CheckStore {
    /// A store over a folder; call [`Self::rescan`] to fill the index.
    #[must_use]
    pub fn new(root: &Path, ids: Arc<dyn IdBook>) -> Self {
        CheckStore {
            root: root.to_path_buf(),
            ids,
            index: RwLock::default(),
            cache: Mutex::default(),
            numbers: Mutex::default(),
        }
    }

    /// The folder of the store.
    #[must_use]
    pub fn root(&self) -> &Path {
        &self.root
    }

    /// The index now, newest first.
    #[must_use]
    pub fn runs(&self) -> Arc<Vec<CheckRun>> {
        Arc::clone(&self.index.read().unwrap_or_else(PoisonError::into_inner))
    }

    fn read_all(&self) -> Vec<Found> {
        let mut cache = self.cache.lock().unwrap_or_else(PoisonError::into_inner);
        let mut found = Vec::new();
        for (project, dir) in entries(&self.root).into_iter().filter(|(p, _)| is_key(p)) {
            for (_, file) in entries(&dir) {
                let Some(run_id) = run_id_of(&file) else {
                    continue;
                };
                if let Some(record) = cache.get(&file, read_record) {
                    found.push((project.clone(), run_id, record));
                }
            }
        }
        cache.sweep();
        found
    }

    /// The numbers of the runs: the ones already known, then the new ones in the order they
    /// started (first seen), each asked of the book.
    fn numbered(&self, mut found: Vec<Found>) -> Vec<CheckRun> {
        let mut numbers = self.numbers.lock().unwrap_or_else(PoisonError::into_inner);
        found.sort_by(|a, b| (text(&a.2, "startedAt"), &a.1).cmp(&(text(&b.2, "startedAt"), &b.1)));
        let mut runs = Vec::new();
        for (project, run_id, record) in found {
            let key = (project, run_id);
            let n = numbers.get(&key).copied().or_else(|| {
                let n = self.ids.number(&key.0, &key.1)?;
                numbers.insert(key.clone(), n);
                Some(n)
            });
            if let Some(n) = n {
                runs.push(CheckRun {
                    n,
                    project: key.0,
                    run_id: key.1,
                    record,
                });
            }
        }
        runs
    }

    /// Reads the folder again and replaces the index.
    pub fn rescan(&self) {
        let mut runs = self.numbered(self.read_all());
        runs.sort_by(|a, b| (&a.project, b.n).cmp(&(&b.project, a.n)));
        *self.index.write().unwrap_or_else(PoisonError::into_inner) = Arc::new(runs);
    }

    /// The runs of a project, newest first, narrowed to a commit (a prefix of at least 7
    /// characters either way) and to a pull request.
    #[must_use]
    pub fn of_project(
        &self,
        project: &str,
        commit: Option<&str>,
        pr: Option<&str>,
    ) -> Vec<CheckRun> {
        let mut runs: Vec<CheckRun> = self
            .runs()
            .iter()
            .filter(|r| r.project == project)
            .filter(|r| commit.is_none_or(|c| same_commit(&text(&r.record, "commit"), c)))
            .filter(|r| pr.is_none_or(|p| text(&r.record, "pr").eq_ignore_ascii_case(p)))
            .cloned()
            .collect();
        runs.sort_by_key(|r| std::cmp::Reverse(r.n));
        runs
    }

    /// One run of a project by its `C-n`.
    #[must_use]
    pub fn find(&self, project: &str, id: &str) -> Option<CheckRun> {
        let n: u32 = id.strip_prefix("C-")?.parse().ok()?;
        self.runs()
            .iter()
            .find(|r| r.project == project && r.n == n)
            .cloned()
    }

    /// How many runs wait for the go.
    #[must_use]
    pub fn pending(&self) -> usize {
        let pending = |r: &&CheckRun| crate::replay_of(&r.record).is_some_and(|x| x.is_pending());
        self.runs().iter().filter(pending).count()
    }
}

fn same_commit(have: &str, asked: &str) -> bool {
    let (have, asked) = (have.to_ascii_lowercase(), asked.to_ascii_lowercase());
    let short = have.len().min(asked.len()) >= 7;
    have == asked || (short && (have.starts_with(&asked) || asked.starts_with(&have)))
}
