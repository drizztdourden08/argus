//! `.tmp/` of the cache: entries being built, and what a build that was cut off left. A folder older than a day is
//! left over.

use crate::scan::tree_size;
use crate::segment::join;
use argus_worktrees::Fs;

pub const TMP_DIR: &str = ".tmp";
const DAY_MS: i64 = 86_400_000;

/// A left-over folder of `.tmp/`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Tmp {
    pub path: String,
    pub size_bytes: u64,
}

/// The folders of `.tmp/` last touched more than a day ago. A link is never listed, and nothing is when `.tmp` itself is
/// a link.
pub fn stale(fs: &dyn Fs, root: &str, now_ms: i64) -> Vec<Tmp> {
    let dir = join(root, TMP_DIR);
    if fs.lstat(&dir).is_ok_and(|s| s.is_link) {
        return Vec::new();
    }
    let mut found: Vec<Tmp> = fs
        .read_dir(&dir)
        .unwrap_or_default()
        .into_iter()
        .filter(|e| e.is_dir && !e.is_link)
        .filter_map(|e| {
            let path = join(&dir, &e.name);
            let touched = fs.lstat(&path).ok()?.mtime_ms?;
            (now_ms - touched > DAY_MS).then(|| Tmp {
                size_bytes: tree_size(fs, &path),
                path,
            })
        })
        .collect();
    found.sort_by(|a, b| a.path.cmp(&b.path));
    found
}
