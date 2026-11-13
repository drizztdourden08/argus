//! The bounded look inside one scratch folder: is anything in it younger than the cutoff, and does it hold a `.git`
//! anywhere. A link is a leaf, never entered. One walk answers both, and stops at the first reason to keep the folder.

use crate::fs::Fs;
use crate::paths::join_path;

/// How many entries the walk visits before it gives up.
const WALK_CAP: u32 = 200_000;

/// How deep the walk goes.
const MAX_DEPTH: u32 = 24;

/// What the walk found. Only `Old` lets a folder go; every other answer keeps it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) enum Look {
    /// Nothing inside, the folder itself included, is as young as the cutoff, and no `.git` is in it.
    Old,
    /// Something was touched at or after the cutoff (or has no readable time).
    Young,
    /// A `.git` (file, folder or link) at this path.
    Git(String),
    /// More entries or levels than the bounds: unknown, never "nothing found".
    TooBig,
    /// A folder that could not be read, with the reason.
    Unreadable(String),
}

/// Look in the folder, its own time first.
pub(super) fn look(fs: &dyn Fs, dir: &str, cutoff_ms: i64) -> Look {
    match fs.lstat(dir) {
        Ok(stat) if stat.mtime_ms.is_some_and(|m| m < cutoff_ms) => {}
        Ok(_) => return Look::Young,
        Err(err) => return Look::Unreadable(format!("{dir}: {err}")),
    }
    let mut budget = WALK_CAP;
    walk(fs, dir, 0, cutoff_ms, &mut budget)
}

fn walk(fs: &dyn Fs, dir: &str, level: u32, cutoff_ms: i64, budget: &mut u32) -> Look {
    if level > MAX_DEPTH {
        return Look::TooBig;
    }
    let entries = match fs.read_dir(dir) {
        Ok(entries) => entries,
        Err(err) if err.kind() == std::io::ErrorKind::NotFound => return Look::Old,
        Err(err) => return Look::Unreadable(format!("{dir}: {err}")),
    };
    for entry in entries {
        let path = join_path(dir, &entry.name);
        if entry.name == ".git" {
            return Look::Git(path);
        }
        if *budget == 0 {
            return Look::TooBig;
        }
        *budget -= 1;
        let Ok(stat) = fs.lstat(&path) else {
            continue;
        };
        if stat.mtime_ms.is_none_or(|m| m >= cutoff_ms) {
            return Look::Young;
        }
        if entry.is_dir && !entry.is_link {
            match walk(fs, &path, level + 1, cutoff_ms, budget) {
                Look::Old => {}
                other => return other,
            }
        }
    }
    Look::Old
}
