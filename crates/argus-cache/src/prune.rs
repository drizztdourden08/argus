//! Carrying out a plan: whole `<buildKey>` folders, then the empty version and name folders, then the left-over temp
//! folders. A folder that cannot be moved out of sight stays where it is, whole.

use crate::entry::Entry;
use crate::outcome::{Deleted, Outcome};
use crate::plan::Plan;
use crate::remove::{RemoveError, delete_tree, drop_if_empty, parents_of, park};
use argus_core::clock::iso_from_ms;
use argus_worktrees::Fs;

/// Where the deletes of a run are noted.
struct Run {
    outcome: Outcome,
}

impl Run {
    fn fail(&mut self, path: &str, error: RemoveError) {
        match error {
            RemoveError::Outside(_) | RemoveError::Link(_) => {
                self.outcome.refused.push(error.to_string());
            }
            RemoveError::Io(e) => self.outcome.failed.push(format!("{path}: {e}")),
        }
    }

    fn entry(&mut self, fs: &dyn Fs, root: &str, entry: &Entry, tag: &str) {
        let parked = match park(fs, root, &entry.path, tag) {
            Ok(parked) => parked,
            Err(e) => return self.fail(&entry.path, e),
        };
        self.outcome.deleted.push(Deleted::of(entry));
        self.outcome.freed_bytes += entry.size_bytes;
        if let Err(e) = delete_tree(fs, root, &parked) {
            self.outcome
                .failed
                .push(format!("{parked}: left in .tmp for the next prune ({e})"));
        }
        for folder in parents_of(&entry.path) {
            if !drop_if_empty(fs, root, &folder) {
                break;
            }
        }
    }
}

/// Deletes what the plan lists. Nothing outside `root` is touched.
pub fn run(fs: &dyn Fs, root: &str, plan: &Plan, now_ms: i64) -> Outcome {
    let mut run = Run {
        outcome: Outcome {
            at: iso_from_ms(now_ms),
            ..Outcome::default()
        },
    };
    for (n, entry) in plan.entries.iter().enumerate() {
        run.entry(fs, root, entry, &format!("pruned-{now_ms}-{n}"));
    }
    for tmp in &plan.tmp {
        match delete_tree(fs, root, &tmp.path) {
            Ok(()) => {
                run.outcome.tmp_deleted += 1;
                run.outcome.freed_bytes += tmp.size_bytes;
            }
            Err(e) => run.fail(&tmp.path, e),
        }
    }
    run.outcome
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::entry::Kind;
    use crate::tmp::Tmp;
    use argus_worktrees::{RealFs, clean_path};
    use std::fs;

    fn entry_at(path: &str) -> Entry {
        Entry {
            kind: Kind::Package,
            name: "n".to_owned(),
            version: "1.0.0".to_owned(),
            build_key: "node-aa".to_owned(),
            path: path.to_owned(),
            size_bytes: 7,
            last_used_ms: 0,
        }
    }

    #[test]
    fn a_plan_that_names_a_path_outside_the_root_is_refused_and_nothing_is_deleted() {
        let dir = tempfile::tempdir().expect("temp dir");
        let base = clean_path(&dir.path().to_string_lossy());
        let (root, outside) = (format!("{base}/a/cache"), format!("{base}/a/outside"));
        fs::create_dir_all(&root).expect("root");
        fs::create_dir_all(format!("{outside}/files")).expect("outside");
        fs::write(format!("{outside}/files/keep.js"), "x").expect("file");
        let plan = Plan {
            entries: vec![
                entry_at(&outside),
                entry_at(&format!("{root}/package/../../a/outside")),
            ],
            tmp: vec![Tmp {
                path: outside.clone(),
                size_bytes: 1,
            }],
        };
        let outcome = run(&RealFs, &root, &plan, 0);
        assert_eq!(outcome.refused.len(), 3, "{outcome:?}");
        assert_eq!(outcome.deleted, Vec::new());
        assert_eq!((outcome.tmp_deleted, outcome.freed_bytes), (0, 0));
        assert!(fs::exists(format!("{outside}/files/keep.js")).expect("exists"));
        assert!(outcome.summary().contains("3 refused"));
    }
}
