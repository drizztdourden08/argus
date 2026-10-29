//! Reading the cache: every `<buildKey>` folder under `package/` and `electron/`, with its size and last use.

use crate::entry::{Entry, Kind};
use crate::manifest;
use crate::segment::{decode_name, is_safe, join};
use argus_worktrees::Fs;

/// The real sub-folders of a folder with a safe name, sorted. A link is never one.
fn child_dirs(fs: &dyn Fs, path: &str) -> Vec<String> {
    let mut names: Vec<String> = fs
        .read_dir(path)
        .unwrap_or_default()
        .into_iter()
        .filter(|e| e.is_dir && !e.is_link && is_safe(&e.name))
        .map(|e| e.name)
        .collect();
    names.sort();
    names
}

/// The bytes of the files below a folder; a link counts for nothing and is not entered.
pub fn tree_size(fs: &dyn Fs, path: &str) -> u64 {
    let mut total = 0;
    for child in fs.read_dir(path).unwrap_or_default() {
        let child_path = join(path, &child.name);
        let Ok(stat) = fs.lstat(&child_path) else {
            continue;
        };
        if stat.is_link {
            continue;
        }
        total += if stat.is_dir {
            tree_size(fs, &child_path)
        } else {
            stat.size
        };
    }
    total
}

fn entry_at(fs: &dyn Fs, kind: Kind, names: [&str; 3], path: String, now_ms: i64) -> Entry {
    let facts = manifest::read(fs, &path);
    let folder_ms = || fs.lstat(&path).ok().and_then(|s| s.mtime_ms);
    Entry {
        kind,
        name: decode_name(names[0]),
        version: names[1].to_owned(),
        build_key: names[2].to_owned(),
        size_bytes: facts.size_bytes.unwrap_or_else(|| tree_size(fs, &path)),
        last_used_ms: facts.used_ms.or_else(folder_ms).unwrap_or(now_ms),
        path,
    }
}

fn entries_of_kind(fs: &dyn Fs, root: &str, kind: Kind, now_ms: i64) -> Vec<Entry> {
    let top = join(root, kind.dir());
    let mut found = Vec::new();
    for name in child_dirs(fs, &top) {
        let name_path = join(&top, &name);
        for version in child_dirs(fs, &name_path) {
            let version_path = join(&name_path, &version);
            for key in child_dirs(fs, &version_path) {
                let path = join(&version_path, &key);
                found.push(entry_at(fs, kind, [&name, &version, &key], path, now_ms));
            }
        }
    }
    found
}

/// Every entry of the cache.
pub fn entries(fs: &dyn Fs, root: &str, now_ms: i64) -> Vec<Entry> {
    Kind::ALL
        .into_iter()
        .flat_map(|kind| entries_of_kind(fs, root, kind, now_ms))
        .collect()
}
