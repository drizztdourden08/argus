//! The pair of a check run record (Brock's file) and the run of the run engine that made it: the
//! run prints the path of its record as its first line, and the pair is kept in the dashboard
//! database (`check-link:<project>:<runId>` holds `R-n`), so the checks answer shows `run` on the
//! record and a number is never lost across a restart. The guard's verdict on a record (the run
//! changed tracked files) is kept beside it (`check-flag:<project>:<runId>` holds the note).

use std::sync::Arc;

use argus_store::Db;

/// The pairs, over the database's key value table.
pub struct CheckLinks(Arc<Db>);

fn key_of(project: &str, run_id: &str) -> String {
    format!("check-link:{project}:{run_id}")
}

fn flag_key(project: &str, run_id: &str) -> String {
    format!("check-flag:{project}:{run_id}")
}

fn plain(name: &str) -> bool {
    !name.is_empty()
        && name
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || b"-_.".contains(&b))
}

/// The run id a first line names: the file name without `.json` of a path whose folder is the
/// project. `None` for any other line.
#[must_use]
pub fn run_id_of(project: &str, line: &str) -> Option<String> {
    let path = line.trim().replace('\\', "/");
    let mut parts = path.rsplit('/');
    let file = parts.next()?.strip_suffix(".json")?;
    (parts.next() == Some(project) && plain(file)).then(|| file.to_owned())
}

impl CheckLinks {
    /// The pairs kept in `db`.
    #[must_use]
    pub fn new(db: Arc<Db>) -> Self {
        CheckLinks(db)
    }

    /// Keeps the pair: the record `run_id` of `project` was made by the run `rid`. False when
    /// the database refused it.
    #[must_use]
    pub fn link(&self, project: &str, run_id: &str, rid: &str) -> bool {
        self.0.kv_set(&key_of(project, run_id), rid).is_ok()
    }

    /// The run that made a record, when one is known.
    #[must_use]
    pub fn run_of(&self, project: &str, run_id: &str) -> Option<String> {
        self.0.kv_get(&key_of(project, run_id)).ok().flatten()
    }

    /// Fails the record `run_id` of `project` with `note`: the run changed tracked files. False
    /// when the database refused it.
    #[must_use]
    pub fn flag(&self, project: &str, run_id: &str, note: &str) -> bool {
        self.0.kv_set(&flag_key(project, run_id), note).is_ok()
    }

    /// The guard's note on a record, when it failed one.
    #[must_use]
    pub fn flag_of(&self, project: &str, run_id: &str) -> Option<String> {
        self.0.kv_get(&flag_key(project, run_id)).ok().flatten()
    }
}
