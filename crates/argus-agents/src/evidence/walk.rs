//! The newest file under a folder, for the one sign of life a transcript cannot give: an agent writing files.

use crate::fs::FileSystem;
use std::path::Path;

/// A worktree walk stays cheap: it skips what no agent edits by hand and stops after this many entries.
const SKIP_DIRS: [&str; 5] = [".git", "node_modules", "target", ".next", ".cache"];
const BUDGET: u32 = 6000;
const DEPTH: u32 = 8;

struct Walk<'a> {
    fs: &'a dyn FileSystem,
    budget: u32,
    newest: i64,
    enough: i64,
}

impl Walk<'_> {
    /// True when the walk is over: out of budget, or a file at or after `enough` was found.
    fn visit(&mut self, dir: &Path, depth: u32) -> bool {
        let Ok(entries) = self.fs.list(dir) else {
            return false;
        };
        let mut below = Vec::new();
        for entry in entries {
            if self.budget == 0 {
                return true;
            }
            self.budget -= 1;
            if entry.is_dir {
                if depth < DEPTH && !SKIP_DIRS.contains(&entry.name.as_str()) {
                    below.push(dir.join(&entry.name));
                }
                continue;
            }
            if let Some(stat) = self.fs.stat(&dir.join(&entry.name))
                && stat.mtime_ms > self.newest
            {
                self.newest = stat.mtime_ms;
                if self.newest >= self.enough {
                    return true;
                }
            }
        }
        below.iter().any(|sub| self.visit(sub, depth + 1))
    }
}

/// The modification time of the newest file under `dir`, stopping early at the first one at or after `enough`; 0 when
/// there is none. Bounded by 6000 entries and 8 levels.
pub(crate) fn newest_file_mtime(fs: &dyn FileSystem, dir: &Path, enough: i64) -> i64 {
    let mut walk = Walk {
        fs,
        budget: BUDGET,
        newest: 0,
        enough,
    };
    walk.visit(dir, 0);
    walk.newest
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::fs::StdFs;
    use std::fs;
    use std::time::{Duration, UNIX_EPOCH};

    fn file_at(dir: &Path, rel: &str, ms: u64) {
        let path = dir.join(rel);
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        fs::write(&path, "x").unwrap();
        let file = fs::OpenOptions::new().write(true).open(&path).unwrap();
        file.set_modified(UNIX_EPOCH + Duration::from_millis(ms))
            .unwrap();
    }

    #[test]
    fn the_newest_file_wins_and_folders_no_agent_edits_are_skipped() {
        let dir = tempfile::tempdir().unwrap();
        file_at(dir.path(), "src/old.rs", 1_000);
        file_at(dir.path(), "src/deep/new.rs", 5_000);
        file_at(dir.path(), "node_modules/pkg/index.js", 9_000);
        file_at(dir.path(), "target/debug/out", 9_500);
        file_at(dir.path(), ".git/index", 9_900);
        assert_eq!(newest_file_mtime(&StdFs, dir.path(), i64::MAX), 5_000);
    }

    #[test]
    fn the_walk_stops_at_the_first_file_new_enough_and_an_empty_folder_has_none() {
        let dir = tempfile::tempdir().unwrap();
        assert_eq!(newest_file_mtime(&StdFs, dir.path(), 0), 0);
        file_at(dir.path(), "a.rs", 4_000);
        file_at(dir.path(), "z/b.rs", 8_000);
        assert!(newest_file_mtime(&StdFs, dir.path(), 3_000) >= 4_000);
        assert_eq!(newest_file_mtime(&StdFs, Path::new("/nowhere"), 0), 0);
    }
}
